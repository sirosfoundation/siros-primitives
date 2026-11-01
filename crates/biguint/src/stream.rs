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

/// The schoolbook product `inner * mul`, one limb at a time: each product limb, then the high
/// carry limb, then `0` forever.
///
/// The one 64x64->128 multiply of the multiplication path lives here. LLVM lowers the `u128`
/// product to `mul` or `mulx`.
pub(crate) struct MulIter<I> {
    inner: I,
    mul: u64,
    carry: u64,
}

impl<I> MulIter<I> {
    #[inline(always)]
    pub(crate) const fn new(inner: I, mul: u64) -> Self {
        Self {
            inner,
            mul,
            carry: 0,
        }
    }
}

impl<I: LimbStream> LimbStream for MulIter<I> {
    #[inline(always)]
    fn next_limb(&mut self) -> u64 {
        let product = self.inner.next_limb() as u128 * self.mul as u128;
        let (out, overflow) = (product as u64).overflowing_add(self.carry);
        self.carry = (product >> 64) as u64 + u64::from(overflow);
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
