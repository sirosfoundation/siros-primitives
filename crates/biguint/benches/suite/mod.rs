//! The cells of the comparison: each operation at each width in each crate.
//!
//! [`visit`] walks the rows (one operation at one width) and offers every crate's cell of the
//! row to a [`Visitor`]. A cell either runs one case (the crate's function over a batch of
//! operand sets) or records why the crate has no case. Before a case is measured, its result on
//! every operand set is compared with the exact result from `num-bigint`.

mod libs;
mod limbs;

use core::cmp::Ordering;
use core::fmt;
use std::hint::black_box;

use num_bigint::BigUint;

pub use libs::WIDTHS;
use limbs::trim;
pub use limbs::{FromLimbs, Limbs, ToLimbs};

/// The number of operand sets in the batch of each case.
pub const BATCH: usize = 256;

/// An operation under comparison.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Op {
    /// `a + b`.
    Add,
    /// `a - b`.
    Sub,
    /// `a * b`.
    Mul,
    /// `a / b`.
    Div,
    /// `a % b`.
    Mod,
    /// `(a * b) mod m`.
    MulMod,
}

impl Op {
    /// Every operation, in measurement order.
    pub const ALL: [Self; 6] = [
        Self::Add,
        Self::Sub,
        Self::Mul,
        Self::Div,
        Self::Mod,
        Self::MulMod,
    ];

    /// One operand set of width `bits`:
    /// - add, mul: two uniform `bits`-bit values;
    /// - sub: two uniform `bits`-bit values, the larger first;
    /// - div, mod: a dividend of exactly `bits` bits and a divisor of exactly `bits / 2` bits;
    /// - mulmod: a modulus `m` of exactly `bits` bits and two uniform values below `m`, as
    ///   `[a, b, m]`.
    ///
    /// "Exactly `n` bits" means that bit `n - 1` is set.
    fn operands(self, bits: usize, rng: &mut Rng) -> Vec<Limbs> {
        let limbs = bits / 64;
        match self {
            Self::Add | Self::Mul => vec![rng.uniform(limbs), rng.uniform(limbs)],
            Self::Sub => {
                let (a, b) = (rng.uniform(limbs), rng.uniform(limbs));
                if compare(&a, &b) == Ordering::Less {
                    vec![b, a]
                } else {
                    vec![a, b]
                }
            }
            Self::Div | Self::Mod => vec![rng.exact(limbs, bits), rng.exact(limbs, bits / 2)],
            Self::MulMod => {
                let modulus = rng.exact(limbs, bits);
                vec![rng.below(&modulus), rng.below(&modulus), modulus]
            }
        }
    }

    /// The exact result of the operation on `operands`.
    fn exact(self, operands: &[BigUint]) -> BigUint {
        match (self, operands) {
            (Self::Add, [a, b]) => a + b,
            (Self::Sub, [a, b]) => a - b,
            (Self::Mul, [a, b]) => a * b,
            (Self::Div, [a, b]) => a / b,
            (Self::Mod, [a, b]) => a % b,
            (Self::MulMod, [a, b, modulus]) => a * b % modulus,
            (op, operands) => panic!("{op} on {} operands", operands.len()),
        }
    }
}

impl fmt::Display for Op {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Add => "add",
            Self::Sub => "sub",
            Self::Mul => "mul",
            Self::Div => "div",
            Self::Mod => "mod",
            Self::MulMod => "mulmod",
        })
    }
}

/// A crate under comparison.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Lib {
    /// `biguint`, the crate under test.
    Biguint,
    /// `u64` at 64 bits and `u128` at 128 bits.
    Native,
    /// `ruint`.
    Ruint,
    /// `crypto-bigint`.
    CryptoBigint,
    /// `bnum`.
    Bnum,
    /// The parity `uint` crate.
    Uint,
    /// `num-bigint`.
    NumBigint,
}

impl Lib {
    /// Whether the results wrap modulo `2^bits`. `num-bigint` returns exact results.
    const fn wraps(self) -> bool {
        !matches!(self, Self::NumBigint)
    }
}

impl fmt::Display for Lib {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Biguint => "biguint",
            Self::Native => "native",
            Self::Ruint => "ruint",
            Self::CryptoBigint => "crypto-bigint",
            Self::Bnum => "bnum",
            Self::Uint => "uint",
            Self::NumBigint => "num-bigint",
        })
    }
}

