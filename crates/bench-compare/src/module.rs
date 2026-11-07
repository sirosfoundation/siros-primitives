//! A reader of the WebAssembly modules that `rustc` links for `wasm32-unknown-unknown` with
//! `-C link-arg=--emit-relocs`: their sections, and every instruction of their functions as a stream
//! with the operands that layout moves taken out.
//!
//! The linker's relocations say which operands are addresses, and of which symbol. `--emit-relocs`
//! adds the custom sections `linking`, `reloc.CODE` and `reloc.DATA`, and writes the zeros of `.bss`
//! as a data segment: the types, the functions, the table, the code and the other segments keep
//! their bytes.
//!
//! The stream of a function holds its bytes with [`Hole`]s where a function index or the symbol of an
//! address stood: [`crate::identity`] fills them with hashes. An opcode outside the table of
//! [`immediates`] is a [`ModuleError`] that names the opcode and its offset.

/// A module holds what this reader does not take.
#[derive(Debug, thiserror::Error)]
#[error("the module at offset {offset:#x}: {reason}")]
pub struct ModuleError {
    /// The offset in the module.
    pub offset: usize,
    /// What it holds there.
    pub reason: String,
}

/// Marks in a stream: what stood there has no bytes that the hash may keep.
mod mark {
    /// A `call` of a function index.
    pub(super) const CALL: u8 = 0xf0;
    /// An address: its distance from a symbol follows.
    pub(super) const ADDRESS: u8 = 0xf1;
    /// The table index of a function.
    pub(super) const TABLE: u8 = 0xf2;
}

/// The relocation types of the linker's conventions that this reader reads. The linker writes the
/// others on operands that the table of [`immediates`] takes by their opcode.
mod relocation {
    pub(super) const FUNCTION_INDEX_LEB: u8 = 0;
    pub(super) const TABLE_INDEX_SLEB: u8 = 1;
    pub(super) const TABLE_INDEX_I32: u8 = 2;
    pub(super) const MEMORY_ADDR_LEB: u8 = 3;
    pub(super) const MEMORY_ADDR_SLEB: u8 = 4;
    pub(super) const MEMORY_ADDR_I32: u8 = 5;
    pub(super) const TYPE_INDEX_LEB: u8 = 6;
    pub(super) const GLOBAL_INDEX_LEB: u8 = 7;
    pub(super) const TABLE_NUMBER_LEB: u8 = 20;
}

/// The symbol flags and kinds of the linker's conventions.
mod symbol {
    pub(super) const KIND_FUNCTION: u8 = 0;
    pub(super) const KIND_DATA: u8 = 1;
    pub(super) const KIND_GLOBAL: u8 = 2;
    pub(super) const KIND_SECTION: u8 = 3;
    pub(super) const KIND_TAG: u8 = 4;
    pub(super) const KIND_TABLE: u8 = 5;
    pub(super) const UNDEFINED: u32 = 0x10;
    pub(super) const EXPLICIT_NAME: u32 = 0x40;
    pub(super) const ABSOLUTE: u32 = 0x200;
}

/// The subsections of the `linking` section that this reader reads.
mod linking {
    pub(super) const VERSION: u32 = 2;
    pub(super) const SEGMENT_INFO: u8 = 5;
    pub(super) const SYMBOL_TABLE: u8 = 8;
}

/// A position in a [`Stream`] that takes a hash.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Hole {
    /// The bytes of the stream before the hole.
    pub(crate) at: usize,
    /// What fills it.
    pub(crate) kind: HoleKind,
}

/// What fills a [`Hole`].
#[derive(Clone, Copy, Debug)]
pub(crate) enum HoleKind {
    /// The hash of a function.
    Call(u32),
    /// The hash of a symbol.
    Data(u32),
}

/// A function's control flow, which decides whether it can return.
#[derive(Clone, Debug)]
pub(crate) enum Control {
    /// `block`.
    Block,
    /// `loop`.
    Loop,
    /// `if`.
    If,
    /// `else`.
    Else,
    /// `end`.
    End,
    /// `br`.
    Br(u32),
    /// `br_if`.
    BrIf(u32),
    /// `br_table`, with every label.
    BrTable(Vec<u32>),
    /// `return`.
    Return,
    /// `unreachable`.
    Unreachable,
    /// `call`.
    Call(u32),
}

/// One function, read: its instructions with what layout moves taken out.
#[derive(Debug, Default)]
pub(crate) struct Stream {
    /// The signature, the locals and the instructions.
    pub(crate) bytes: Vec<u8>,
    /// The holes of `bytes`, in order.
    pub(crate) holes: Vec<Hole>,
    /// Its control flow, in order.
    pub(crate) control: Vec<Control>,
    /// Whether it holds a `call_indirect`, a `ref.func` or another instruction that names a
    /// function or a segment at run time.
    pub(crate) indirect: bool,
}

/// A function that the module imports.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Import<'a> {
    module: &'a [u8],
    name: &'a [u8],
    signature: &'a [u8],
}

impl Import<'_> {
    /// The bytes that identify the import.
    pub(crate) fn identity(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.module.len() + self.name.len() + 16);
        for part in [self.module, self.name, self.signature] {
            bytes.extend_from_slice(&(part.len() as u64).to_le_bytes());
            bytes.extend_from_slice(part);
        }
        bytes
    }
}

