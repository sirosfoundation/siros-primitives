//! The limb-stream algebra that every multi-limb operation is written in.
//!
//! An operation is a lazy, infinite stream of little-endian limbs ([`LimbStream`]). A source
//! yields its limbs and then `0` forever ([`ZeroExtended`]). The zero fill is the end marker, so
//! the hot loop handles plain `u64` limbs. Carries fall out of the algebra: [`AddOp`] on two
//! exhausted operands with a pending carry yields the carry limb, and [`MulIter`] yields its high
//! limb once its operand is exhausted. `Uint::from_stream` pulls exactly `ceil(N / 64)` limbs and
//! masks the top one. Every combinator inlines, and the compiler unrolls the pull into
//! straight-line code over the limbs.

use core::marker::PhantomData;

use crate::limb::{BorrowingSub, CarryingAdd};

/// An infinite little-endian limb iterator.
pub(crate) trait LimbStream {
    /// The next limb; `0` once the source is exhausted.
    fn next_limb(&mut self) -> u64;
}

/// The limbs of a finite iterator, then `0` forever.
pub(crate) struct ZeroExtended<I> {
    iter: I,
}

impl<I> ZeroExtended<I> {
    #[inline(always)]
    pub(crate) const fn new(iter: I) -> Self {
        Self { iter }
    }
}

impl<I: Iterator<Item = u64>> LimbStream for ZeroExtended<I> {
    #[inline(always)]
    fn next_limb(&mut self) -> u64 {
        self.iter.next().unwrap_or(0)
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
        Self {
            a,
            b,
            carry: 0,
            _op: PhantomData,
        }
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

/// Bitwise complement. Past the operand the fill becomes `u64::MAX`; the collector reads only
/// `ceil(N / 64)` limbs and masks the top one.
pub(crate) struct NotOp;

impl Op1Op for NotOp {
    #[inline(always)]
    fn op(a: u64) -> u64 {
        !a
    }
}

/// The product of a run of limbs by the limb `mul`, one limb at a time, with its running carry.
///
/// The one 64x64->128 multiply of the multiplication path lives here. LLVM lowers the `u128`
/// product to `mul` or `mulx`.
pub(crate) struct MulCarry {
    mul: u64,
    carry: u64,
}

impl MulCarry {
    #[inline(always)]
    pub(crate) const fn new(mul: u64) -> Self {
        Self { mul, carry: 0 }
    }

    /// The next product limb: `limb * mul` plus the carry, which takes the high limb.
    #[inline(always)]
    pub(crate) fn step(&mut self, limb: u64) -> u64 {
        let product = limb as u128 * self.mul as u128;
        let (out, overflow) = (product as u64).overflowing_add(self.carry);
        self.carry = (product >> 64) as u64 + u64::from(overflow);
        out
    }
}

/// The schoolbook product `inner * mul`, one limb at a time: each product limb, then the high
/// carry limb, then `0` forever.
pub(crate) struct MulIter<I> {
    inner: I,
    product: MulCarry,
}

impl<I> MulIter<I> {
    #[inline(always)]
    pub(crate) const fn new(inner: I, mul: u64) -> Self {
        Self {
            inner,
            product: MulCarry::new(mul),
        }
    }
}

impl<I: LimbStream> LimbStream for MulIter<I> {
    #[inline(always)]
    fn next_limb(&mut self) -> u64 {
        self.product.step(self.inner.next_limb())
    }
}

/// `limb << shift` with the bits that `below` sheds into it, for `shift < 64`.
#[inline(always)]
pub(crate) const fn funnel_left(limb: u64, below: u64, shift: u32) -> u64 {
    // `(below >> 1) >> (63 - shift)` is `below >> (64 - shift)`, and `0` when `shift` is `0`.
    (limb << shift) | ((below >> 1) >> (63 - shift))
}

/// `limb >> shift` with the bits that `above` sheds into it, for `shift < 64`.
#[inline(always)]
pub(crate) const fn funnel_right(limb: u64, above: u64, shift: u32) -> u64 {
    // `(above << 1) << (63 - shift)` is `above << (64 - shift)`, and `0` when `shift` is `0`.
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

/// Adds `stream` into the little-endian window `into` over its whole length and returns the
/// carry out of the window's top limb.
#[inline(always)]
pub(crate) fn add_into(mut stream: impl LimbStream, into: &mut [u64]) -> u64 {
    let mut carry = 0u64;
    for slot in into {
        (*slot, carry) = (*slot).add_carry(stream.next_limb(), carry);
    }
    carry
}
