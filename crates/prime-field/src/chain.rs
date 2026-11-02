//! Exponentiation by a public exponent fixed by the modulus, through an addition chain computed
//! at compile time.
//!
//! The chain follows McLoughlin's `addchain` hybrid: the exponent splits into runs of ones longer
//! than the window and into sliding windows of odd values below `2^w`. The run lengths get an
//! addition sequence; a step `c = a + b` of it costs `b` squarings and one multiplication, since
//! `x^(2^c - 1) = (x^(2^a - 1))^(2^b) · x^(2^b - 1)`, so the sequence takes an exact pair when
//! one exists and else the cheaper of a multiple of the next value and a halving. The window
//! values get a Bos–Coster sequence (approximation, halving, delta) at one multiplication per
//! step. The terms then combine from the top: squarings over the gap to the next term, then one
//! multiplication by its dictionary entry. Every window width from 1 to [`MAX_WINDOW`] is tried,
//! with every long run as a run term and with a run term at the top only, and the binary method
//! is the fallback that always fits; the program with the fewest squarings and multiplications
//! wins.
//!
//! A [`Program`] is a straight-line list of steps over a register file. The steps and their
//! indices depend on the exponent alone, so the runtime loop takes one path for every base.

#![expect(
    clippy::indexing_slicing,
    reason = "every index is a loop counter below a checked length, checked before use, or masked \
              to the register file"
)]

use crate::field::{Fp, Modulus};
use crate::kernel::{bit, bit_len, limb, set};
use crate::params::{Limbs, Shape};
use crate::width::Width;

/// The widest window of odd values: the dictionary holds odd values below `2^MAX_WINDOW`.
const MAX_WINDOW: usize = 6;
/// The registers of the runtime loop, a power of two: the base in register `0`, then the
/// dictionary entries. A register index is masked to the file.
const REGISTERS: usize = 32;
/// The rows of program storage, each one limb array of the modulus: one step per limb, so the
/// capacity scales with the width. The binary method needs at most `64 L + 1` steps, which fits
/// `ROWS L` for every `L`.
const ROWS: usize = 96;
/// The most nodes of one addition sequence.
const NODES: usize = 128;
/// The most distinct run lengths the dictionary admits; a longer run beyond that count splits
/// into windows.
const RUN_LENGTHS: usize = 32;

/// A register of the runtime loop, below [`REGISTERS`].
type Register = u8;

/// A dictionary step: `regs[dst] = regs[src]^(2^squarings) · regs[mul]`, without the product
/// when `mul` is `None`.
#[derive(Clone, Copy)]
struct Build {
    squarings: u32,
    src: Register,
    mul: Option<Register>,
    dst: Register,
}

/// A main step: `acc = acc^(2^squarings) · regs[mul]`, without the product when `mul` is
/// `None`.
#[derive(Clone, Copy)]
struct Climb {
    squarings: u32,
    mul: Option<Register>,
}

/// The byte of a packed step that holds no register: every register is below [`REGISTERS`].
const NO_REGISTER: u64 = 0xff;

/// A step packed into one limb: the squarings in the low 32 bits, then the bytes of `src`, `mul`
/// and `dst`.
const fn pack(squarings: u32, src: Register, mul: Option<Register>, dst: Register) -> u64 {
    let mul = match mul {
        Some(register) => register as u64,
        None => NO_REGISTER,
    };
    squarings as u64 | ((src as u64) << 32) | (mul << 40) | ((dst as u64) << 48)
}

/// The register in byte `byte` of a packed step.
#[inline(always)]
const fn unpack(word: u64, byte: u32) -> Option<Register> {
    let value = (word >> (8 * byte)) & 0xff;
    if value == NO_REGISTER {
        None
    } else {
        Some(value as Register)
    }
}

/// The steps of one exponentiation, and their cost: the dictionary steps, then the main steps
/// from the accumulator's first value, the register `first`.
pub(crate) struct Program<M: Modulus> {
    words: [Limbs<M>; ROWS],
    len: usize,
    /// The dictionary steps at the front.
    dictionary: usize,
    /// The register the accumulator starts from; `None` for the exponent `0`, where it is `1`.
    first: Option<Register>,
    /// The registers in use.
    registers: usize,
    /// The squarings of a run.
    pub(crate) squarings: usize,
    /// The multiplications of a run.
    pub(crate) multiplications: usize,
}

