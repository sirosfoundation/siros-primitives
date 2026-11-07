//! The identity reader's soundness on hand-assembled modules: what a function's hash covers, what
//! layout leaves alone, and what the reader refuses. Each offset that a refusal names comes from the
//! assembler below.

use bench_compare::Fingerprint;
use bench_tools::{Identity, ModuleError};

/// One instruction of a function under assembly.
#[derive(Clone)]
enum Instruction {
    /// `i32.const`.
    Const(i32),
    /// `call` of the function with this index in [`Sample::functions`].
    Call(usize),
    /// `i32.const` of the address of a symbol, at a distance, with its relocation.
    Address(&'static str, i32),
    /// `if` with an empty type.
    If,
    /// `end`.
    End,
    /// `unreachable`.
    Unreachable,
    /// `call_indirect` of the only type on table 0.
    CallIndirect,
    /// Bytes that the reader takes as an instruction.
    Raw(Vec<u8>),
}

/// A word of a symbol that holds the address of another symbol.
#[derive(Clone)]
struct Pointer {
    at: usize,
    target: &'static str,
    addend: i32,
}

/// A data symbol.
#[derive(Clone)]
struct Data {
    name: &'static str,
    /// Whether it lies in the `.rodata` segment; the other is `.data`.
    read_only: bool,
    bytes: Vec<u8>,
    pointers: Vec<Pointer>,
}

/// A module under assembly. The table holds the functions from its index 1.
#[derive(Clone)]
struct Sample {
    /// Bytes before the symbols of each segment: moves every address.
    padding: usize,
    /// Functions before the others: moves every function index.
    spacers: u32,
    functions: Vec<Vec<Instruction>>,
    symbols: Vec<Data>,
    /// Whether it holds the `linking` section.
    linking: bool,
    /// Adds this to the operand of the first address, against what its relocation says.
    wrong_operand: u32,
    /// Keeps a relocation on the first byte of the first function's body, which holds no operand.
    stray_relocation: bool,
}

/// The module, with the offsets that a refusal can name.
struct Assembled {
    bytes: Vec<u8>,
    /// Where the first [`Instruction::Raw`] stands.
    raw: Option<usize>,
    /// Where the operand of the first [`Instruction::Address`] stands.
    first_address: Option<usize>,
    /// Where the first function's body starts.
    first_body: usize,
}

fn unsigned(mut value: u32, out: &mut Vec<u8>) {
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        if value == 0 {
            out.push(byte);
            return;
        }
        out.push(byte | 0x80);
    }
}

fn signed(mut value: i32, out: &mut Vec<u8>) {
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        if (value == 0 && byte & 0x40 == 0) || (value == -1 && byte & 0x40 != 0) {
            out.push(byte);
            return;
        }
        out.push(byte | 0x80);
    }
}

fn name(text: &str, out: &mut Vec<u8>) {
    unsigned(text.len() as u32, out);
    out.extend_from_slice(text.as_bytes());
}

/// Appends a section.
fn section(id: u8, contents: &[u8], out: &mut Vec<u8>) {
    out.push(id);
    unsigned(contents.len() as u32, out);
    out.extend_from_slice(contents);
}

/// Appends a custom section.
fn custom(title: &str, contents: &[u8], out: &mut Vec<u8>) {
    let mut named = Vec::new();
    name(title, &mut named);
    named.extend_from_slice(contents);
    section(0, &named, out);
}

/// A relocation of a `reloc.` section.
fn relocation(kind: u8, offset: usize, symbol: usize, addend: i32, out: &mut Vec<u8>) {
    out.push(kind);
    unsigned(offset as u32, out);
    unsigned(symbol as u32, out);
    signed(addend, out);
}

/// The relocation types of the linker's conventions that an address takes.
const MEMORY_ADDR_SLEB: u8 = 4;
const MEMORY_ADDR_I32: u8 = 5;

/// The first address of the segments.
const FIRST_ADDRESS: usize = 1024;

impl Sample {
    fn symbol(&self, wanted: &str) -> usize {
        self.symbols
            .iter()
            .position(|symbol| symbol.name == wanted)
            .expect("a symbol of the sample")
    }

    /// Per symbol: the index of its segment and its offset there.
    fn lay_out(&self) -> Vec<(usize, usize)> {
        let mut next = [self.padding, self.padding];
        self.symbols
            .iter()
            .map(|symbol| {
                let segment = usize::from(!symbol.read_only);
                let offset = next[segment];
                next[segment] += symbol.bytes.len();
                (segment, offset)
            })
            .collect()
    }