/// One cell: an operation at a width in a crate. It displays as `op/bits/crate`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CaseId {
    /// The operation.
    pub op: Op,
    /// The width in bits.
    pub bits: usize,
    /// The crate.
    pub lib: Lib,
}

impl fmt::Display for CaseId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}/{}/{}", self.op, self.bits, self.lib)
    }
}

/// What a cell runs.
#[derive(Clone, Copy, Debug)]
pub enum Call {
    /// The crate's function for the operation, by name.
    Api(&'static str),
    /// A composition of the crate's public functions, described.
    Composed(&'static str),
    /// No case; the reason.
    Absent(&'static str),
}

impl Call {
    /// `api`, `composed` or `absent`.
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Api(_) => "api",
            Self::Composed(_) => "composed",
            Self::Absent(_) => "absent",
        }
    }

    /// The function, the composition, or the reason for the absence.
    pub const fn detail(&self) -> &'static str {
        match self {
            Self::Api(detail) | Self::Composed(detail) | Self::Absent(detail) => detail,
        }
    }
}

/// Receives every cell of the comparison.
pub trait Visitor: Sized {
    /// Takes note of the cell `id`, which runs `call`.
    fn note(&mut self, _id: &CaseId, _call: &Call) {}

    /// Whether to check and measure the case `id`.
    fn wants(&self, id: &CaseId) -> bool;

    /// Measures `run` over `batch`, the inputs of the case `id`.
    #[must_use]
    fn measure<I, O>(self, id: &CaseId, batch: &[I], run: impl Fn(&I) -> O + Copy) -> Self;
}

/// Offers every cell to `visitor`: operation by operation, width by width, crate by crate.
pub fn visit<V: Visitor>(visitor: V) -> V {
    Op::ALL.into_iter().fold(visitor, |visitor, op| {
        WIDTHS.into_iter().fold(visitor, |visitor, bits| {
            libs::offer(Row {
                op,
                bits,
                samples: None,
                visitor,
            })
            .visitor
        })
    })
}

/// Runs `run` on each input of `batch` and returns the number of runs. Each input and each
/// result passes through [`black_box`], so the compiler can neither precompute a result nor skip
/// one.
#[inline(never)]
pub fn run_batch<I, O>(batch: &[I], run: impl Fn(&I) -> O) -> usize {
    batch
        .iter()
        .map(|input| black_box(run(black_box(input))))
        .fold(0, |runs, _result| runs + 1)
}

/// One operation at one width, with the visitor that receives its cells.
pub struct Row<V> {
    op: Op,
    bits: usize,
    /// The operand sets, drawn when the first case of the row is checked.
    samples: Option<Vec<Sample>>,
    visitor: V,
}

impl<V: Visitor> Row<V> {
    /// The width in bits.
    pub const fn bits(&self) -> usize {
        self.bits
    }

    /// Offers the cells of `lib` through `adapter`. Panics unless the adapter offers exactly one
    /// cell for the row's operation.
    pub fn lib(self, lib: Lib, adapter: impl FnOnce(Cells<V>) -> Cells<V>) -> Self {
        let cells = adapter(Cells {
            row: self,
            lib,
            offered: 0,
        });
        assert_eq!(
            cells.offered, 1,
            "{lib} offers {} cells for {} at {} bits",
            cells.offered, cells.row.op, cells.row.bits
        );
        cells.row
    }
}

/// The cells of one crate in one row. The crate's adapter offers a cell for every operation;
/// only the cell of the row's operation reaches the visitor.
pub struct Cells<V> {
    row: Row<V>,
    lib: Lib,
    offered: usize,
}

impl<V: Visitor> Cells<V> {
    /// Offers the case that runs the crate's `function` for `op`. `prepare` builds the input of
    /// one operand set; `run` is the operation on one input.
    pub fn api<I, O: ToLimbs>(
        self,
        op: Op,
        function: &'static str,
        prepare: impl Fn(&[Limbs]) -> I,
        run: impl Fn(&I) -> O + Copy,
    ) -> Self {
        self.offer(op, Call::Api(function), prepare, run)
    }