impl<M: Modulus> Program<M> {
    /// The program without steps: the accumulator stays `1`.
    pub(crate) const EMPTY: Self = Self {
        words: [const { M::Width::ZERO }; ROWS],
        len: 0,
        dictionary: 0,
        first: None,
        registers: 1,
        squarings: 0,
        multiplications: 0,
    };

    /// The steps the storage holds.
    const CAPACITY: usize = ROWS * Shape::<M>::LIMBS;

    /// The shortest program for `exponent` among the window widths and the binary method.
    pub(crate) const fn for_exponent(exponent: &[u64]) -> Self {
        let mut best = Self::binary(exponent);
        let mut window = 1;
        while window <= MAX_WINDOW {
            let mut runs = 0;
            while runs < 2 {
                if let Some(candidate) = Self::hybrid(exponent, window, runs == 1)
                    && candidate.cost() < best.cost()
                {
                    best = candidate;
                }
                runs += 1;
            }
            window += 1;
        }
        best
    }

    /// The squarings and multiplications of a run.
    const fn cost(&self) -> usize {
        self.squarings + self.multiplications
    }

    /// The packed step at `index`.
    #[inline(always)]
    const fn word(&self, index: usize) -> u64 {
        let row = index / Shape::<M>::LIMBS;
        if row < ROWS {
            limb(self.words[row].as_slice(), index % Shape::<M>::LIMBS)
        } else {
            0
        }
    }

    /// The dictionary step at `index`, below [`Self::dictionary`].
    #[inline(always)]
    const fn build(&self, index: usize) -> Build {
        let word = self.word(index);
        Build {
            squarings: word as u32,
            src: (word >> 32) as Register,
            mul: unpack(word, 5),
            dst: (word >> 48) as Register,
        }
    }

    /// The main step at `index`, at or above [`Self::dictionary`].
    #[inline(always)]
    const fn climb(&self, index: usize) -> Climb {
        let word = self.word(index);
        Climb {
            squarings: word as u32,
            mul: unpack(word, 5),
        }
    }

    /// Appends a packed step of `squarings` squarings and the product by `mul`; `false` when the
    /// storage is full.
    const fn push(&mut self, word: u64, squarings: u32, mul: Option<Register>) -> bool {
        let row = self.len / Shape::<M>::LIMBS;
        if self.len >= Self::CAPACITY || row >= ROWS {
            return false;
        }
        set(
            self.words[row].as_mut_slice(),
            self.len % Shape::<M>::LIMBS,
            word,
        );
        self.len += 1;
        self.squarings += squarings as usize;
        if mul.is_some() {
            self.multiplications += 1;
        }
        true
    }

    /// Appends a dictionary step; `false` when the storage is full.
    const fn push_build(&mut self, step: Build) -> bool {
        self.push(
            pack(step.squarings, step.src, step.mul, step.dst),
            step.squarings,
            step.mul,
        )
    }

    /// Appends a main step; `false` when the storage is full.
    const fn push_climb(&mut self, step: Climb) -> bool {
        self.push(
            pack(step.squarings, 0, step.mul, 0),
            step.squarings,
            step.mul,
        )
    }

    /// A fresh register, or `None` when every one is in use.
    const fn allocate(&mut self) -> Option<Register> {
        if self.registers >= REGISTERS {
            return None;
        }
        let register = self.registers as Register;
        self.registers += 1;
        Some(register)
    }

    /// Square-and-multiply: the base in register `0`, one step per set bit below the top. The
    /// storage holds `64 L + 1` steps, the most an exponent of `L` limbs needs.
    const fn binary(exponent: &[u64]) -> Self {
        let mut program = Self::EMPTY;
        let mut index = bit_len(exponent);
        let mut gap = 0;
        while index > 0 {
            index -= 1;
            if bit(exponent, index) {
                if program.first.is_none() {
                    program.first = Some(0);
                } else {
                    assert!(
                        program.push_climb(Climb {
                            squarings: gap,
                            mul: Some(0),
                        }),
                        "the program storage holds every set bit of the exponent"
                    );
                }
                gap = 0;
            }
            gap += 1;
        }
        if program.first.is_some() && gap > 1 {
            assert!(
                program.push_climb(Climb {
                    squarings: gap - 1,
                    mul: None,
                }),
                "the program storage holds the final squarings"
            );
        }
        program
    }