    fn assemble(&self) -> Assembled {
        let places = self.lay_out();
        let size = |segment: usize| {
            self.symbols
                .iter()
                .zip(&places)
                .filter(|(_, place)| place.0 == segment)
                .map(|(symbol, _)| symbol.bytes.len())
                .sum::<usize>()
                + self.padding
        };
        let bases = [FIRST_ADDRESS, FIRST_ADDRESS + size(0) + 16];
        let address = |symbol: usize| {
            let (segment, offset) = places[symbol];
            (bases[segment] + offset) as i32
        };

        let mut out = b"\0asm\x01\0\0\0".to_vec();
        section(1, &[1, 0x60, 0, 0], &mut out);

        let total = self.spacers as usize + self.functions.len();
        let mut declared = Vec::new();
        unsigned(total as u32, &mut declared);
        declared.resize(declared.len() + total, 0);
        section(3, &declared, &mut out);

        let mut elements = Vec::new();
        unsigned(1, &mut elements);
        unsigned(0, &mut elements);
        elements.extend_from_slice(&[0x41, 1, 0x0b]);
        unsigned(self.functions.len() as u32, &mut elements);
        for function in 0..self.functions.len() as u32 {
            unsigned(self.spacers + function, &mut elements);
        }
        section(9, &elements, &mut out);

        let mut code = Vec::new();
        unsigned(total as u32, &mut code);
        let mut code_relocations = Vec::new();
        let mut relocated = 0_u32;
        let mut raw = None;
        let mut first_address = None;
        for _ in 0..self.spacers {
            code.extend_from_slice(&[2, 0, 0x0b]);
        }
        let mut first_body = 0;
        for (index, function) in self.functions.iter().enumerate() {
            let mut body = vec![0];
            let mut places_in_body = Vec::new();
            let mut raw_in_body = None;
            for instruction in function {
                match instruction {
                    Instruction::Const(value) => {
                        body.push(0x41);
                        signed(*value, &mut body);
                    }
                    Instruction::Call(callee) => {
                        body.push(0x10);
                        unsigned(self.spacers + *callee as u32, &mut body);
                    }
                    Instruction::Address(symbol, addend) => {
                        let symbol = self.symbol(symbol);
                        body.push(0x41);
                        places_in_body.push((body.len(), symbol, *addend));
                        let wrong = if places_in_body.len() == 1 && first_address.is_none() {
                            self.wrong_operand as i32
                        } else {
                            0
                        };
                        signed(address(symbol) + addend + wrong, &mut body);
                    }
                    Instruction::If => body.extend_from_slice(&[0x04, 0x40]),
                    Instruction::End => body.push(0x0b),
                    Instruction::Unreachable => body.push(0x00),
                    Instruction::CallIndirect => body.extend_from_slice(&[0x11, 0, 0]),
                    Instruction::Raw(bytes) => {
                        raw_in_body = raw_in_body.or(Some(body.len()));
                        body.extend_from_slice(bytes);
                    }
                }
            }
            body.push(0x0b);
            unsigned(body.len() as u32, &mut code);
            let start = code.len();
            if index == 0 {
                first_body = start;
                if self.stray_relocation {
                    relocation(MEMORY_ADDR_SLEB, start, 0, 0, &mut code_relocations);
                    relocated += 1;
                }
            }
            for (at, symbol, addend) in places_in_body {
                first_address = first_address.or(Some(start + at));
                relocation(
                    MEMORY_ADDR_SLEB,
                    start + at,
                    symbol,
                    addend,
                    &mut code_relocations,
                );
                relocated += 1;
            }
            if let Some(at) = raw_in_body {
                raw = raw.or(Some(start + at));
            }
            code.extend_from_slice(&body);
        }
        section(10, &code, &mut out);
        let code_at = out.len() - code.len();
        let code_index = 3_u32;

        let mut data = Vec::new();
        unsigned(2, &mut data);
        let mut data_relocations = Vec::new();
        let mut data_relocated = 0_u32;
        for (segment, base) in bases.into_iter().enumerate() {
            unsigned(0, &mut data);
            data.push(0x41);
            signed(base as i32, &mut data);
            data.push(0x0b);
            unsigned(size(segment) as u32, &mut data);
            let start = data.len();
            data.resize(start + size(segment), 0);
            for (index, symbol) in self.symbols.iter().enumerate() {
                let (found, offset) = places[index];
                if found != segment {
                    continue;
                }
                let place = start + offset;
                data[place..place + symbol.bytes.len()].copy_from_slice(&symbol.bytes);
                for pointer in &symbol.pointers {
                    let target = self.symbol(pointer.target);
                    let word = (address(target) + pointer.addend) as u32;
                    let at = place + pointer.at;
                    data[at..at + 4].copy_from_slice(&word.to_le_bytes());
                    relocation(
                        MEMORY_ADDR_I32,
                        at,
                        target,
                        pointer.addend,
                        &mut data_relocations,
                    );
                    data_relocated += 1;
                }
            }
        }
        let data_index = code_index + 1;
        section(11, &data, &mut out);

        if self.linking {
            let mut linking = vec![2];
            let mut info = Vec::new();
            unsigned(2, &mut info);
            for segment in [".rodata", ".data"] {
                name(segment, &mut info);
                unsigned(0, &mut info);
                unsigned(0, &mut info);
            }
            linking.push(5);
            unsigned(info.len() as u32, &mut linking);
            linking.extend_from_slice(&info);
            let mut table = Vec::new();
            unsigned(self.symbols.len() as u32, &mut table);
            for (symbol, (segment, offset)) in self.symbols.iter().zip(&places) {
                table.push(1);
                unsigned(0, &mut table);
                name(symbol.name, &mut table);
                unsigned(*segment as u32, &mut table);
                unsigned(*offset as u32, &mut table);
                unsigned(symbol.bytes.len() as u32, &mut table);
            }
            linking.push(8);
            unsigned(table.len() as u32, &mut linking);
            linking.extend_from_slice(&table);
            custom("linking", &linking, &mut out);
        }

        let mut reloc_code = Vec::new();
        unsigned(code_index, &mut reloc_code);
        unsigned(relocated, &mut reloc_code);
        reloc_code.extend_from_slice(&code_relocations);
        custom("reloc.CODE", &reloc_code, &mut out);

        let mut reloc_data = Vec::new();
        unsigned(data_index, &mut reloc_data);
        unsigned(data_relocated, &mut reloc_data);
        reloc_data.extend_from_slice(&data_relocations);
        custom("reloc.DATA", &reloc_data, &mut out);

        Assembled {
            bytes: out,
            raw: raw.map(|at| at + code_at),
            first_address: first_address.map(|at| at + code_at),
            first_body: first_body + code_at,
        }
    }

