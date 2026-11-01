//! Loop kernels over little-endian limb slices, for the operand sizes that the compiler keeps as
//! loops.
//!
//! A multiply-accumulate step threads one carry limb: `a * b + c + d` is at most `2^128 - 1`, so
//! the carry stays an ordinary register across iterations. The stream algebra threads two carries
//! through the flags register, which serves fully unrolled code and costs a flag save and restore
//! per limb inside a loop.

use core::cmp::Ordering;

use crate::limb::{BorrowingSub, CarryingAdd};

/// `a * b + c + d` as `(low, high)`.
#[inline(always)]
pub(crate) const fn mul_add(a: u64, b: u64, c: u64, d: u64) -> (u64, u64) {
    let wide = a as u128 * b as u128 + c as u128 + d as u128;
    (wide as u64, (wide >> 64) as u64)
}

/// Writes `a * m` to the first `a.len()` limbs of `out` and returns the limb above them.
#[inline(always)]
pub(crate) fn mul_row(out: &mut [u64], a: &[u64], m: u64) -> u64 {
    out.iter_mut().zip(a).fold(0, |carry, (slot, &limb)| {
        let (low, high) = mul_add(limb, m, carry, 0);
        *slot = low;
        high
    })
}

/// Adds `a * m` to the first `a.len()` limbs of `out` and returns the carry limb above them.
#[inline(always)]
pub(crate) fn add_mul_row(out: &mut [u64], a: &[u64], m: u64) -> u64 {
    out.iter_mut().zip(a).fold(0, |carry, (slot, &limb)| {
        let (low, high) = mul_add(limb, m, *slot, carry);
        *slot = low;
        high
    })
}

/// Subtracts `a * m` from the first `a.len()` limbs of `out` and returns the limb that the
/// subtraction owes above them.
#[inline(always)]
pub(crate) fn sub_mul_row(out: &mut [u64], a: &[u64], m: u64) -> u64 {
    out.iter_mut().zip(a).fold(0, |borrow, (slot, &limb)| {
        let (low, high) = mul_add(limb, m, borrow, 0);
        let (difference, owed) = slot.overflowing_sub(low);
        *slot = difference;
        // `high` reaches `2^64 - 1` only with `low = 0`, which owes nothing.
        high + u64::from(owed)
    })
}

/// Adds the equally long `a` to `out` and returns the carry out of the top limb.
#[inline(always)]
pub(crate) fn add_assign(out: &mut [u64], a: &[u64]) -> u64 {
    out.iter_mut().zip(a).fold(0, |carry, (slot, &limb)| {
        let (sum, carry) = slot.add_carry(limb, carry);
        *slot = sum;
        carry
    })
}

/// Subtracts the equally long `a` from `out` and returns the borrow out of the top limb.
#[inline(always)]
pub(crate) fn sub_assign(out: &mut [u64], a: &[u64]) -> u64 {
    out.iter_mut().zip(a).fold(0, |borrow, (slot, &limb)| {
        let (difference, borrow) = slot.sub_borrow(limb, borrow);
        *slot = difference;
        borrow
    })
}

/// Writes `a + b` to `out`, all equally long, and returns the carry out of the top limb.
#[inline(always)]
pub(crate) fn add_to(out: &mut [u64], a: &[u64], b: &[u64]) -> u64 {
    out.iter_mut()
        .zip(a.iter().zip(b))
        .fold(0, |carry, (slot, (&x, &y))| {
            let (sum, carry) = x.add_carry(y, carry);
            *slot = sum;
            carry
        })
}

/// Adds `carry` to the bottom limb of `out`, ripples it up until it is absorbed, and returns the
/// carry out of the top.
#[inline(always)]
pub(crate) fn add_limb(out: &mut [u64], carry: u64) -> u64 {
    let mut carry = carry;
    for slot in out {
        if carry == 0 {
            break;
        }
        let (sum, overflow) = slot.overflowing_add(carry);
        *slot = sum;
        carry = u64::from(overflow);
    }
    carry
}

/// Subtracts `borrow` from the bottom limb of `out`, ripples it up until it is absorbed, and
/// returns the borrow out of the top.
#[inline(always)]
pub(crate) fn sub_limb(out: &mut [u64], borrow: u64) -> u64 {
    let mut borrow = borrow;
    for slot in out {
        if borrow == 0 {
            break;
        }
        let (difference, underflow) = slot.overflowing_sub(borrow);
        *slot = difference;
        borrow = u64::from(underflow);
    }
    borrow
}

/// Compares `a` with `b`, the shorter one zero-extended.
#[inline(always)]
pub(crate) fn compare(a: &[u64], b: &[u64]) -> Ordering {
    let len = a.len().max(b.len());
    from_top(a, len).cmp(from_top(b, len))
}

/// The limbs of `limbs` zero-extended to `len`, from the most significant.
#[inline(always)]
fn from_top(limbs: &[u64], len: usize) -> impl Iterator<Item = u64> + '_ {
    core::iter::repeat_n(0, len.saturating_sub(limbs.len())).chain(limbs.iter().rev().copied())
}

/// `slice.split_at(mid)` with `mid` clamped to the length.
#[inline(always)]
pub(crate) fn split(slice: &[u64], mid: usize) -> (&[u64], &[u64]) {
    slice.split_at(mid.min(slice.len()))
}

/// `slice.split_at_mut(mid)` with `mid` clamped to the length.
#[inline(always)]
pub(crate) fn split_mut(slice: &mut [u64], mid: usize) -> (&mut [u64], &mut [u64]) {
    let mid = mid.min(slice.len());
    slice.split_at_mut(mid)
}

/// Drops the carry or borrow out of the top of a result that is known to fit: the carries out of
/// the steps that build it cancel.
#[inline(always)]
pub(crate) fn fits(_carry: u64) {}