    /// The hybrid chain for the window width `window`, with every long run a run term or only a
    /// run at the top, or `None` when it exceeds the registers or the storage.
    const fn hybrid(exponent: &[u64], window: usize, top_run_only: bool) -> Option<Self> {
        // Pass one: the dictionary.
        let mut dictionary = Dictionary::EMPTY;
        let mut scan = Scan::new(exponent, window, top_run_only);
        while let Some(term) = scan.next() {
            match term.entry {
                Entry::Run(length) => dictionary.add_run(length),
                Entry::Window(value) => dictionary.add_window(value),
            }
        }
        // Every window value `2^k - 1` is the run of length `k`.
        let mut length = 2;
        while length <= MAX_WINDOW as u32 {
            if dictionary.has_window((1 << length) - 1) {
                dictionary.add_run(length);
            }
            length += 1;
        }
        let Some(runs) = Sequence::through(&dictionary.runs, &[1], Cost::Runs) else {
            return None;
        };
        // The window sequence starts from the base and from the runs the dictionary holds.
        let mut window_base = Targets::EMPTY;
        window_base.add(1);
        let mut index = 0;
        while index < runs.len {
            let length = runs.steps[index].sum;
            if length <= MAX_WINDOW as u32 {
                window_base.add((1 << length) - 1);
            }
            index += 1;
        }
        let mut window_targets = Targets::EMPTY;
        let mut value = 3;
        while value < (1 << MAX_WINDOW) {
            if dictionary.has_window(value) && !power_of_two(value + 1) {
                window_targets.add(value);
            }
            value += 2;
        }
        let Some(windows) =
            Sequence::through(&window_targets, window_base.as_slice(), Cost::Windows)
        else {
            return None;
        };
        // The dictionary steps: the runs, then the windows.
        let mut program = Self::EMPTY;
        let mut map = Map::EMPTY;
        let mut index = 0;
        while index < runs.len {
            let Addition { sum, a, b } = runs.steps[index];
            let (Some(register), Some(src), Some(mul)) = (
                program.allocate(),
                map.register(Entry::Run(a)),
                map.register(Entry::Run(b)),
            ) else {
                return None;
            };
            map.add(Entry::Run(sum), register);
            if !program.push_build(Build {
                squarings: b,
                src,
                mul: Some(mul),
                dst: register,
            }) {
                return None;
            }
            index += 1;
        }
        let mut index = 0;
        while index < windows.len {
            let Addition { sum, a, b } = windows.steps[index];
            let (Some(register), Some(src), Some(mul)) = (
                program.allocate(),
                map.register(Entry::Window(a)),
                map.register(Entry::Window(b)),
            ) else {
                return None;
            };
            map.add(Entry::Window(sum), register);
            let step = if a == b {
                Build {
                    squarings: 1,
                    src,
                    mul: None,
                    dst: register,
                }
            } else {
                Build {
                    squarings: 0,
                    src,
                    mul: Some(mul),
                    dst: register,
                }
            };
            if !program.push_build(step) {
                return None;
            }
            index += 1;
        }
        program.dictionary = program.len;
        // Pass two: the terms, from the top.
        let mut scan = Scan::new(exponent, window, top_run_only);
        let mut previous: Option<usize> = None;
        while let Some(term) = scan.next() {
            let Some(entry) = map.register(term.entry) else {
                return None;
            };
            match previous {
                None => program.first = Some(entry),
                Some(shift) => {
                    if !program.push_climb(Climb {
                        squarings: (shift - term.shift) as u32,
                        mul: Some(entry),
                    }) {
                        return None;
                    }
                }
            }
            previous = Some(term.shift);
        }
        if let Some(shift) = previous
            && shift > 0
            && !program.push_climb(Climb {
                squarings: shift as u32,
                mul: None,
            })
        {
            return None;
        }
        Some(program)
    }
}

/// Whether `value` is a power of two.
const fn power_of_two(value: u32) -> bool {
    value != 0 && value & (value - 1) == 0
}

/// A dictionary entry: a run of `k` ones, `x^(2^k - 1)`, or an odd window value `v`, `x^v`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Entry {
    Run(u32),
    Window(u32),
}