/// A data segment: the bytes that the module copies to `address` when it starts.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Segment<'a> {
    /// The address.
    pub(crate) address: u32,
    /// The bytes.
    pub(crate) bytes: &'a [u8],
    /// Where `bytes` starts in the module.
    position: usize,
    /// Whether the name of the segment makes it `.rodata`.
    pub(crate) read_only: bool,
}

/// An entry of the linker's symbol table.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Symbol<'a> {
    /// A definition in a data segment.
    Data(DataSymbol<'a>),
    /// An address that the linker computes, such as `__heap_base`.
    Absolute {
        /// Its name.
        name: &'a str,
        /// The address.
        address: u32,
    },
    /// A function, a global, a table, a section, or data that the module does not define.
    Other,
}

/// A definition in a data segment.
#[derive(Clone, Copy, Debug)]
pub(crate) struct DataSymbol<'a> {
    /// Its name.
    pub(crate) name: &'a str,
    /// The index of its segment in [`Module::segments`].
    pub(crate) segment: usize,
    /// Its address.
    pub(crate) address: u32,
    /// The bytes that it spans in the segment.
    pub(crate) bytes: &'a [u8],
}

impl Symbol<'_> {
    /// The address of the symbol, if it has one.
    fn address(&self) -> Option<u32> {
        match self {
            Symbol::Data(DataSymbol { address, .. }) | Symbol::Absolute { address, .. } => {
                Some(*address)
            }
            Symbol::Other => None,
        }
    }
}

/// What the linker wrote at a place in the code or the data.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Relocated {
    /// An address.
    Address {
        /// The index of the symbol in [`Module::symbols`].
        symbol: u32,
        /// The distance of the address from the symbol.
        addend: i32,
    },
    /// The table index of a function. Only a `call_indirect` can run it, and [`Stream::indirect`]
    /// marks that instruction wherever it stands.
    TableIndex,
}

/// A place in the module that the linker wrote, and what it wrote.
#[derive(Clone, Copy, Debug)]
struct Relocation {
    /// The offset in the module.
    position: usize,
    /// What it wrote.
    relocated: Relocated,
}

/// A module, read.
#[derive(Debug)]
pub(crate) struct Module<'a> {
    /// The functions that the module imports: function indices from 0.
    pub(crate) imports: Vec<Import<'a>>,
    /// The functions that the module defines, whose indices follow the imports.
    pub(crate) streams: Vec<Stream>,
    /// The function at each index of the table.
    pub(crate) table: Vec<Option<u32>>,
    /// The data segments, as the module orders them.
    pub(crate) segments: Vec<Segment<'a>>,
    /// The name of each function that the name section names, by function index.
    pub(crate) names: Vec<(u32, &'a str)>,
    /// The linker's symbol table.
    pub(crate) symbols: Vec<Symbol<'a>>,
    /// The words of the data segments that the linker wrote, by address.
    pub(crate) data_relocations: Vec<(u32, Relocated)>,
}

/// A cursor over the bytes of a module, up to `end`.
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
    end: usize,
}

impl<'a> Reader<'a> {
    fn malformed(&self, what: &str) -> ModuleError {
        ModuleError {
            offset: self.at,
            reason: what.to_owned(),
        }
    }

    fn done(&self) -> bool {
        self.at >= self.end
    }

    fn byte(&mut self) -> Result<u8, ModuleError> {
        if self.at >= self.end {
            return Err(self.malformed("a byte past the end of its section"));
        }
        let byte = self.bytes[self.at];
        self.at += 1;
        Ok(byte)
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], ModuleError> {
        let end = self
            .at
            .checked_add(count)
            .filter(|end| *end <= self.end)
            .ok_or_else(|| self.malformed("bytes past the end of their section"))?;
        let bytes = &self.bytes[self.at..end];
        self.at = end;
        Ok(bytes)
    }

    fn skip(&mut self, count: usize) -> Result<(), ModuleError> {
        self.take(count).map(|_bytes| ())
    }

    /// An unsigned LEB128 number of at most 32 bits.
    fn u32(&mut self) -> Result<u32, ModuleError> {
        let mut value = 0_u32;
        for index in 0..5 {
            let byte = self.byte()?;
            value |= u32::from(byte & 0x7f) << (7 * index);
            if byte & 0x80 == 0 {
                return Ok(value);
            }
        }
        Err(self.malformed("a LEB128 number of more than 32 bits"))
    }

    /// Passes over an unsigned LEB128 number of at most 32 bits.
    fn skip_u32(&mut self) -> Result<(), ModuleError> {
        self.u32().map(|_value| ())
    }

    /// A signed LEB128 number of at most `bits` bits.
    fn signed(&mut self, bits: u32) -> Result<i64, ModuleError> {
        let mut value = 0_i64;
        let mut shift = 0;
        loop {
            let byte = self.byte()?;
            value |= i64::from(byte & 0x7f) << shift;
            shift += 7;
            if byte & 0x80 == 0 {
                if shift < 64 && byte & 0x40 != 0 {
                    value |= -1_i64 << shift;
                }
                return Ok(value);
            }
            if shift >= bits.div_ceil(7) * 7 {
                return Err(self.malformed("a signed LEB128 number that is too long"));
            }
        }
    }

