//! The limb-stream algebra that every multi-limb operation is written in.
//!
//! An operation is a lazy, infinite stream of little-endian limbs ([`LimbStream`]). A source
//! yields the limbs of a slice and then `0` forever ([`ZeroExtended`]). The zero fill is the end
//! marker, so the hot loop handles plain `u64` limbs. Carries fall out of the algebra: [`AddOp`] on
//! two exhausted operands with a pending carry yields the carry limb. `Uint::from_stream` pulls
//! exactly `ceil(N / 64)` limbs and masks the top one when the operation may set bits above `N`.
//! Every combinator inlines. The compiler unrolls a short pull into straight-line code; a long one
//! runs four limbs per iteration ([`for_each_slot`]).

use core::marker::PhantomData;

use crate::limb::{BorrowingSub, CarryingAdd};

/// An infinite little-endian limb iterator.
pub(crate) trait LimbStream {
    /// The next limb; `0` once the source is exhausted.
    fn next_limb(&mut self) -> u64;
}

/// The limbs of a slice, then `0` forever.
///
/// It reads by index: the index advances with the consumer's loop, so the compiler proves the
/// reads in bounds and drops the zero fill when the slice is at least as long as the output.
pub(crate) struct ZeroExtended<'a> {
    limbs: &'a [u64],
    index: usize,
}

impl<'a> ZeroExtended<'a> {
    #[inline(always)]
    pub(crate) const fn new(limbs: &'a [u64]) -> Self {
        Self { limbs, index: 0 }
    }
}

impl LimbStream for ZeroExtended<'_> {
    #[inline(always)]
    fn next_limb(&mut self) -> u64 {
        let limb = self.limbs.get(self.index).copied().unwrap_or(0);
        self.index += 1;
        limb
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

/// A limb operation on two operands with independent limbs.
pub(crate) trait Op2Op {
    /// Combines one limb of each operand.
    fn op(a: u64, b: u64) -> u64;
}

/// Two streams combined limb by limb under `O`.
pub(crate) struct Op2Iter<A, B, O> {
    a: A,
    b: B,
    _op: PhantomData<O>,
}

impl<A, B, O> Op2Iter<A, B, O> {
    #[inline(always)]
    pub(crate) const fn new(a: A, b: B) -> Self {
        Self {
            a,
            b,
            _op: PhantomData,
        }
    }
}

impl<A: LimbStream, B: LimbStream, O: Op2Op> LimbStream for Op2Iter<A, B, O> {
    #[inline(always)]
    fn next_limb(&mut self) -> u64 {
        O::op(self.a.next_limb(), self.b.next_limb())
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

/// A limb operation on one operand.
pub(crate) trait Op1Op {
    /// Maps one limb.
    fn op(a: u64) -> u64;
}

/// A stream mapped limb by limb under `O`.
pub(crate) struct Op1Iter<A, O> {
    a: A,
    _op: PhantomData<O>,
}

impl<A, O> Op1Iter<A, O> {
    #[inline(always)]
    pub(crate) const fn new(a: A) -> Self {
        Self {
            a,
            _op: PhantomData,
        }
    }
}

impl<A: LimbStream, O: Op1Op> LimbStream for Op1Iter<A, O> {
    #[inline(always)]
    fn next_limb(&mut self) -> u64 {
        O::op(self.a.next_limb())
    }
}

/// The limbs of `!inner`, the two's complement `2^(64 * k) - inner` of `k` limbs once a carry of one
/// enters the bottom limb of the sum that takes them: `a + TwosComplement(b)` with that carry is
/// `a - b + 2^(64 * k)`, whose carry out of the top limb is set exactly when `a >= b`.
pub(crate) struct TwosComplement<I>(pub(crate) I);

impl<I: LimbStream> LimbStream for TwosComplement<I> {
    #[inline(always)]
    fn next_limb(&mut self) -> u64 {
        !self.0.next_limb()
    }
}

/// Bitwise complement. Past the operand the fill becomes `u64::MAX`; the collector reads only
/// `ceil(N / 64)` limbs and masks the top one.
pub(crate) struct NotOp;

impl Op1Op for NotOp {
    #[inline(always)]
    fn op(a: u64) -> u64 {
        !a
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
/// limb below it.
pub(crate) struct ShlBits<I> {
    inner: I,
    shift: u32,
    below: u64,
}

impl<I> ShlBits<I> {
    #[inline(always)]
    pub(crate) const fn new(inner: I, shift: u32) -> Self {
        Self {
            inner,
            shift,
            below: 0,
        }
    }
}

impl<I: LimbStream> LimbStream for ShlBits<I> {
    #[inline(always)]
    fn next_limb(&mut self) -> u64 {
        let limb = self.inner.next_limb();
        let out = funnel_left(limb, self.below, self.shift);
        self.below = limb;
        out
    }
}

/// The limbs of `floor(inner / 2^shift)` for `shift < 64`: each limb takes the bits shifted out of
/// the limb above it.
pub(crate) struct ShrBits<I> {
    inner: I,
    shift: u32,
    current: u64,
}

impl<I: LimbStream> ShrBits<I> {
    #[inline(always)]
    pub(crate) fn new(mut inner: I, shift: u32) -> Self {
        let current = inner.next_limb();
        Self {
            inner,
            shift,
            current,
        }
    }
}

impl<I: LimbStream> LimbStream for ShrBits<I> {
    #[inline(always)]
    fn next_limb(&mut self) -> u64 {
        let above = self.inner.next_limb();
        let out = funnel_right(self.current, above, self.shift);
        self.current = above;
        out
    }
}