    /// The hash of each function of [`Sample::functions`].
    fn hashes(&self) -> Vec<u128> {
        let assembled = self.assemble();
        let mut identity = Identity::of(&assembled.bytes).expect("a sample that the reader takes");
        (0..self.functions.len() as u32)
            .map(|function| {
                identity
                    .entry(function + 1)
                    .expect("a function of the table")
                    .hash
            })
            .collect()
    }

    fn refusal(&self) -> ModuleError {
        let assembled = self.assemble();
        match Identity::of(&assembled.bytes) {
            Ok(_) => panic!("the reader took a module that it must refuse"),
            Err(error) => error,
        }
    }
}

/// The functions of [`sample`], by index.
mod function {
    pub const LEAF: usize = 0;
    pub const OTHER_LEAF: usize = 1;
    pub const VIA_LEAF: usize = 2;
    pub const VIA_OTHER_LEAF: usize = 3;
    pub const READS_TABLE: usize = 4;
    pub const READS_OTHER_TABLE: usize = 5;
    pub const READS_POINTER: usize = 6;
    pub const EVEN: usize = 7;
    pub const ODD: usize = 8;
    pub const PANICS: usize = 9;
    pub const MAY_PANIC: usize = 10;
    pub const READS_STATIC: usize = 11;
    pub const READS_LOCATION: usize = 12;
    pub const COUNT: usize = 13;
}

/// The bytes of a `core::panic::Location` without the address of its file.
fn location(line: u32) -> Vec<u8> {
    let mut bytes = vec![0; 4];
    bytes.extend_from_slice(&7_u32.to_le_bytes());
    bytes.extend_from_slice(&line.to_le_bytes());
    bytes.extend_from_slice(&5_u32.to_le_bytes());
    bytes
}