impl Entry {
    const fn equals(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Self::Run(a), Self::Run(b)) | (Self::Window(a), Self::Window(b)) if a == b
        )
    }
}

/// One term of the exponent: `entry · 2^shift`.
struct Term {
    entry: Entry,
    shift: usize,
}

/// A set of distinct values.
struct Targets {
    values: [u32; NODES],
    len: usize,
}

impl Targets {
    const EMPTY: Self = Self {
        values: [0; NODES],
        len: 0,
    };

    const fn contains(&self, value: u32) -> bool {
        let mut index = 0;
        while index < self.len {
            if self.values[index] == value {
                return true;
            }
            index += 1;
        }
        false
    }

    /// Adds `value` once; a full set drops it.
    const fn add(&mut self, value: u32) {
        if !self.contains(value) && self.len < NODES {
            self.values[self.len] = value;
            self.len += 1;
        }
    }

    const fn as_slice(&self) -> &[u32] {
        self.values.split_at(self.len).0
    }
}

/// The dictionary of one window width: the run lengths and the odd window values.
struct Dictionary {
    runs: Targets,
    /// Bit `v` set for each odd window value `v`.
    windows: u64,
}

impl Dictionary {
    const EMPTY: Self = Self {
        runs: Targets::EMPTY,
        windows: 0,
    };

    const fn add_run(&mut self, length: u32) {
        self.runs.add(length);
    }

    const fn add_window(&mut self, value: u32) {
        self.windows |= 1 << value;
    }

    const fn has_window(&self, value: u32) -> bool {
        self.windows & (1 << value) != 0
    }
}

/// The scan of an exponent into terms, from the top bit down. A run of ones longer than the
/// window is a run term while the dictionary admits its length, or only at the top when
/// `top_run_only`, since the squarings that build the top run's power are the squarings over its
/// bits, where a lower run's power costs its own. Every other set bit starts a window of at most
/// `window` bits that stops before the next run term, trimmed to an odd value. Two scans of one
/// exponent yield the same terms.
struct Scan<'a> {
    exponent: &'a [u64],
    window: usize,
    top_run_only: bool,
    /// The run lengths admitted so far.
    runs: [u32; RUN_LENGTHS],
    runs_len: usize,
    /// The bits below `index` are unread.
    index: usize,
}

impl<'a> Scan<'a> {
    const fn new(exponent: &'a [u64], window: usize, top_run_only: bool) -> Self {
        Self {
            exponent,
            window,
            top_run_only,
            runs: [0; RUN_LENGTHS],
            runs_len: 0,
            index: bit_len(exponent),
        }
    }

    /// The ones from bit `index` downward.
    const fn run_length(&self, index: usize) -> usize {
        let mut length = 0;
        while index >= length && bit(self.exponent, index - length) {
            length += 1;
        }
        length
    }

    /// Whether the dictionary holds the run length, or has room for it.
    const fn admits(&self, length: usize) -> bool {
        let mut index = 0;
        while index < self.runs_len {
            if self.runs[index] as usize == length {
                return true;
            }
            index += 1;
        }
        self.runs_len < RUN_LENGTHS
    }

    /// Records a run length that [`Self::admits`].
    const fn admit(&mut self, length: usize) {
        let mut index = 0;
        while index < self.runs_len {
            if self.runs[index] as usize == length {
                return;
            }
            index += 1;
        }
        if self.runs_len < RUN_LENGTHS {
            self.runs[self.runs_len] = length as u32;
            self.runs_len += 1;
        }
    }

    /// Whether a run term starts at bit `index`.
    const fn run_starts(&self, index: usize) -> bool {
        if !bit(self.exponent, index) {
            return false;
        }
        if self.top_run_only && index + 1 != bit_len(self.exponent) {
            return false;
        }
        let length = self.run_length(index);
        length > self.window && self.admits(length)
    }