    fn name(&mut self) -> Result<&'a [u8], ModuleError> {
        let length = self.u32()? as usize;
        self.take(length)
    }

    /// Passes over a name.
    fn skip_name(&mut self) -> Result<(), ModuleError> {
        self.name().map(|_name| ())
    }

    /// A name that must be UTF-8.
    fn text(&mut self) -> Result<&'a str, ModuleError> {
        let name = self.name()?;
        str::from_utf8(name).map_err(|error| self.malformed(&format!("a name: {error}")))
    }

    /// The reader of the next `size` bytes, which this reader passes over.
    fn section(&mut self, size: usize) -> Result<Self, ModuleError> {
        let start = self.at;
        let bytes = self.take(size)?;
        Ok(Self {
            bytes: self.bytes,
            at: start,
            end: start + bytes.len(),
        })
    }

    /// The value of a constant expression that holds one `i32.const`.
    fn constant(&mut self) -> Result<u32, ModuleError> {
        if self.byte()? != op::I32_CONST {
            return Err(self.malformed("a constant expression other than i32.const"));
        }
        let value = self.signed(32)? as u32;
        if self.byte()? != op::END {
            return Err(self.malformed("a constant expression of more than one instruction"));
        }
        Ok(value)
    }
}

/// The opcodes that this reader treats apart.
mod op {
    pub(super) const UNREACHABLE: u8 = 0x00;
    pub(super) const BLOCK: u8 = 0x02;
    pub(super) const LOOP: u8 = 0x03;
    pub(super) const IF: u8 = 0x04;
    pub(super) const ELSE: u8 = 0x05;
    pub(super) const END: u8 = 0x0b;
    pub(super) const BR: u8 = 0x0c;
    pub(super) const RETURN: u8 = 0x0f;
    pub(super) const I32_CONST: u8 = 0x41;
    pub(super) const PREFIX_FC: u8 = 0xfc;
}

/// The operands that follow an opcode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Immediates {
    /// None.
    None,
    /// A block type.
    Block,
    /// A label.
    Label,
    /// A list of labels and a default.
    Labels,
    /// A function index.
    Function,
    /// A type index and a table index.
    Indirect,
    /// A list of value types.
    Types,
    /// A local or global index.
    Index,
    /// A table index.
    Table,
    /// An alignment and an offset.
    Memarg,
    /// One zero byte.
    Zero,
    /// A signed 32-bit number.
    I32,
    /// A signed 64-bit number.
    I64,
    /// Four bytes.
    F32,
    /// Eight bytes.
    F64,
    /// A value type.
    Reference,
    /// A function index, taken at run time.
    FunctionReference,
}

/// The operands of the instruction `opcode`, for the instructions of the WebAssembly specification
/// without SIMD; `None` for any other opcode.
const fn immediates(opcode: u8) -> Option<Immediates> {
    Some(match opcode {
        0x00 | 0x01 | 0x05 | 0x0b | 0x0f | 0x1a | 0x1b | 0x45..=0xc4 | 0xd1 => Immediates::None,
        0x02..=0x04 => Immediates::Block,
        0x0c | 0x0d => Immediates::Label,
        0x0e => Immediates::Labels,
        0x10 => Immediates::Function,
        0x11 => Immediates::Indirect,
        0x1c => Immediates::Types,
        0x20..=0x24 => Immediates::Index,
        0x25 | 0x26 => Immediates::Table,
        0x28..=0x3e => Immediates::Memarg,
        0x3f | 0x40 => Immediates::Zero,
        0x41 => Immediates::I32,
        0x42 => Immediates::I64,
        0x43 => Immediates::F32,
        0x44 => Immediates::F64,
        0xd0 => Immediates::Reference,
        0xd2 => Immediates::FunctionReference,
        _ => return None,
    })
}

/// The instructions after the prefix `0xfc`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Prefixed {
    /// No operands: a saturating conversion, or an instruction of the wide-arithmetic proposal.
    Plain,
    /// `memory.copy`.
    MemoryCopy,
    /// `memory.fill`.
    MemoryFill,
    /// An instruction with a segment or table operand, which decides at run time what it reaches.
    Indirect(u8),
}

/// The instruction `0xfc sub`; `None` for an opcode outside the specification and the
/// wide-arithmetic proposal.
const fn prefixed(sub: u32) -> Option<Prefixed> {
    Some(match sub {
        0..=7 | 19..=22 => Prefixed::Plain,
        10 => Prefixed::MemoryCopy,
        11 => Prefixed::MemoryFill,
        // `memory.init` and `table.init`, `table.copy` take two operands.
        8 | 12 | 14 => Prefixed::Indirect(2),
        // `data.drop`, `elem.drop`, `table.grow`, `table.size` and `table.fill` take one.
        9 | 13 | 15 | 16 | 17 => Prefixed::Indirect(1),
        _ => return None,
    })
}

/// The next relocations of the code, which the operands of the functions take in order.
struct CodeRelocations<'r> {
    remaining: core::slice::Iter<'r, Relocation>,
}

