//! The limb algebra that every multi-limb operation is written in.
//!
//! A carry-free operation is a [`LimbSource`]: its little-endian limb at an index, from that index
//! alone. [`ZeroExtended`] yields the limbs of a slice and `0` past its end; the bitwise operations,
//! the complement and the left bit shift combine sources limb by limb. `Uint::from_source` writes
//! each output limb from its index, which is a plain loop over arrays: the compiler vectorizes it.
//! A bit shift into a slice runs the slice kernels of `kernel`, which pair the limbs without
//! bounds checks.
//!
//! A carry chain is a [`LimbStream`], pulled from the bottom limb up. [`Pull`] reads a source in
//! order, and [`Op2CarryIter`] threads a carry or borrow through two streams. Carries fall out of
//! the algebra: [`AddOp`] on two exhausted operands with a pending carry yields the carry limb.
//! `Uint::from_stream` pulls exactly `ceil(N / 64)` limbs, four per loop iteration
//! ([`for_each_slot`]), and masks the top one when the operation may set bits above `N`. Every
//! combinator inlines.

use core::marker::PhantomData;

use generic_array::sequence::GenericSequence;
use generic_array::{ArrayLength, GenericArray};

use crate::limb::{BorrowingSub, CarryingAdd};
#[cfg(all(target_arch = "wasm32", not(wide_arithmetic)))]
use crate::limb::{add_carry_ahead, sub_borrow_ahead};

/// A little-endian value read by limb index.
pub(crate) trait LimbSource {
    /// Limb `index`; past the limbs of the operands, the fill of the operation.
    fn limb(&self, index: usize) -> u64;

    /// The limbs in order from the bottom.
    #[inline(always)]
    fn pull(self) -> Pull<Self>
    where
        Self: Sized,
    {
        Pull {
            source: self,
            index: 0,
        }
    }
}

/// An infinite little-endian limb iterator.
pub(crate) trait LimbStream {
    /// The next limb.
    fn next_limb(&mut self) -> u64;
}

/// The limbs of a slice, then `0`.
pub(crate) struct ZeroExtended<'a> {
    limbs: &'a [u64],
}

impl<'a> ZeroExtended<'a> {
    #[inline(always)]
    pub(crate) const fn new(limbs: &'a [u64]) -> Self {
        Self { limbs }
    }
}

impl LimbSource for ZeroExtended<'_> {
    #[inline(always)]
    fn limb(&self, index: usize) -> u64 {
        self.limbs.get(index).copied().unwrap_or(0)
    }
}

/// The limbs of a source in order from the bottom.
///
/// The index advances with the consumer's loop, so the compiler proves the reads of a slice in
/// bounds and drops the zero fill when the slice is at least as long as the output.
pub(crate) struct Pull<S> {
    source: S,
    index: usize,
}

impl<S: LimbSource> LimbStream for Pull<S> {
    #[inline(always)]
    fn next_limb(&mut self) -> u64 {
        let limb = self.source.limb(self.index);
        self.index += 1;
        limb
    }
}

/// The next `L` limbs of `stream`. Up to four limbs the pull unrolls; longer pulls run in groups of
/// four. On WebAssembly, a pull of more than 32 limbs runs in blocks of 32. LLVM unrolls a block
/// whole, then splits the operands and the output into locals, with no bulk memory operation.
/// Without wide arithmetic it unrolls only the first block of a loop of blocks, so the second runs
/// outside the loop: the 4096-bit sum took Chrome 247 cycles against 340, Firefox 319 against 398.
#[inline(always)]
pub(crate) fn pull_limbs<L: ArrayLength>(stream: &mut impl LimbStream) -> GenericArray<u64, L> {
    if L::USIZE <= 4 {
        GenericArray::generate(|_| stream.next_limb())
    } else if cfg!(target_arch = "wasm32") && L::USIZE > 32 {
        let mut limbs = GenericArray::<u64, L>::default();
        let mut pull = |slots: &mut [u64]| for_each_slot(slots, |slot| *slot = stream.next_limb());
        let (blocks, rest) = limbs.as_chunks_mut::<32>();
        let mut blocks = blocks.iter_mut();
        if let Some(block) = blocks.next() {
            pull(block);
        }
        if let Some(block) = blocks.next() {
            pull(block);
        }
        for block in blocks {
            pull(block);
        }
        pull(rest);
        limbs
    } else {
        let mut limbs = GenericArray::<u64, L>::default();
        for_each_slot(&mut limbs, |slot| *slot = stream.next_limb());
        limbs
    }
}

