//! The cells of a comparison: each operation on each row in each crate.
//!
//! [`visit`] walks the rows of a [`Suite`] (one operation on one row key, such as a width or a
//! field) and offers every crate's cell of the row to a [`Visitor`]. A cell either runs one case
//! (the crate's function over a batch of operand sets) or records why the crate has no case.
//! Before a case is measured, its result on every operand set is checked against the accepted
//! results that the suite computed independently.

use core::fmt;
use std::hint::black_box;

use crate::record::Key;

/// The number of operand sets in the batch of each case.
pub const BATCH: usize = 256;

/// Little-endian `u64` limbs.
pub type Limbs = Vec<u64>;

/// `limbs` without its high zero limbs.
#[must_use]
pub fn trim(limbs: &[u64]) -> &[u64] {
    let len = limbs
        .iter()
        .rposition(|limb| *limb != 0)
        .map_or(0, |top| top + 1);
    &limbs[..len]
}

/// One operand set and the results that a crate may return for it, each trimmed.
pub struct Sample {
    /// The operands.
    pub operands: Vec<Limbs>,
    /// The accepted results, as [`trim`] leaves them.
    pub accepted: Vec<Limbs>,
}

/// A comparison: its operations, its rows and the adapters of its crates.
pub trait Suite {
    /// An operation.
    type Op: Copy + Eq + fmt::Display + Send + Sync + 'static;
    /// A row key: the width or the field of a row.
    type Key: Key;

    /// Every operation, in measurement order.
    const OPS: &'static [Self::Op];

    /// Every row key, in measurement order.
    fn keys() -> Vec<Self::Key>;

    /// The operand sets of `op` on the row `key`, with their accepted results. Every crate in
    /// the row sees the same operands.
    fn samples(op: Self::Op, key: &Self::Key) -> Vec<Sample>;

    /// Offers the cells of every crate in `row`.
    fn offer<V: Visitor<Self::Op, Self::Key>>(
        row: Row<Self::Op, Self::Key, V>,
    ) -> Row<Self::Op, Self::Key, V>;
}

/// One cell: an operation on a row in a crate. It displays as `op/key/crate`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaseId<O, K> {
    /// The operation.
    pub op: O,
    /// The row.
    pub key: K,
    /// The crate.
    pub lib: &'static str,
}

impl<O: fmt::Display, K: fmt::Display> fmt::Display for CaseId<O, K> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}/{}/{}", self.op, self.key, self.lib)
    }
}