impl CodeRelocations<'_> {
    /// What the linker wrote at the operand at `position`, if it wrote there.
    fn take(&mut self, position: usize) -> Option<Relocated> {
        let next = self.remaining.as_slice().first()?;
        if next.position != position {
            return None;
        }
        self.remaining.next().map(|relocation| relocation.relocated)
    }
}

impl Stream {
    fn push_u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn push_hole(&mut self, kind: HoleKind) {
        self.holes.push(Hole {
            at: self.bytes.len(),
            kind,
        });
    }
}

/// The parts of a module that its functions refer to.
#[derive(Default)]
struct Sections<'a> {
    /// The index in the module of the section that the reader takes next.
    index: u32,
    types: Vec<&'a [u8]>,
    imports: Vec<Import<'a>>,
    declared: Vec<u32>,
    code: Option<CodeSection>,
    data: Option<DataSection>,
    table: Vec<Option<u32>>,
    segments: Vec<Segment<'a>>,
    names: Vec<(u32, &'a str)>,
    linked: bool,
    symbols: Vec<Symbol<'a>>,
    code_relocations: Vec<Relocation>,
    data_relocations: Vec<Relocation>,
}

/// The code section: its place, and the place of each body.
struct CodeSection {
    index: u32,
    start: usize,
    bodies: Vec<(usize, usize)>,
}

/// The data section: its place.
struct DataSection {
    index: u32,
    start: usize,
}

impl<'a> Module<'a> {
    /// Reads the module `bytes`, which a build with `-C link-arg=--emit-relocs` linked.
    pub(crate) fn read(bytes: &'a [u8]) -> Result<Self, ModuleError> {
        let mut reader = Reader {
            bytes,
            at: 0,
            end: bytes.len(),
        };
        if reader.take(8)? != b"\0asm\x01\0\0\0" {
            return Err(ModuleError {
                offset: 0,
                reason: "a header other than that of WebAssembly 1".to_owned(),
            });
        }
        let mut sections = Sections::default();
        while !reader.done() {
            let id = reader.byte()?;
            let size = reader.u32()? as usize;
            let section = reader.section(size)?;
            sections.section(id, section)?;
            sections.index += 1;
        }
        sections.finish(bytes)
    }

    /// The defined function at the function index `index`.
    pub(crate) fn stream(&self, index: u32) -> Option<&Stream> {
        (index as usize)
            .checked_sub(self.imports.len())
            .and_then(|defined| self.streams.get(defined))
    }

    /// The number of functions, imported and defined.
    pub(crate) fn functions(&self) -> u32 {
        (self.imports.len() + self.streams.len()) as u32
    }

    /// The relocated words of the data at `address`, over `size` bytes.
    pub(crate) fn relocations_in(&self, address: u32, size: usize) -> &[(u32, Relocated)] {
        let end = u64::from(address) + size as u64;
        let start = self
            .data_relocations
            .partition_point(|(at, _)| *at < address);
        let stop = self
            .data_relocations
            .partition_point(|(at, _)| u64::from(*at) < end);
        &self.data_relocations[start..stop]
    }

    /// Whether `symbol` holds a `core::panic::Location`: the address and the length of a file name
    /// that ends in `.rs`, a line and a column.
    pub(crate) fn is_location(&self, symbol: &DataSymbol<'_>) -> bool {
        const MOST_FILE_BYTES: u32 = 4096;
        const MOST_LINES: u32 = 1 << 24;
        let Ok(words) = <[u8; 16]>::try_from(symbol.bytes) else {
            return false;
        };
        let word = |index: usize| {
            u32::from_le_bytes([
                words[4 * index],
                words[4 * index + 1],
                words[4 * index + 2],
                words[4 * index + 3],
            ])
        };
        let (length, line, column) = (word(1), word(2), word(3));
        if !(1..=MOST_FILE_BYTES).contains(&length)
            || !(1..MOST_LINES).contains(&line)
            || !(1..MOST_LINES).contains(&column)
        {
            return false;
        }
        let [
            (
                at,
                Relocated::Address {
                    symbol: file,
                    addend,
                },
            ),
        ] = self.relocations_in(symbol.address, words.len())
        else {
            return false;
        };
        if *at != symbol.address {
            return false;
        }
        let Some(Symbol::Data(file)) = self.symbols.get(*file as usize) else {
            return false;
        };
        usize::try_from(*addend)
            .ok()
            .and_then(|start| file.bytes.get(start..start.checked_add(length as usize)?))
            .is_some_and(|name| name.ends_with(b".rs"))
    }
}

impl<'a> Sections<'a> {
    /// Takes in the section `id`.
    fn section(&mut self, id: u8, mut section: Reader<'a>) -> Result<(), ModuleError> {
        match id {
            0 => self.custom(&mut section),
            1 => self.types(&mut section),
            2 => self.imports(&mut section),
            3 => self.declared(&mut section),
            9 => self.elements(&mut section),
            10 => self.code(&mut section),
            11 => self.data(&mut section),
            // Tables, memories, globals, exports, the start function and the data count: a
            // `call_indirect`, a `global.get` and a `global.set` keep their indices.
            4..=8 | 12 => Ok(()),
            _ => Err(section.malformed(&format!("a section of the id {id}"))),
        }
    }