/// Runs `step` on every slot of `slots` in order, four slots per loop iteration: the compiler
/// keeps a carry in the flags within each group of four.
#[inline(always)]
pub(crate) fn for_each_slot(slots: &mut [u64], mut step: impl FnMut(&mut u64)) {
    let (groups, rest) = slots.as_chunks_mut::<4>();
    for group in groups {
        for slot in group {
            step(slot);
        }
    }
    for slot in rest {
        step(slot);
    }
}

/// A limb operation on two operands that threads a carry or borrow in `{0, 1}`.
pub(crate) trait Op2CarryOp {
    /// Combines one limb of each operand and updates the carry.
    fn op(a: u64, b: u64, carry: &mut u64) -> u64;
}

/// Two streams combined limb by limb under `O`, starting with carry `0`.
pub(crate) struct Op2CarryIter<A, B, O> {
    a: A,
    b: B,
    carry: u64,
    _op: PhantomData<O>,
}

impl<A, B, O> Op2CarryIter<A, B, O> {
    #[inline(always)]
    pub(crate) const fn new(a: A, b: B) -> Self {
        Self::with_carry(a, b, 0)
    }

    /// The streams combined from `carry` into the bottom limb.
    #[inline(always)]
    pub(crate) const fn with_carry(a: A, b: B, carry: u64) -> Self {
        Self {
            a,
            b,
            carry,
            _op: PhantomData,
        }
    }

    /// The carry or borrow into the next limb.
    #[inline(always)]
    pub(crate) const fn carry(&self) -> u64 {
        self.carry
    }
}

impl<A: LimbStream, B: LimbStream, O: Op2CarryOp> LimbStream for Op2CarryIter<A, B, O> {
    #[inline(always)]
    fn next_limb(&mut self) -> u64 {
        O::op(self.a.next_limb(), self.b.next_limb(), &mut self.carry)
    }
}

/// Addition with carry.
pub(crate) struct AddOp;

impl Op2CarryOp for AddOp {
    #[inline(always)]
    fn op(a: u64, b: u64, carry: &mut u64) -> u64 {
        let (sum, carry_out) = a.add_carry(b, *carry);
        *carry = carry_out;
        sum
    }
}

/// Subtraction with borrow. The collector drops the borrow out of the top limb, which makes the
/// result `(a - b) mod 2^N`.
pub(crate) struct SubOp;

impl Op2CarryOp for SubOp {
    #[inline(always)]
    fn op(a: u64, b: u64, borrow: &mut u64) -> u64 {
        let (difference, borrow_out) = a.sub_borrow(b, *borrow);
        *borrow = borrow_out;
        difference
    }
}

/// [`AddOp`] with the carry of [`add_carry_ahead`], for WebAssembly without wide arithmetic.
#[cfg(all(target_arch = "wasm32", not(wide_arithmetic)))]
pub(crate) struct AddAheadOp;

#[cfg(all(target_arch = "wasm32", not(wide_arithmetic)))]
impl Op2CarryOp for AddAheadOp {
    #[inline(always)]
    fn op(a: u64, b: u64, carry: &mut u64) -> u64 {
        let (sum, carry_out) = add_carry_ahead(a, b, *carry);
        *carry = carry_out;
        sum
    }
}

/// [`SubOp`] with the borrow of [`sub_borrow_ahead`], for WebAssembly without wide arithmetic.
#[cfg(all(target_arch = "wasm32", not(wide_arithmetic)))]
pub(crate) struct SubAheadOp;

#[cfg(all(target_arch = "wasm32", not(wide_arithmetic)))]
impl Op2CarryOp for SubAheadOp {
    #[inline(always)]
    fn op(a: u64, b: u64, borrow: &mut u64) -> u64 {
        let (difference, borrow_out) = sub_borrow_ahead(a, b, *borrow);
        *borrow = borrow_out;
        difference
    }
}