    const fn next(&mut self) -> Option<Term> {
        while self.index > 0 && !bit(self.exponent, self.index - 1) {
            self.index -= 1;
        }
        if self.index == 0 {
            return None;
        }
        let top = self.index - 1;
        if self.run_starts(top) {
            let length = self.run_length(top);
            self.admit(length);
            self.index -= length;
            return Some(Term {
                entry: Entry::Run(length as u32),
                shift: self.index,
            });
        }
        let mut width = 1;
        while width < self.window && width <= top && !self.run_starts(top - width) {
            width += 1;
        }
        let mut value: u32 = 0;
        let mut offset = 0;
        while offset < width {
            value = (value << 1) | bit(self.exponent, top - offset) as u32;
            offset += 1;
        }
        let trailing = value.trailing_zeros() as usize;
        self.index -= width - trailing;
        Some(Term {
            entry: Entry::Window(value >> trailing),
            shift: self.index,
        })
    }
}

/// A step of an addition sequence: `sum = a + b` with `a >= b`.
#[derive(Clone, Copy)]
struct Addition {
    sum: u32,
    a: u32,
    b: u32,
}

/// An addition sequence, in ascending order of sums.
struct Sequence {
    steps: [Addition; NODES],
    len: usize,
}

/// The standing of a value in the Bos–Coster working set.
#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    /// Given.
    Base,
    /// Wanted, its step not yet chosen.
    Pending,
    /// Its step chosen.
    Done,
}

#[derive(Clone, Copy)]
struct Node {
    value: u32,
    state: State,
}

/// The Bos–Coster working set: every value the sequence will hold.
struct Nodes {
    nodes: [Node; NODES],
    len: usize,
}

impl Nodes {
    const EMPTY: Self = Self {
        nodes: [Node {
            value: 0,
            state: State::Base,
        }; NODES],
        len: 0,
    };

    const fn position(&self, value: u32) -> Option<usize> {
        let mut index = 0;
        while index < self.len {
            if self.nodes[index].value == value {
                return Some(index);
            }
            index += 1;
        }
        None
    }

    const fn contains(&self, value: u32) -> bool {
        self.position(value).is_some()
    }

    /// Adds `value` in `state` when absent; `false` when the set is full.
    const fn insert(&mut self, value: u32, state: State) -> bool {
        if self.contains(value) {
            return true;
        }
        if self.len >= NODES {
            return false;
        }
        self.nodes[self.len] = Node { value, state };
        self.len += 1;
        true
    }

    const fn mark_done(&mut self, value: u32) {
        if let Some(index) = self.position(value) {
            self.nodes[index].state = State::Done;
        }
    }

    const fn largest_pending(&self) -> Option<u32> {
        let mut best: Option<u32> = None;
        let mut index = 0;
        while index < self.len {
            let node = self.nodes[index];
            if matches!(node.state, State::Pending)
                && match best {
                    Some(value) => node.value > value,
                    None => true,
                }
            {
                best = Some(node.value);
            }
            index += 1;
        }
        best
    }

    /// The squarings a star chain from the largest set value at most `value` needs to reach it;
    /// `0` for a set value.
    const fn climb(&self, value: u32) -> u32 {
        value - self.largest_below(value + 1)
    }

    /// The largest value below `bound`; `1` is always present.
    const fn largest_below(&self, bound: u32) -> u32 {
        let mut best = 0;
        let mut index = 0;
        while index < self.len {
            let value = self.nodes[index].value;
            if value < bound && value > best {
                best = value;
            }
            index += 1;
        }
        best
    }

    /// The pair `a >= b` of the set with `a + b <= f` and the smallest gap `f - a - b`; among
    /// equal gaps, the largest `a`. Returned as `(a, b, gap)`.
    const fn approximation(&self, f: u32) -> Option<(u32, u32, u32)> {
        let mut best: Option<(u32, u32, u32)> = None;
        let mut i = 0;
        while i < self.len {
            let a = self.nodes[i].value;
            let mut j = 0;
            while j < self.len {
                let b = self.nodes[j].value;
                if b <= a && a + b <= f {
                    let gap = f - a - b;
                    let better = match best {
                        None => true,
                        Some((best_a, _, best_gap)) => {
                            gap < best_gap || (gap == best_gap && a > best_a)
                        }
                    };
                    if better {
                        best = Some((a, b, gap));
                    }
                }
                j += 1;
            }
            i += 1;
        }
        best
    }
}

impl Sequence {
    const EMPTY: Self = Self {
        steps: [Addition { sum: 0, a: 0, b: 0 }; NODES],
        len: 0,
    };