    /// Offers the case that composes `op` from the crate's public functions, as `description`
    /// says.
    pub fn composed<I, O: ToLimbs>(
        self,
        op: Op,
        description: &'static str,
        prepare: impl Fn(&[Limbs]) -> I,
        run: impl Fn(&I) -> O + Copy,
    ) -> Self {
        self.offer(op, Call::Composed(description), prepare, run)
    }

    /// Records that the crate has no case for `op`, for `reason`.
    pub fn absent(mut self, op: Op, reason: &'static str) -> Self {
        if op == self.row.op {
            self.offered += 1;
            let id = self.id();
            self.row.visitor.note(&id, &Call::Absent(reason));
        }
        self
    }

    fn offer<I, O: ToLimbs>(
        mut self,
        op: Op,
        call: Call,
        prepare: impl Fn(&[Limbs]) -> I,
        run: impl Fn(&I) -> O + Copy,
    ) -> Self {
        if op != self.row.op {
            return self;
        }
        self.offered += 1;
        let id = self.id();
        self.row.visitor.note(&id, &call);
        if self.row.visitor.wants(&id) {
            let samples = self
                .row
                .samples
                .get_or_insert_with(|| samples(id.op, id.bits));
            let batch: Vec<I> = samples
                .iter()
                .map(|sample| prepare(&sample.operands))
                .collect();
            check(&id, samples, &batch, run);
            self.row.visitor = self.row.visitor.measure(&id, &batch, run);
        }
        self
    }

    const fn id(&self) -> CaseId {
        CaseId {
            op: self.row.op,
            bits: self.row.bits,
            lib: self.lib,
        }
    }
}

/// One operand set and the exact result of the row's operation on it.
struct Sample {
    operands: Vec<Limbs>,
    exact: Limbs,
}

impl Sample {
    /// The result that `lib` must return at `bits` bits.
    fn expected(&self, lib: Lib, bits: usize) -> &[u64] {
        if lib.wraps() {
            trim(&self.exact[..self.exact.len().min(bits / 64)])
        } else {
            &self.exact
        }
    }
}

/// The operand sets of `op` at `bits` bits, with their exact results. The seed depends on the
/// operation and the width only, so every crate in a row sees the same operands.
fn samples(op: Op, bits: usize) -> Vec<Sample> {
    let mut rng = Rng(((op as u64) << 32) | bits as u64);
    (0..BATCH)
        .map(|_| {
            let operands = op.operands(bits, &mut rng);
            let values: Vec<BigUint> = operands
                .iter()
                .map(|operand| BigUint::from_limbs(operand))
                .collect();
            Sample {
                exact: op.exact(&values).to_u64_digits(),
                operands,
            }
        })
        .collect()
}

/// Panics unless `run` returns the expected result on every input of `batch`.
fn check<I, O: ToLimbs>(id: &CaseId, samples: &[Sample], batch: &[I], run: impl Fn(&I) -> O) {
    for (index, (sample, input)) in samples.iter().zip(batch).enumerate() {
        let result = run(input).to_limbs();
        assert_eq!(
            result.as_deref().map(trim),
            Some(sample.expected(id.lib, id.bits)),
            "{id}: operand set {index}: {:x?}",
            sample.operands
        );
    }
}

/// `a` against `b`, two values of the same number of limbs.
fn compare(a: &[u64], b: &[u64]) -> Ordering {
    a.iter().rev().cmp(b.iter().rev())
}

/// SplitMix64.
struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// A uniform value of `limbs` limbs.
    fn uniform(&mut self, limbs: usize) -> Limbs {
        (0..limbs).map(|_| self.next_u64()).collect()
    }

    /// A uniform value of exactly `bits` bits, in `limbs` limbs.
    fn exact(&mut self, limbs: usize, bits: usize) -> Limbs {
        let mut value: Limbs = (0..limbs)
            .map(|index| {
                let limb = self.next_u64();
                match bits.saturating_sub(index * 64) {
                    0 => 0,
                    kept @ 1..64 => limb & (u64::MAX >> (64 - kept)),
                    _ => limb,
                }
            })
            .collect();
        value[(bits - 1) / 64] |= 1 << ((bits - 1) % 64);
        value
    }

    /// A uniform value below `bound`, in as many limbs as `bound`.
    fn below(&mut self, bound: &[u64]) -> Limbs {
        loop {
            let value = self.uniform(bound.len());
            if compare(&value, bound) == Ordering::Less {
                return value;
            }
        }
    }
}