fn sample() -> Sample {
    use Instruction::{Address, Call, Const, End, If, Unreachable};
    let data = |name, read_only, bytes: &[u8]| Data {
        name,
        read_only,
        bytes: bytes.to_vec(),
        pointers: Vec::new(),
    };
    let mut functions = vec![Vec::new(); function::COUNT];
    functions[function::LEAF] = vec![Const(7)];
    functions[function::OTHER_LEAF] = vec![Const(9)];
    functions[function::VIA_LEAF] = vec![Call(function::LEAF)];
    functions[function::VIA_OTHER_LEAF] = vec![Call(function::OTHER_LEAF)];
    functions[function::READS_TABLE] = vec![Address("table", 4)];
    functions[function::READS_OTHER_TABLE] = vec![Address("other table", 0)];
    functions[function::READS_POINTER] = vec![Address("pointer", 0)];
    functions[function::EVEN] = vec![Const(1), If, Call(function::ODD), End];
    functions[function::ODD] = vec![
        Const(1),
        If,
        Call(function::EVEN),
        End,
        Call(function::LEAF),
    ];
    functions[function::PANICS] = vec![Const(3), Unreachable];
    functions[function::MAY_PANIC] = vec![Const(0), If, Call(function::PANICS), End];
    functions[function::READS_STATIC] = vec![Address("static", 0)];
    functions[function::READS_LOCATION] = vec![Address("location", 0)];
    Sample {
        padding: 0,
        spacers: 0,
        functions,
        symbols: vec![
            data("table", true, &[1, 2, 3, 4, 5, 6, 7, 8]),
            data("other table", true, &[8, 7, 6, 5, 4, 3, 2, 1]),
            Data {
                name: "pointer",
                read_only: true,
                bytes: vec![0; 8],
                pointers: vec![Pointer {
                    at: 0,
                    target: "table",
                    addend: 4,
                }],
            },
            data("static", false, &[1, 2, 3, 4]),
            data("file", true, b"file.rs"),
            Data {
                name: "location",
                read_only: true,
                bytes: location(40),
                pointers: vec![Pointer {
                    at: 0,
                    target: "file",
                    addend: 0,
                }],
            },
        ],
        linking: true,
        wrong_operand: 0,
        stray_relocation: false,
    }
}

impl Sample {
    fn data_mut(&mut self, wanted: &str) -> &mut Data {
        self.symbols
            .iter_mut()
            .find(|symbol| symbol.name == wanted)
            .expect("a symbol of the sample")
    }
}

/// Which functions a change must change, and no others. A false "same" returns a stale figure; a
/// false "differs" costs a measurement.
#[test]
fn a_change_reaches_the_functions_that_can_run_it_and_no_others() {
    use function::{
        EVEN, LEAF, MAY_PANIC, ODD, OTHER_LEAF, PANICS, READS_LOCATION, READS_OTHER_TABLE,
        READS_POINTER, READS_STATIC, READS_TABLE, VIA_LEAF,
    };
    type Change = (&'static str, fn(&mut Sample), &'static [usize]);
    let cases: [Change; 9] = [
        (
            "an immediate reaches its callers, through a cycle of calls too",
            |sample| sample.functions[LEAF][0] = Instruction::Const(8),
            &[LEAF, VIA_LEAF, EVEN, ODD],
        ),
        (
            "an immediate in a cycle of calls reaches each of its functions",
            |sample| sample.functions[EVEN][0] = Instruction::Const(2),
            &[EVEN, ODD],
        ),
        (
            "a callee that another function replaces",
            |sample| sample.functions[VIA_LEAF][0] = Instruction::Call(OTHER_LEAF),
            &[VIA_LEAF],
        ),
        (
            "a byte of a table reaches what names it, through a pointer too",
            |sample| sample.data_mut("table").bytes[6] = 0,
            &[READS_TABLE, READS_POINTER],
        ),
        (
            "the distance of an address from its symbol",
            |sample| sample.functions[READS_TABLE][0] = Instruction::Address("table", 5),
            &[READS_TABLE],
        ),
        (
            "the initial value of a static",
            |sample| sample.data_mut("static").bytes[0] = 0,
            &[READS_STATIC],
        ),
        (
            "the name of a static: two statics of equal bytes are different memory",
            |sample| {
                sample.data_mut("static").name = "renamed";
                sample.functions[READS_STATIC][0] = Instruction::Address("renamed", 0);
            },
            &[READS_STATIC],
        ),
        (
            "a byte of another table",
            |sample| sample.data_mut("other table").bytes[0] = 0,
            &[READS_OTHER_TABLE],
        ),
        (
            "the target of a pointer",
            |sample| sample.data_mut("pointer").pointers[0].target = "other table",
            &[READS_POINTER],
        ),
    ];
    let base = sample().hashes();
    for (what, change, expected) in cases {
        let mut changed = sample();
        change(&mut changed);
        let differ: Vec<usize> = base
            .iter()
            .zip(changed.hashes())
            .enumerate()
            .filter(|(_, (before, after))| **before != *after)
            .map(|(index, _)| index)
            .collect();
        let mut expected = expected.to_vec();
        expected.sort_unstable();
        assert_eq!(differ, expected, "{what}");
    }

    // A function that cannot return ends the run that calls it, and a panic's location reaches only
    // its message: their changes leave the hash of what surrounds them.
    let mut changed = sample();
    changed.functions[PANICS][0] = Instruction::Const(4);
    changed.data_mut("location").bytes = location(41);
    let after = changed.hashes();
    assert_eq!(base[MAY_PANIC], after[MAY_PANIC]);
    assert_eq!(base[READS_LOCATION], after[READS_LOCATION]);
    assert_ne!(base[PANICS], after[PANICS]);
}

