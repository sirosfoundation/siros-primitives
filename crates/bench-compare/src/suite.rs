//! The cells of a comparison: each operation on each row in each crate.
//!
//! [`visit`] walks the rows of a [`Suite`] (one operation on one row key, such as a width or a
//! field) and offers every crate's cell of the row to a [`Visitor`]. A cell either runs one case
//! (the crate's function on operand sets that the suite draws) or records why the crate has no
//! case. Before a case is measured, its result on the checking operand sets is compared with
//! the accepted results that the suite computed independently. A case with a chain form also
//! runs as the chain `x = op(x, y)`, where each result is the first operand of the next
//! operation.

use core::fmt;

use crate::barrier::{Barrier, opaque, sink};
use crate::record::Key;

/// The number of operand sets that check each case.
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

    /// The operations that the bench measures, in measurement order. A case offers its operation
    /// as a [`Selected`], whose `MEASURED` must agree with this list.
    fn ops() -> Vec<Self::Op>;

    /// Every row key, in measurement order.
    fn keys() -> Vec<Self::Key>;

    /// The [`BATCH`] operand sets that check the cases of `op` on the row `key`, with their
    /// accepted results. Every crate in the row sees the same operands.
    fn samples(op: Self::Op, key: &Self::Key) -> Vec<Sample>;

    /// One operand set of `op` on the row `key`, drawn with `rng` from the distribution of
    /// [`samples`](Self::samples).
    fn operands(op: Self::Op, key: &Self::Key, rng: &mut Rng) -> Vec<Limbs>;

    /// Offers the cells of every crate in `row`.
    fn offer<V: Visitor<Self::Op, Self::Key>>(
        row: Row<Self::Op, Self::Key, V>,
    ) -> Row<Self::Op, Self::Key, V>;
}

/// The operation of a case, with whether the bench measures it as a constant. A case of an
/// operation that the bench leaves out instantiates no code: a suite split over several bench
/// binaries compiles each case in one binary.
pub trait Selected<O> {
    /// Whether the bench measures the operation: it is in [`Suite::ops`].
    const MEASURED: bool;

    /// The operation.
    fn op(&self) -> O;
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
    /// The crate's function for the operation.
    Case(Function),
    /// No case; the reason.
    Absent(&'static str),
}

/// A crate's function for an operation.
#[derive(Clone, Copy, Debug)]
pub struct Function {
    /// The function, by name.
    pub name: &'static str,
    /// How its results deviate from the row's definition, in one phrase, such as a skipped final
    /// reduction; `None` for a function that meets it.
    pub deviation: Option<&'static str>,
}

/// An input whose first operand the result of the operation can replace: one step of the chain
/// `x = op(x, y)`. The chain applies where the result has the type of the first operand and the
/// other operands are `Copy`: a step copies them, which a chain of the operation alone does not
/// do, and a copy of an allocated value would allocate.
pub trait Chain<R> {
    /// This input with `result` as its first operand.
    #[must_use]
    fn chain(&self, result: R) -> Self;
}

/// One operand: the result is the next operand.
impl<T> Chain<T> for T {
    fn chain(&self, result: T) -> Self {
        result
    }
}

impl<A, B: Copy> Chain<A> for (A, B) {
    fn chain(&self, result: A) -> Self {
        (result, self.1)
    }
}

impl<A, B: Copy, C: Copy> Chain<A> for (A, B, C) {
    fn chain(&self, result: A) -> Self {
        (result, self.1, self.2)
    }
}

/// The inputs of a case: operand sets drawn fresh from the row's distribution, in the crate's
/// representation.
pub struct Inputs<'a, I> {
    fresh: &'a dyn Fn(&mut Rng) -> I,
}

impl<I> Inputs<'_, I> {
    /// `count` inputs drawn with `rng`.
    #[must_use]
    pub fn draw(&self, rng: &mut Rng, count: usize) -> Vec<I> {
        (0..count).map(|_| (self.fresh)(rng)).collect()
    }
}

/// Receives every cell of the comparison.
pub trait Visitor<O, K>: Sized {
    /// Takes note of the cell `id`, which runs `call`.
    fn note(&mut self, _id: &CaseId<O, K>, _call: &Call) {}

    /// Whether to measure the case `id`.
    fn wants(&self, id: &CaseId<O, K>) -> bool;

    /// Whether to check the case `id` against the accepted results before it is measured.
    fn checks(&self, _id: &CaseId<O, K>) -> bool {
        true
    }

    /// Measures `run` on the inputs that `inputs` draws for the case `id`.
    #[must_use]
    fn measure<I, R>(
        self,
        id: &CaseId<O, K>,
        inputs: &Inputs<'_, I>,
        run: impl Fn(&I) -> R + Copy,
    ) -> Self;

    /// Measures `run` as a chain on the inputs that `inputs` draws, for a case with a chain
    /// form, after [`measure`](Self::measure).
    #[must_use]
    fn measure_chain<I: Chain<R>, R>(
        self,
        _id: &CaseId<O, K>,
        _inputs: &Inputs<'_, I>,
        _run: impl Fn(&I) -> R + Copy,
    ) -> Self {
        self
    }
}