    /// Records `sum = a + b`; `false` when the sequence is full.
    const fn push(&mut self, sum: u32, a: u32, b: u32) -> bool {
        if self.len >= NODES {
            return false;
        }
        let (a, b) = if a >= b { (a, b) } else { (b, a) };
        self.steps[self.len] = Addition { sum, a, b };
        self.len += 1;
        true
    }

    /// Insertion sort by sum.
    const fn sort(&mut self) {
        let mut index = 1;
        while index < self.len {
            let step = self.steps[index];
            let mut slot = index;
            while slot > 0 && self.steps[slot - 1].sum > step.sum {
                self.steps[slot] = self.steps[slot - 1];
                slot -= 1;
            }
            self.steps[slot] = step;
            index += 1;
        }
    }

    /// An addition sequence from `base` through every target, resolving the largest pending
    /// value first. `None` when the working set overflows.
    const fn through(targets: &Targets, base: &[u32], cost: Cost) -> Option<Self> {
        let mut nodes = Nodes::EMPTY;
        let mut index = 0;
        while index < base.len() {
            if !nodes.insert(base[index], State::Base) {
                return None;
            }
            index += 1;
        }
        let mut index = 0;
        while index < targets.len {
            if !nodes.insert(targets.values[index], State::Pending) {
                return None;
            }
            index += 1;
        }
        let mut sequence = Self::EMPTY;
        while let Some(f) = nodes.largest_pending() {
            let resolved = match cost {
                Cost::Runs => resolve_run(&mut nodes, &mut sequence, f),
                Cost::Windows => resolve_window(&mut nodes, &mut sequence, f),
            };
            if !resolved {
                return None;
            }
            nodes.mark_done(f);
        }
        sequence.sort();
        Some(sequence)
    }
}

/// What a step `c = a + b` with `a >= b` costs.
#[derive(Clone, Copy)]
enum Cost {
    /// `b` squarings and one multiplication: the sum of `b` over the sequence.
    Runs,
    /// One multiplication, or one squaring when `a == b`: the length of the sequence.
    Windows,
}

/// Chooses the step of `f` for a window sequence, by the Bos–Coster heuristics: approximation
/// when two set values sum to `f`, or fall short of it by a set value; halving into a doubling
/// chain when `f` is at least twice the next value; else the difference to the next value.
const fn resolve_window(nodes: &mut Nodes, sequence: &mut Sequence, f: u32) -> bool {
    if let Some((a, b, gap)) = nodes.approximation(f) {
        if gap == 0 {
            return sequence.push(f, a, b);
        }
        if nodes.contains(gap) && a + gap < f {
            return nodes.insert(a + gap, State::Pending) && sequence.push(f, a + gap, b);
        }
    }
    let next = nodes.largest_below(f);
    if f < 2 * next {
        let delta = f - next;
        return nodes.insert(delta, State::Pending) && sequence.push(f, next, delta);
    }
    halve(nodes, sequence, f, next)
}

/// Chooses the step of `f` for a run sequence, whose cost is the sum of the smaller summands:
/// an exact pair with the largest `a`, else the cheaper of a multiple of the next value and a
/// halving, by the estimate of a chain that climbs from the nearest set value.
const fn resolve_run(nodes: &mut Nodes, sequence: &mut Sequence, f: u32) -> bool {
    if let Some((a, b, 0)) = nodes.approximation(f) {
        return sequence.push(f, a, b);
    }
    let next = nodes.largest_below(f);
    let (q, r) = (f / next, f % next);
    let multiple = (q - 1) * next + r + nodes.climb(r) + q.ilog2() + q.count_ones() - 1;
    let halving = if f >= 2 * next {
        let mut k = 0;
        while (f >> (k + 1)) >= next {
            k += 1;
        }
        let d = f >> k;
        let r = f - (d << k);
        Some(((1 << k) - 1) * d + r + nodes.climb(d) + nodes.climb(r) + k)
    } else {
        None
    };
    if let Some(halving) = halving
        && halving < multiple
    {
        return halve(nodes, sequence, f, next);
    }
    // `f = q next + r`: the binary method on `q` from `next`, then `r`.
    let mut acc = next;
    let mut index = q.ilog2();
    while index > 0 {
        index -= 1;
        let doubled = 2 * acc;
        if doubled == f {
            return sequence.push(f, acc, acc);
        }
        if !nodes.contains(doubled)
            && !(nodes.insert(doubled, State::Done) && sequence.push(doubled, acc, acc))
        {
            return false;
        }
        acc = doubled;
        if (q >> index) & 1 == 1 {
            let sum = acc + next;
            if sum == f {
                return sequence.push(f, acc, next);
            }
            if !nodes.contains(sum)
                && !(nodes.insert(sum, State::Done) && sequence.push(sum, acc, next))
            {
                return false;
            }
            acc = sum;
        }
    }
    nodes.insert(r, State::Pending) && sequence.push(f, acc, r)
}