/// Moved function indices and addresses are what every rebuild does to code that stayed as it was.
#[test]
fn layout_leaves_the_hash() {
    let moved = Sample {
        padding: 64,
        spacers: 3,
        ..sample()
    };
    assert_eq!(sample().hashes(), moved.hashes());
}

/// A loop that can run code that the hash does not hold always counts as changed.
#[test]
fn a_reachable_indirect_call_marks_every_caller() {
    use function::{LEAF, VIA_LEAF, VIA_OTHER_LEAF};
    let mut indirect = sample();
    indirect.functions[LEAF] = vec![Instruction::Const(0), Instruction::CallIndirect];
    let assembled = indirect.assemble();
    let mut identity = Identity::of(&assembled.bytes).expect("a sample that the reader takes");
    let mut marked = |slot: usize| {
        identity
            .entry(slot as u32 + 1)
            .expect("a function of the table")
            .indirect
    };
    assert!(marked(LEAF));
    assert!(marked(VIA_LEAF));
    assert!(!marked(VIA_OTHER_LEAF));
}

/// Each refusal names the offset of what it refuses, and the sample without the fault is taken.
#[test]
fn the_reader_refuses_what_it_cannot_read() {
    use function::{LEAF, READS_TABLE};
    assert!(!sample().hashes().is_empty());

    let mut unknown = sample();
    unknown.functions[READS_TABLE] = vec![Instruction::Raw(vec![0xff])];
    let at = unknown.assemble().raw.expect("the unknown opcode");
    assert_eq!(unknown.refusal().offset, at);

    let mut unknown = sample();
    unknown.functions[LEAF] = vec![Instruction::Raw(vec![0xfc, 0x7f])];
    let at = unknown.assemble().raw.expect("the unknown opcode");
    assert_eq!(unknown.refusal().offset, at);

    let mut beyond = sample();
    beyond.functions[LEAF] = vec![Instruction::Raw(vec![0x10, 0xff, 0xff, 0x03])];
    let at = beyond.assemble().raw.expect("the call");
    assert_eq!(beyond.refusal().offset, at + 1);

    let wrong = Sample {
        wrong_operand: 4,
        ..sample()
    };
    let at = wrong.assemble().first_address.expect("an address");
    assert_eq!(wrong.refusal().offset, at);

    let stray = Sample {
        stray_relocation: true,
        ..sample()
    };
    assert_eq!(stray.refusal().offset, stray.assemble().first_body);

    let unlinked = Sample {
        linking: false,
        ..sample()
    };
    assert_eq!(unlinked.refusal().offset, unlinked.assemble().bytes.len());
}

/// The hash under every key: FNV-1a of 128 bits, whose offset basis is the hash of no bytes. The
/// figure for one byte comes from the specification's formula, by hand: (basis xor 0x61) * prime.
#[test]
fn the_hash_is_fnv_1a_of_128_bits() {
    assert_eq!(
        Fingerprint::of(b""),
        0x6c62_272e_07bb_0142_62b8_2175_6295_c58d
    );
    assert_eq!(
        Fingerprint::of(b"a"),
        0xd228_cb69_6f1a_8caf_7891_2b70_4e4a_8964
    );
}