/// Offers every cell of `S` to `visitor`: operation by operation, row by row, crate by crate.
#[must_use]
pub fn visit<S: Suite, V: Visitor<S::Op, S::Key>>(visitor: V) -> V {
    S::ops().into_iter().fold(visitor, |visitor, op| {
        S::keys().into_iter().fold(visitor, |visitor, key| {
            S::offer(Row {
                op,
                key,
                samples: None,
                visitor,
                draw: S::samples,
                operands: S::operands,
            })
            .visitor
        })
    })
}

/// Runs `run` on each input of `batch` and returns the number of runs. The address of each input
/// passes through an empty inline-assembly block, and another takes the address of each result,
/// so the compiler can neither precompute a result nor skip one. The loop calls `run` directly,
/// so an operation that inlines into its caller inlines here whatever its size.
#[inline(never)]
pub fn run_batch<I, R>(batch: &[I], run: impl Fn(&I) -> R) -> usize {
    let mut runs = 0;
    for input in batch {
        sink(&run(opaque(input)));
        runs += 1;
    }
    runs
}

/// Runs `run` on the inputs of `batch` as the chain `x = op(x, y)` and returns the number of
/// runs: from the second input on, the result of the previous run, held by the barrier `B`,
/// replaces the first operand, so each run waits for the one before it, in order. The loop calls
/// `run` directly, as [`run_batch`]'s does.
#[inline(never)]
pub fn run_chain<I: Chain<R>, R, B: Barrier>(batch: &[I], run: impl Fn(&I) -> R) -> usize {
    B::chain(batch, run)
}

/// One operation on one row, with the visitor that receives its cells.
pub struct Row<O, K, V> {
    op: O,
    key: K,
    /// The checking operand sets, drawn when the first case of the row is checked.
    samples: Option<Vec<Sample>>,
    visitor: V,
    draw: fn(O, &K) -> Vec<Sample>,
    operands: fn(O, &K, &mut Rng) -> Vec<Limbs>,
}

impl<O: Copy + Eq + fmt::Display, K: Key, V: Visitor<O, K>> Row<O, K, V> {
    /// The row key.
    #[must_use]
    pub const fn key(&self) -> &K {
        &self.key
    }

    /// The checking operand sets of the row, drawn on the first call.
    fn samples(&mut self) -> &[Sample] {
        let draw = self.draw;
        self.samples.get_or_insert_with(|| draw(self.op, &self.key))
    }

    /// Offers the cells of `lib` through `adapter`. Panics unless the adapter offers exactly one
    /// cell for the row's operation, and unless every deviation it declares reaches a case.
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
            deviation: None,
        });
        assert_eq!(
            cells.offered, 1,
            "{lib} offers {} cells for {} on {}",
            cells.offered, cells.row.op, cells.row.key
        );
        assert!(
            cells.deviation.is_none(),
            "{lib} declares a deviation that no case of {} on {} takes",
            cells.row.op,
            cells.row.key
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
    /// The deviation that [`deviating`](Self::deviating) declared for the next case.
    deviation: Option<&'static str>,
}

impl<O: Copy + Eq + fmt::Display, K: Key, V: Visitor<O, K>> Cells<O, K, V> {
    /// The row key.
    pub const fn key(&self) -> &K {
        &self.row.key
    }

    /// Declares that the next case deviates from the row's definition, as `deviation` says in one
    /// phrase, such as "skips the final reduction; the result is not canonical". The case's
    /// `read` applies the crate's own normalization: a result is accepted when it equals an
    /// accepted result after that normalization, and the report marks the cell. Panics when the
    /// next cell is absent.
    #[must_use]
    pub fn deviating(self, deviation: &'static str) -> Self {
        assert!(
            self.deviation.is_none(),
            "{}: two deviations for one case",
            self.lib
        );
        Self {
            deviation: Some(deviation),
            ..self
        }
    }

    /// Offers the case that runs the crate's `function` for `op`. `prepare` builds the input of
    /// one operand set; `run` is the operation on one input; `read` is the result as limbs, or
    /// `None` when the result holds no value.
    #[must_use]
    pub fn api<I, R>(
        self,
        op: impl Selected<O>,
        function: &'static str,
        prepare: impl Fn(&[Limbs]) -> I,
        run: impl Fn(&I) -> R + Copy,
        read: impl Fn(&R) -> Option<Limbs>,
    ) -> Self {
        self.api_with(
            op,
            function,
            || (),
            |&(), set| prepare(set),
            run,
            |&(), result| read(result),
        )
    }