/// What a cell runs.
#[derive(Clone, Copy, Debug)]
pub enum Call {
    /// The crate's function for the operation, by name.
    Api(&'static str),
    /// No case; the reason.
    Absent(&'static str),
}

/// Receives every cell of the comparison.
pub trait Visitor<O, K>: Sized {
    /// Takes note of the cell `id`, which runs `call`.
    fn note(&mut self, _id: &CaseId<O, K>, _call: &Call) {}

    /// Whether to check and measure the case `id`.
    fn wants(&self, id: &CaseId<O, K>) -> bool;

    /// Measures `run` over `batch`, the inputs of the case `id`.
    #[must_use]
    fn measure<I, R>(self, id: &CaseId<O, K>, batch: &[I], run: impl Fn(&I) -> R + Copy) -> Self;
}

/// Offers every cell of `S` to `visitor`: operation by operation, row by row, crate by crate.
#[must_use]
pub fn visit<S: Suite, V: Visitor<S::Op, S::Key>>(visitor: V) -> V {
    S::OPS.iter().fold(visitor, |visitor, op| {
        S::keys().into_iter().fold(visitor, |visitor, key| {
            S::offer(Row {
                op: *op,
                key,
                samples: None,
                visitor,
                draw: S::samples,
            })
            .visitor
        })
    })
}

/// Runs `run` on each input of `batch` and returns the number of runs. Each input and each
/// result passes through [`black_box`], so the compiler can neither precompute a result nor skip
/// one. Callgrind counts the instructions from the entry of this function to its exit, so it
/// keeps a symbol of its own.
#[inline(never)]
pub fn run_batch<I, R>(batch: &[I], run: impl Fn(&I) -> R) -> usize {
    batch
        .iter()
        .map(|input| black_box(run(black_box(input))))
        .fold(0, |runs, _result| runs + 1)
}

/// One operation on one row, with the visitor that receives its cells.
pub struct Row<O, K, V> {
    op: O,
    key: K,
    /// The operand sets, drawn when the first case of the row is checked.
    samples: Option<Vec<Sample>>,
    visitor: V,
    draw: fn(O, &K) -> Vec<Sample>,
}

impl<O: Copy + Eq + fmt::Display, K: Key, V: Visitor<O, K>> Row<O, K, V> {
    /// The row key.
    #[must_use]
    pub const fn key(&self) -> &K {
        &self.key
    }

    /// Offers the cells of `lib` through `adapter`. Panics unless the adapter offers exactly one
    /// cell for the row's operation.
    #[must_use]
    pub fn lib(
        self,
        lib: &'static str,
        adapter: impl FnOnce(Cells<O, K, V>) -> Cells<O, K, V>,
    ) -> Self {
        let cells = adapter(Cells {
            row: self,
            lib,
            offered: 0,
        });
        assert_eq!(
            cells.offered, 1,
            "{lib} offers {} cells for {} on {}",
            cells.offered, cells.row.op, cells.row.key
        );
        cells.row
    }
}

/// The cells of one crate in one row. The crate's adapter offers a cell for every operation;
/// only the cell of the row's operation reaches the visitor.
pub struct Cells<O, K, V> {
    row: Row<O, K, V>,
    lib: &'static str,
    offered: usize,
}

impl<O: Copy + Eq + fmt::Display, K: Key, V: Visitor<O, K>> Cells<O, K, V> {
    /// The row key.
    pub const fn key(&self) -> &K {
        &self.row.key
    }

    /// Offers the case that runs the crate's `function` for `op`. `prepare` builds the input of
    /// one operand set; `run` is the operation on one input; `read` is the result as limbs, or
    /// `None` when the result holds no value.
    #[must_use]
    pub fn api<I, R>(
        self,
        op: O,
        function: &'static str,
        prepare: impl Fn(&[Limbs]) -> I,
        run: impl Fn(&I) -> R + Copy,
        read: impl Fn(&R) -> Option<Limbs>,
    ) -> Self {
        self.offer(op, Call::Api(function), prepare, run, read)
    }

    /// Records that the crate has no case for `op`, for `reason`.
    #[must_use]
    pub fn absent(mut self, op: O, reason: &'static str) -> Self {
        if op == self.row.op {
            self.offered += 1;
            let id = self.id();
            self.row.visitor.note(&id, &Call::Absent(reason));
        }
        self
    }

    fn offer<I, R>(
        mut self,
        op: O,
        call: Call,
        prepare: impl Fn(&[Limbs]) -> I,
        run: impl Fn(&I) -> R + Copy,
        read: impl Fn(&R) -> Option<Limbs>,
    ) -> Self {
        if op != self.row.op {
            return self;
        }
        self.offered += 1;
        let id = self.id();
        self.row.visitor.note(&id, &call);
        if self.row.visitor.wants(&id) {
            let draw = self.row.draw;
            let samples = self.row.samples.get_or_insert_with(|| draw(id.op, &id.key));
            let batch: Vec<I> = samples
                .iter()
                .map(|sample| prepare(&sample.operands))
                .collect();
            check(&id, samples, &batch, run, read);
            self.row.visitor = self.row.visitor.measure(&id, &batch, run);
        }
        self
    }

    fn id(&self) -> CaseId<O, K> {
        CaseId {
            op: self.row.op,
            key: self.row.key.clone(),
            lib: self.lib,
        }
    }
}

/// Panics unless `run` returns an accepted result on every input of `batch`.
fn check<O: fmt::Display, K: fmt::Display, I, R>(
    id: &CaseId<O, K>,
    samples: &[Sample],
    batch: &[I],
    run: impl Fn(&I) -> R,
    read: impl Fn(&R) -> Option<Limbs>,
) {
    for (index, (sample, input)) in samples.iter().zip(batch).enumerate() {
        let result = read(&run(input));
        let accepted = result
            .as_deref()
            .map(trim)
            .is_some_and(|result| sample.accepted.iter().any(|value| value == result));
        assert!(
            accepted,
            "{id}: operand set {index}: {:x?} gave {result:x?}, accepted {:x?}",
            sample.operands, sample.accepted
        );
    }
}

/// SplitMix64.
pub struct Rng(pub u64);

impl Rng {
    /// The next value.
    #[must_use]
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// A uniform value of `limbs` limbs.
    #[must_use]
    pub fn uniform(&mut self, limbs: usize) -> Limbs {
        (0..limbs).map(|_| self.next_u64()).collect()
    }

    /// A uniform value of exactly `bits` bits, in `limbs` limbs: bit `bits - 1` is set.
    #[must_use]
    pub fn exact(&mut self, limbs: usize, bits: usize) -> Limbs {
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

    /// A uniform value below `bound`, in as many limbs as `bound`: a draw of the bit length of
    /// `bound`, until one falls below it.
    #[must_use]
    pub fn below(&mut self, bound: &[u64]) -> Limbs {
        let significant = trim(bound);
        let bits = significant.len() * 64
            - significant
                .last()
                .map_or(0, |top| top.leading_zeros() as usize);
        loop {
            let mut value = self.uniform(bound.len());
            for (index, limb) in value.iter_mut().enumerate() {
                match bits.saturating_sub(index * 64) {
                    0 => *limb = 0,
                    kept @ 1..64 => *limb &= u64::MAX >> (64 - kept),
                    _ => {}
                }
            }
            if value.iter().rev().cmp(bound.iter().rev()) == core::cmp::Ordering::Less {
                return value;
            }
        }
    }
}