    fn custom(&mut self, section: &mut Reader<'a>) -> Result<(), ModuleError> {
        match section.name()? {
            b"name" => self.names(section),
            b"linking" => self.linking(section),
            b"reloc.CODE" => {
                let code = self.code.as_ref().map(|code| (code.index, code.start));
                self.code_relocations = relocations(section, code)?;
                Ok(())
            }
            b"reloc.DATA" => {
                let data = self.data.as_ref().map(|data| (data.index, data.start));
                self.data_relocations = relocations(section, data)?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn names(&mut self, section: &mut Reader<'a>) -> Result<(), ModuleError> {
        while !section.done() {
            let id = section.byte()?;
            let size = section.u32()? as usize;
            let mut part = section.section(size)?;
            if id == 1 {
                for _ in 0..part.u32()? {
                    let index = part.u32()?;
                    let name = part.name()?;
                    if let Ok(name) = str::from_utf8(name) {
                        self.names.push((index, name));
                    }
                }
            }
        }
        Ok(())
    }

    fn linking(&mut self, section: &mut Reader<'a>) -> Result<(), ModuleError> {
        let version = section.u32()?;
        if version != linking::VERSION {
            return Err(section.malformed(&format!("the linking metadata version {version}")));
        }
        while !section.done() {
            let id = section.byte()?;
            let size = section.u32()? as usize;
            let mut part = section.section(size)?;
            match id {
                linking::SEGMENT_INFO => self.segment_info(&mut part)?,
                linking::SYMBOL_TABLE => self.symbol_table(&mut part)?,
                _ => {}
            }
        }
        self.linked = true;
        Ok(())
    }

    fn segment_info(&mut self, part: &mut Reader<'a>) -> Result<(), ModuleError> {
        let count = part.u32()? as usize;
        if count != self.segments.len() {
            return Err(part.malformed(&format!(
                "information on {count} segments of {}",
                self.segments.len()
            )));
        }
        for index in 0..count {
            let name = part.text()?;
            self.segments[index].read_only = name.starts_with(".rodata");
            // The alignment and the flags.
            for _ in 0..2 {
                part.skip_u32()?;
            }
        }
        Ok(())
    }

    fn symbol_table(&mut self, part: &mut Reader<'a>) -> Result<(), ModuleError> {
        for _ in 0..part.u32()? {
            let kind = part.byte()?;
            let flags = part.u32()?;
            let symbol = match kind {
                symbol::KIND_FUNCTION
                | symbol::KIND_GLOBAL
                | symbol::KIND_TAG
                | symbol::KIND_TABLE => {
                    part.skip_u32()?;
                    if flags & symbol::UNDEFINED == 0 || flags & symbol::EXPLICIT_NAME != 0 {
                        part.skip_name()?;
                    }
                    Symbol::Other
                }
                symbol::KIND_SECTION => {
                    part.skip_u32()?;
                    Symbol::Other
                }
                symbol::KIND_DATA => self.data_symbol(part, flags)?,
                _ => return Err(part.malformed(&format!("a symbol of the kind {kind}"))),
            };
            self.symbols.push(symbol);
        }
        Ok(())
    }

    /// The fields of a data symbol with the flags `flags`.
    fn data_symbol(&self, part: &mut Reader<'a>, flags: u32) -> Result<Symbol<'a>, ModuleError> {
        let name = part.text()?;
        if flags & symbol::UNDEFINED != 0 {
            return Ok(Symbol::Other);
        }
        let segment = part.u32()? as usize;
        let offset = part.u32()?;
        let size = part.u32()? as usize;
        if flags & symbol::ABSOLUTE != 0 {
            return Ok(Symbol::Absolute {
                name,
                address: offset,
            });
        }
        let found = self.segments.get(segment).ok_or_else(|| {
            part.malformed(&format!("the symbol {name} in the segment {segment}"))
        })?;
        let start = offset as usize;
        let bytes = start
            .checked_add(size)
            .and_then(|end| found.bytes.get(start..end))
            .ok_or_else(|| part.malformed(&format!("the symbol {name} past its segment")))?;
        Ok(Symbol::Data(DataSymbol {
            name,
            segment,
            address: found.address.wrapping_add(offset),
            bytes,
        }))
    }

    fn types(&mut self, section: &mut Reader<'a>) -> Result<(), ModuleError> {
        for _ in 0..section.u32()? {
            let start = section.at;
            if section.byte()? != 0x60 {
                return Err(section.malformed("a type other than a function type"));
            }
            for _ in 0..2 {
                let count = section.u32()? as usize;
                section.skip(count)?;
            }
            self.types.push(&section.bytes[start..section.at]);
        }
        Ok(())
    }

    fn signature(&self, index: u32, section: &Reader<'a>) -> Result<&'a [u8], ModuleError> {
        self.types
            .get(index as usize)
            .copied()
            .ok_or_else(|| section.malformed(&format!("the type {index}")))
    }

    fn imports(&mut self, section: &mut Reader<'a>) -> Result<(), ModuleError> {
        for _ in 0..section.u32()? {
            let module = section.name()?;
            let name = section.name()?;
            if section.byte()? != 0 {
                return Err(section.malformed("an import other than a function"));
            }
            let index = section.u32()?;
            let signature = self.signature(index, section)?;
            self.imports.push(Import {
                module,
                name,
                signature,
            });
        }
        Ok(())
    }

    fn declared(&mut self, section: &mut Reader<'a>) -> Result<(), ModuleError> {
        for _ in 0..section.u32()? {
            self.declared.push(section.u32()?);
        }
        Ok(())
    }

    fn elements(&mut self, section: &mut Reader<'a>) -> Result<(), ModuleError> {
        for _ in 0..section.u32()? {
            if section.u32()? != 0 {
                return Err(section.malformed("an element segment other than active, of functions"));
            }
            let offset = section.constant()? as usize;
            let count = section.u32()? as usize;
            if self.table.len() < offset + count {
                self.table.resize(offset + count, None);
            }
            for slot in offset..offset + count {
                self.table[slot] = Some(section.u32()?);
            }
        }
        Ok(())
    }

    fn code(&mut self, section: &mut Reader<'a>) -> Result<(), ModuleError> {
        let start = section.at;
        let mut bodies = Vec::new();
        for _ in 0..section.u32()? {
            let size = section.u32()? as usize;
            let body = section.section(size)?;
            bodies.push((body.at, body.end));
        }
        self.code = Some(CodeSection {
            index: self.index,
            start,
            bodies,
        });
        Ok(())
    }

    fn data(&mut self, section: &mut Reader<'a>) -> Result<(), ModuleError> {
        self.data = Some(DataSection {
            index: self.index,
            start: section.at,
        });
        for _ in 0..section.u32()? {
            if section.u32()? != 0 {
                return Err(section.malformed("a data segment other than active"));
            }
            let address = section.constant()?;
            let length = section.u32()? as usize;
            let position = section.at;
            let bytes = section.take(length)?;
            self.segments.push(Segment {
                address,
                bytes,
                position,
                read_only: false,
            });
        }
        Ok(())
    }

    /// The address of a relocated value: of `symbol`, at the distance `addend`.
    fn address(&self, symbol: u32, addend: i32, position: usize) -> Result<u32, ModuleError> {
        self.symbols
            .get(symbol as usize)
            .and_then(Symbol::address)
            .map(|address| address.wrapping_add_signed(addend))
            .ok_or_else(|| ModuleError {
                offset: position,
                reason: format!("a relocation against the symbol {symbol}, which holds no address"),
            })
    }

    /// The words of the data that the linker wrote, by address. Each must hold what it says.
    fn data_words(&self, bytes: &[u8]) -> Result<Vec<(u32, Relocated)>, ModuleError> {
        let mut words = Vec::with_capacity(self.data_relocations.len());
        for relocation in &self.data_relocations {
            let position = relocation.position;
            let segment = self
                .segments
                .iter()
                .find(|segment| {
                    segment.position <= position
                        && position + 4 <= segment.position + segment.bytes.len()
                })
                .ok_or_else(|| ModuleError {
                    offset: position,
                    reason: "a relocation outside the data segments".to_owned(),
                })?;
            if let Relocated::Address { symbol, addend } = relocation.relocated {
                let expected = self.address(symbol, addend, position)?;
                let word = bytes
                    .get(position..position + 4)
                    .and_then(|word| <[u8; 4]>::try_from(word).ok())
                    .map(u32::from_le_bytes);
                if word != Some(expected) {
                    return Err(ModuleError {
                        offset: position,
                        reason: format!(
                            "a word of the data that holds {word:?}, where its relocation says \
                             {expected}"
                        ),
                    });
                }
            }
            words.push((
                segment.address + (position - segment.position) as u32,
                relocation.relocated,
            ));
        }
        words.sort_unstable_by_key(|(address, _)| *address);
        Ok(words)
    }

    /// Reads every function.
    fn finish(self, bytes: &'a [u8]) -> Result<Module<'a>, ModuleError> {
        if !self.linked {
            return Err(ModuleError {
                offset: bytes.len(),
                reason: "no linking section: a build without -C link-arg=--emit-relocs".to_owned(),
            });
        }
        let code = self.code.as_ref().ok_or_else(|| ModuleError {
            offset: bytes.len(),
            reason: "no code section".to_owned(),
        })?;
        if code.bodies.len() != self.declared.len() {
            return Err(ModuleError {
                offset: bytes.len(),
                reason: format!(
                    "{} functions with {} bodies",
                    self.declared.len(),
                    code.bodies.len()
                ),
            });
        }
        let mut relocations = CodeRelocations {
            remaining: self.code_relocations.iter(),
        };
        let streams = code
            .bodies
            .iter()
            .zip(&self.declared)
            .map(|(&(at, end), &signature)| {
                let mut reader = Reader { bytes, at, end };
                let signature = self.signature(signature, &reader)?;
                self.stream(&mut reader, signature, &mut relocations)
            })
            .collect::<Result<Vec<_>, ModuleError>>()?;
        if let Some(left) = relocations.remaining.as_slice().first() {
            return Err(ModuleError {
                offset: left.position,
                reason: "a relocation on no operand of i32.const or of a memory access".to_owned(),
            });
        }
        let data_relocations = self.data_words(bytes)?;
        Ok(Module {
            imports: self.imports,
            streams,
            table: self.table,
            segments: self.segments,
            names: self.names,
            symbols: self.symbols,
            data_relocations,
        })
    }

    /// The stream of the function whose body `reader` spans.
    fn stream(
        &self,
        reader: &mut Reader<'a>,
        signature: &[u8],
        relocations: &mut CodeRelocations<'_>,
    ) -> Result<Stream, ModuleError> {
        let mut stream = Stream::default();
        stream.bytes.extend_from_slice(signature);
        for _ in 0..reader.u32()? {
            let count = reader.u32()?;
            let kind = reader.byte()?;
            stream.push_u32(count);
            stream.bytes.push(kind);
        }
        let mut depth = 1_u32;
        while depth > 0 {
            let offset = reader.at;
            let opcode = reader.byte()?;
            stream.bytes.push(opcode);
            match opcode {
                op::BLOCK | op::LOOP | op::IF => depth += 1,
                op::END => depth -= 1,
                _ => {}
            }
            if opcode == op::PREFIX_FC {
                Self::prefixed(reader, &mut stream, offset)?;
                continue;
            }
            let kinds = immediates(opcode).ok_or(ModuleError {
                offset,
                reason: format!("the unknown opcode {opcode:#04x}"),
            })?;
            self.operands(opcode, kinds, reader, &mut stream, relocations)?;
            if let Some(control) = plain_control(opcode) {
                stream.control.push(control);
            }
        }
        if !reader.done() {
            return Err(reader.malformed("bytes after the last end of a function"));
        }
        Ok(stream)
    }

    /// The instruction after the prefix `0xfc` at `offset`.
    fn prefixed(
        reader: &mut Reader<'a>,
        stream: &mut Stream,
        offset: usize,
    ) -> Result<(), ModuleError> {
        let sub = reader.u32()?;
        stream.push_u32(sub);
        match prefixed(sub) {
            Some(Prefixed::Plain) => {}
            Some(Prefixed::MemoryCopy) => {
                reader.skip(2)?;
            }
            Some(Prefixed::MemoryFill) => {
                reader.skip(1)?;
            }
            Some(Prefixed::Indirect(operands)) => {
                stream.indirect = true;
                for _ in 0..operands {
                    stream.push_u32(reader.u32()?);
                }
            }
            None => {
                return Err(ModuleError {
                    offset,
                    reason: format!("the unknown opcode 0xfc {sub:#x}"),
                });
            }
        }
        Ok(())
    }

    /// The 32-bit operand `value` at `position`: the number, or what the linker wrote there.
    fn push_value(
        &self,
        value: u32,
        position: usize,
        stream: &mut Stream,
        relocations: &mut CodeRelocations<'_>,
    ) -> Result<(), ModuleError> {
        match relocations.take(position) {
            None => stream.push_u32(value),
            Some(Relocated::TableIndex) => stream.bytes.push(mark::TABLE),
            Some(Relocated::Address { symbol, addend }) => {
                let expected = self.address(symbol, addend, position)?;
                if value != expected {
                    return Err(ModuleError {
                        offset: position,
                        reason: format!("an operand {value}, where its relocation says {expected}"),
                    });
                }
                stream.bytes.push(mark::ADDRESS);
                stream.bytes.extend_from_slice(&addend.to_le_bytes());
                stream.push_hole(HoleKind::Data(symbol));
            }
        }
        Ok(())
    }

    /// The operands of `opcode`, of the kinds `kinds`, into `stream`.
    fn operands(
        &self,
        opcode: u8,
        kinds: Immediates,
        reader: &mut Reader<'a>,
        stream: &mut Stream,
        relocations: &mut CodeRelocations<'_>,
    ) -> Result<(), ModuleError> {
        match kinds {
            Immediates::None => {}
            Immediates::Block => self.block_type(reader, stream)?,
            Immediates::Label => {
                let label = reader.u32()?;
                stream.push_u32(label);
                stream.control.push(if opcode == op::BR {
                    Control::Br(label)
                } else {
                    Control::BrIf(label)
                });
            }
            Immediates::Labels => {
                let count = reader.u32()?;
                stream.push_u32(count);
                let mut labels = Vec::with_capacity(count as usize + 1);
                for _ in 0..=count {
                    let label = reader.u32()?;
                    stream.push_u32(label);
                    labels.push(label);
                }
                stream.control.push(Control::BrTable(labels));
            }
            Immediates::Function => {
                let callee = reader.u32()?;
                stream.bytes.push(mark::CALL);
                stream.push_hole(HoleKind::Call(callee));
                stream.control.push(Control::Call(callee));
            }
            Immediates::Indirect => {
                stream.indirect = true;
                let index = reader.u32()?;
                let signature = self.signature(index, reader)?;
                stream.bytes.extend_from_slice(signature);
                stream.push_u32(reader.u32()?);
            }
            Immediates::Types => {
                let count = reader.u32()? as usize;
                let types = reader.take(count)?;
                stream.bytes.extend_from_slice(types);
            }
            Immediates::Index | Immediates::Table => stream.push_u32(reader.u32()?),
            Immediates::Memarg => {
                let align = reader.u32()?;
                if align >= 0x40 {
                    return Err(reader.malformed("a memory operand of another memory"));
                }
                stream.push_u32(align);
                let position = reader.at;
                let offset = reader.u32()?;
                self.push_value(offset, position, stream, relocations)?;
            }
            Immediates::Zero => {
                if reader.byte()? != 0 {
                    return Err(reader.malformed("an instruction on another memory"));
                }
            }
            Immediates::I32 => {
                let position = reader.at;
                let value = reader.signed(32)? as u32;
                self.push_value(value, position, stream, relocations)?;
            }
            Immediates::I64 => {
                let value = reader.signed(64)?;
                stream.bytes.extend_from_slice(&value.to_le_bytes());
            }
            Immediates::F32 => stream.bytes.extend_from_slice(reader.take(4)?),
            Immediates::F64 => stream.bytes.extend_from_slice(reader.take(8)?),
            Immediates::Reference => stream.bytes.push(reader.byte()?),
            Immediates::FunctionReference => {
                stream.indirect = true;
                stream.bytes.push(mark::CALL);
                stream.push_hole(HoleKind::Call(reader.u32()?));
            }
        }
        Ok(())
    }

    /// A block type: empty, one value type, or a type index, which a new type moves.
    fn block_type(&self, reader: &mut Reader<'a>, stream: &mut Stream) -> Result<(), ModuleError> {
        let first = reader.bytes.get(reader.at).copied().unwrap_or(0);
        // A value type or the empty type is one negative byte.
        if first & 0xc0 == 0x40 {
            stream.bytes.push(reader.byte()?);
            return Ok(());
        }
        let index = reader.signed(33)?;
        let index = u32::try_from(index)
            .map_err(|_error| reader.malformed(&format!("the block type {index}")))?;
        let signature = self.signature(index, reader)?;
        stream.bytes.extend_from_slice(signature);
        Ok(())
    }
}

/// The relocations of a `reloc.` section whose target is the section `target`: its index, and where
/// its contents start. The kept relocations hold addresses and table indices, by place.
fn relocations(
    section: &mut Reader<'_>,
    target: Option<(u32, usize)>,
) -> Result<Vec<Relocation>, ModuleError> {
    let index = section.u32()?;
    let start = target
        .filter(|(expected, _)| *expected == index)
        .map(|(_, start)| start)
        .ok_or_else(|| section.malformed(&format!("relocations of the section {index}")))?;
    let count = section.u32()?;
    let mut kept = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let kind = section.byte()?;
        let offset = section.u32()? as usize;
        let symbol = section.u32()?;
        let relocated = match kind {
            relocation::FUNCTION_INDEX_LEB
            | relocation::TYPE_INDEX_LEB
            | relocation::GLOBAL_INDEX_LEB
            | relocation::TABLE_NUMBER_LEB => None,
            relocation::TABLE_INDEX_SLEB | relocation::TABLE_INDEX_I32 => {
                Some(Relocated::TableIndex)
            }
            relocation::MEMORY_ADDR_LEB
            | relocation::MEMORY_ADDR_SLEB
            | relocation::MEMORY_ADDR_I32 => {
                let addend = i32::try_from(section.signed(32)?)
                    .map_err(|error| section.malformed(&format!("an addend: {error}")))?;
                Some(Relocated::Address { symbol, addend })
            }
            _ => return Err(section.malformed(&format!("a relocation of the type {kind}"))),
        };
        if let Some(relocated) = relocated {
            kept.push(Relocation {
                position: start + offset,
                relocated,
            });
        }
    }
    Ok(kept)
}

/// The control of the instruction `opcode` without operands.
const fn plain_control(opcode: u8) -> Option<Control> {
    Some(match opcode {
        op::UNREACHABLE => Control::Unreachable,
        op::BLOCK => Control::Block,
        op::LOOP => Control::Loop,
        op::IF => Control::If,
        op::ELSE => Control::Else,
        op::END => Control::End,
        op::RETURN => Control::Return,
        _ => return None,
    })
}

/// The bytes of the custom section `name` of the module `bytes`, if it has one.
pub(crate) fn custom_section<'a>(
    bytes: &'a [u8],
    name: &[u8],
) -> Result<Option<&'a [u8]>, ModuleError> {
    let mut reader = Reader {
        bytes,
        at: 0,
        end: bytes.len(),
    };
    if reader.take(8)? != b"\0asm\x01\0\0\0" {
        return Err(reader.malformed("a header other than that of WebAssembly 1"));
    }
    while !reader.done() {
        let id = reader.byte()?;
        let size = reader.u32()? as usize;
        let mut section = reader.section(size)?;
        if id == 0 && section.name()? == name {
            return Ok(Some(&bytes[section.at..section.end]));
        }
    }
    Ok(None)
}

/// The features that `section`, a `target_features` section, says a module uses: those it marks
/// with `+`, sorted.
pub(crate) fn used_features(section: &[u8]) -> Result<Vec<String>, ModuleError> {
    let mut reader = Reader {
        bytes: section,
        at: 0,
        end: section.len(),
    };
    let count = reader.u32()?;
    let mut features = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let prefix = reader.byte()?;
        let name = reader.name()?;
        let name = str::from_utf8(name)
            .map_err(|error| reader.malformed(&format!("a feature name: {error}")))?;
        if prefix == b'+' {
            features.push(name.to_owned());
        }
    }
    features.sort_unstable();
    Ok(features)
}