    /// Offers the case that runs the crate's `function` for `op`, as [`api`](Self::api) does,
    /// with `setup` building what `prepare` and `read` share, such as a change of basis. `setup`
    /// runs once each time the visitor wants the case, before the case is checked and measured.
    #[must_use]
    pub fn api_with<S, I, R>(
        self,
        op: impl Selected<O>,
        function: &'static str,
        setup: impl FnOnce() -> S,
        prepare: impl Fn(&S, &[Limbs]) -> I,
        run: impl Fn(&I) -> R + Copy,
        read: impl Fn(&S, &R) -> Option<Limbs>,
    ) -> Self {
        self.case(
            op,
            function,
            Case {
                setup,
                prepare,
                run,
                read,
            },
            |visitor, _id, _inputs| visitor,
        )
    }

    /// Offers the case of [`api`](Self::api) with its chain form: the case also runs as the chain
    /// `x = op(x, y)`, where each result is the first operand of the next operation.
    #[must_use]
    pub fn chain<I: Chain<R>, R>(
        self,
        op: impl Selected<O>,
        function: &'static str,
        prepare: impl Fn(&[Limbs]) -> I,
        run: impl Fn(&I) -> R + Copy,
        read: impl Fn(&R) -> Option<Limbs>,
    ) -> Self {
        self.chain_with(
            op,
            function,
            || (),
            |&(), set| prepare(set),
            run,
            |&(), result| read(result),
        )
    }

    /// Offers the case of [`api_with`](Self::api_with) with its chain form, as
    /// [`chain`](Self::chain) does.
    #[must_use]
    pub fn chain_with<S, I: Chain<R>, R>(
        self,
        op: impl Selected<O>,
        function: &'static str,
        setup: impl FnOnce() -> S,
        prepare: impl Fn(&S, &[Limbs]) -> I,
        run: impl Fn(&I) -> R + Copy,
        read: impl Fn(&S, &R) -> Option<Limbs>,
    ) -> Self {
        self.case(
            op,
            function,
            Case {
                setup,
                prepare,
                run,
                read,
            },
            |visitor: V, id, inputs| visitor.measure_chain(id, inputs, run),
        )
    }

    /// Offers `case`, which runs the crate's `function`, for `op`, with the deviation declared
    /// before it: when the visitor wants it, checks it if the visitor asks, measures it, and hands
    /// its inputs to `chain`.
    fn case<P: Selected<O>, S, I, R, Setup, Prepare, Run, Read>(
        mut self,
        op: P,
        function: &'static str,
        case: Case<Setup, Prepare, Run, Read>,
        chain: impl FnOnce(V, &CaseId<O, K>, &Inputs<'_, I>) -> V,
    ) -> Self
    where
        Setup: FnOnce() -> S,
        Prepare: Fn(&S, &[Limbs]) -> I,
        Run: Fn(&I) -> R + Copy,
        Read: Fn(&S, &R) -> Option<Limbs>,
    {
        let deviation = self.deviation.take();
        // A constant: a bench that leaves `op` out instantiates none of the code below.
        if !P::MEASURED {
            return self;
        }
        if op.op() != self.row.op {
            return self;
        }
        self.offered += 1;
        let id = self.id();
        let call = Call::Case(Function {
            name: function,
            deviation,
        });
        self.row.visitor.note(&id, &call);
        if self.row.visitor.wants(&id) {
            let shared = (case.setup)();
            if self.row.visitor.checks(&id) {
                let samples = self.row.samples();
                let results: Vec<Option<Limbs>> = samples
                    .iter()
                    .map(|sample| {
                        let input = (case.prepare)(&shared, &sample.operands);
                        (case.read)(&shared, &apart(case.run, &input))
                    })
                    .collect();
                check(&id, samples, &results);
            }
            let operands = self.row.operands;
            let fresh = |rng: &mut Rng| (case.prepare)(&shared, &operands(id.op, &id.key, rng));
            let inputs = Inputs { fresh: &fresh };
            let measured = self.row.visitor.measure(&id, &inputs, case.run);
            self.row.visitor = chain(measured, &id, &inputs);
        }
        self
    }

    /// Records that the crate has no case for `op`, for `reason`. Panics after a deviation that
    /// no case took.
    #[must_use]
    pub fn absent(mut self, op: O, reason: &'static str) -> Self {
        assert!(
            self.deviation.is_none(),
            "{}: a deviation declared for the absent cell of {op}",
            self.lib
        );
        if op == self.row.op {
            self.offered += 1;
            let id = self.id();
            self.row.visitor.note(&id, &Call::Absent(reason));
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

/// The closures of a case: `setup` builds what the inputs share, `prepare` one input, `run` is
/// the operation, and `read` its result as limbs.
struct Case<Setup, Prepare, Run, Read> {
    setup: Setup,
    prepare: Prepare,
    run: Run,
    read: Read,
}

/// `run` on `input`, in a function of its own: the copy of the operation that the check runs stays
/// out of the code of the case, whose optimization time grows faster than its size.
#[inline(never)]
fn apart<I, R>(run: impl Fn(&I) -> R, input: &I) -> R {
    run(input)
}

/// Panics unless the case `id` returned an accepted result for every sample: `results` holds its
/// result on each, in order.
fn check(id: &dyn fmt::Display, samples: &[Sample], results: &[Option<Limbs>]) {
    for (index, (sample, result)) in samples.iter().zip(results).enumerate() {
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