/// A limb operation on two operands with independent limbs.
pub(crate) trait Op2Op {
    /// Combines one limb of each operand.
    fn op(a: u64, b: u64) -> u64;
}

/// Two sources combined limb by limb under `O`.
pub(crate) struct Op2<A, B, O> {
    a: A,
    b: B,
    _op: PhantomData<O>,
}

impl<A, B, O> Op2<A, B, O> {
    #[inline(always)]
    pub(crate) const fn new(a: A, b: B) -> Self {
        Self {
            a,
            b,
            _op: PhantomData,
        }
    }
}

impl<A: LimbSource, B: LimbSource, O: Op2Op> LimbSource for Op2<A, B, O> {
    #[inline(always)]
    fn limb(&self, index: usize) -> u64 {
        O::op(self.a.limb(index), self.b.limb(index))
    }
}

/// Bitwise AND.
pub(crate) struct AndOp;

impl Op2Op for AndOp {
    #[inline(always)]
    fn op(a: u64, b: u64) -> u64 {
        a & b
    }
}

/// Bitwise OR.
pub(crate) struct OrOp;

impl Op2Op for OrOp {
    #[inline(always)]
    fn op(a: u64, b: u64) -> u64 {
        a | b
    }
}

/// Bitwise XOR.
pub(crate) struct XorOp;

impl Op2Op for XorOp {
    #[inline(always)]
    fn op(a: u64, b: u64) -> u64 {
        a ^ b
    }
}

/// The limbs of `!inner`, the two's complement `2^(64 * k) - inner` of `k` limbs once a carry of one
/// enters the bottom limb of the sum that takes them: `a + TwosComplement(b)` with that carry is
/// `a - b + 2^(64 * k)`, whose carry out of the top limb is set exactly when `a >= b`. Past the
/// operand the fill is `u64::MAX`; the collector reads only `ceil(N / 64)` limbs and masks the top
/// one.
pub(crate) struct TwosComplement<S>(pub(crate) S);

impl<S: LimbSource> LimbSource for TwosComplement<S> {
    #[inline(always)]
    fn limb(&self, index: usize) -> u64 {
        !self.0.limb(index)
    }
}

/// `limb << shift` with the bits that `below` sheds into it, for `shift < 64`: the high limb of
/// the two-limb `limb:below` shifted left. Every shift stays below 64 bits, which LLVM matches to
/// one funnel shift (`shld`) wherever the amount comes from. A `u128` shift leaves the bound to the
/// backend, which loses it across basic blocks and tests for amounts of 64 and more.
#[inline(always)]
pub(crate) const fn funnel_left(limb: u64, below: u64, shift: u32) -> u64 {
    let shift = shift & 63;
    (limb << shift) | ((below >> 1) >> (63 - shift))
}

/// `limb >> shift` with the bits that `above` sheds into it, for `shift < 64`: the low limb of
/// the two-limb `above:limb` shifted right, in the form of [`funnel_left`], which LLVM matches to
/// one funnel shift (`shrd`).
#[inline(always)]
pub(crate) const fn funnel_right(limb: u64, above: u64, shift: u32) -> u64 {
    let shift = shift & 63;
    (limb >> shift) | ((above << 1) << (63 - shift))
}

/// The limbs of `inner * 2^shift` for `shift < 64`: each limb takes the bits shifted out of the
/// limb below it, and the bottom limb takes zeros.
pub(crate) struct ShlBits<S> {
    inner: S,
    shift: u32,
}

impl<S> ShlBits<S> {
    #[inline(always)]
    pub(crate) const fn new(inner: S, shift: u32) -> Self {
        Self { inner, shift }
    }
}

impl<S: LimbSource> LimbSource for ShlBits<S> {
    #[inline(always)]
    fn limb(&self, index: usize) -> u64 {
        let below = index
            .checked_sub(1)
            .map_or(0, |below| self.inner.limb(below));
        funnel_left(self.inner.limb(index), below, self.shift)
    }
}