/// Halving: `f = d 2^k + r` with `next <= d < 2 next`, through the doublings of `d`.
const fn halve(nodes: &mut Nodes, sequence: &mut Sequence, f: u32, next: u32) -> bool {
    let mut k = 0;
    while (f >> (k + 1)) >= next {
        k += 1;
    }
    let d = f >> k;
    let r = f - (d << k);
    if !nodes.insert(d, State::Pending) {
        return false;
    }
    let mut j = 1;
    while j <= k {
        let doubled = d << j;
        if doubled == f {
            return sequence.push(f, d << (j - 1), d << (j - 1));
        }
        if !nodes.contains(doubled)
            && !(nodes.insert(doubled, State::Done)
                && sequence.push(doubled, d << (j - 1), d << (j - 1)))
        {
            return false;
        }
        j += 1;
    }
    nodes.insert(r, State::Pending) && sequence.push(f, d << k, r)
}

/// The registers of the dictionary entries.
struct Map {
    entries: [(Entry, Register); REGISTERS],
    len: usize,
}

impl Map {
    /// The base, `x`, in register `0`: the run of length `1`.
    const EMPTY: Self = Self {
        entries: [(Entry::Run(1), 0); REGISTERS],
        len: 1,
    };

    const fn add(&mut self, entry: Entry, register: Register) {
        if self.len < REGISTERS {
            self.entries[self.len] = (entry, register);
            self.len += 1;
        }
    }

    const fn find(&self, entry: Entry) -> Option<Register> {
        let mut index = 0;
        while index < self.len {
            let (candidate, register) = self.entries[index];
            if candidate.equals(entry) {
                return Some(register);
            }
            index += 1;
        }
        None
    }

    /// The register of `entry`. A window value `2^k - 1` without a register of its own is the
    /// run of length `k`.
    const fn register(&self, entry: Entry) -> Option<Register> {
        if let Some(register) = self.find(entry) {
            return Some(register);
        }
        match entry {
            Entry::Window(value) if power_of_two(value + 1) => {
                self.find(Entry::Run((value + 1).trailing_zeros()))
            }
            Entry::Window(_) | Entry::Run(_) => None,
        }
    }
}

impl<M: Modulus> Fp<M> {
    /// `self^e` for the exponent of `program`, by the constant-time arithmetic when `CT` holds.
    #[inline(always)]
    pub(crate) fn pow_program<const CT: bool>(self, program: &Program<M>) -> Self {
        let mut registers = [self; REGISTERS];
        let mut index = 0;
        while index < program.dictionary {
            let step = program.build(index);
            let value = registers[step.src as usize % REGISTERS].square_times::<CT>(step.squarings);
            registers[step.dst as usize % REGISTERS] = match step.mul {
                Some(mul) => value.mul_with::<CT>(registers[mul as usize % REGISTERS]),
                None => value,
            };
            index += 1;
        }
        let mut acc = match program.first {
            Some(first) => registers[first as usize % REGISTERS],
            None => Self::ONE,
        };
        while index < program.len {
            let step = program.climb(index);
            acc = acc.square_times::<CT>(step.squarings);
            if let Some(mul) = step.mul {
                acc = acc.mul_with::<CT>(registers[mul as usize % REGISTERS]);
            }
            index += 1;
        }
        acc
    }

    /// `self^(2^count)`, by `count` squarings, two per pass.
    #[inline(always)]
    pub(crate) fn square_times<const CT: bool>(self, count: u32) -> Self {
        let mut value = self;
        let mut remaining = count;
        while remaining >= 2 {
            value = value.square_with::<CT>().square_with::<CT>();
            remaining -= 2;
        }
        if remaining == 1 {
            value = value.square_with::<CT>();
        }
        value
    }
}
