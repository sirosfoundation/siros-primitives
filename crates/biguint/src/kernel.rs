//! Loop kernels over little-endian limb slices, for the operand sizes that the compiler keeps as
//! loops.
//!
//! A multiply-accumulate step threads one carry limb: `a * b + c + d` is at most `2^128 - 1`, so
//! the carry stays an ordinary register and each step consumes its own flags.

use core::cmp::Ordering;
use core::hint::select_unpredictable;

use crate::limb::{BorrowingSub, CarryingAdd};
use crate::stream::{
    LimbSource, LimbStream, ZeroExtended, for_each_slot, funnel_left, funnel_right,
};

/// `a * b + c + d` as `(low, high)`, as one `u128` expression: the form that LLVM schedules best in
/// straight-line code, where it sees every step of an unrolled product at once.
#[inline(always)]
pub(crate) const fn mul_add(a: u64, b: u64, c: u64, d: u64) -> (u64, u64) {
    let wide = a as u128 * b as u128 + c as u128 + d as u128;
    (wide as u64, (wide >> 64) as u64)
}

/// `a * b + c + d` as `(low, high)`, each addition an add-with-carry into the high limb: the form
/// that LLVM keeps as two `adc` chains inside a loop, where it otherwise sums `c + d` first and
/// saves that carry with `setb`.
#[inline(always)]
pub(crate) fn mul_add_chained(a: u64, b: u64, c: u64, d: u64) -> (u64, u64) {
    let product = a as u128 * b as u128;
    let (low, carry) = (product as u64).add_carry(d, 0);
    let (high, _) = ((product >> 64) as u64).add_carry(0, carry);
    let (low, carry) = low.add_carry(c, 0);
    let (high, _) = high.add_carry(0, carry);
    (low, high)
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
        let (low, high) = mul_add_chained(limb, m, *slot, carry);
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
    let mut source = ZeroExtended::new(a).pull();
    let mut carry = 0;
    for_each_slot(out, |slot| {
        (*slot, carry) = slot.add_carry(source.next_limb(), carry);
    });
    carry
}

/// Subtracts the equally long `a` from `out` and returns the borrow out of the top limb.
#[inline(always)]
pub(crate) fn sub_assign(out: &mut [u64], a: &[u64]) -> u64 {
    let mut source = ZeroExtended::new(a).pull();
    let mut borrow = 0;
    for_each_slot(out, |slot| {
        (*slot, borrow) = slot.sub_borrow(source.next_limb(), borrow);
    });
    borrow
}

/// The borrow out of `a - b` for equally long `a` and `b`, without storing the difference.
#[inline(always)]
pub(crate) fn borrow_of(a: &[u64], b: &[u64]) -> u64 {
    let mut source = ZeroExtended::new(b).pull();
    a.iter().fold(0, |borrow, &limb| {
        limb.sub_borrow(source.next_limb(), borrow).1
    })
}

/// Subtracts `b & mask` from the equally long `out` and returns the borrow out of the top limb.
#[inline(always)]
pub(crate) fn sub_masked(out: &mut [u64], b: &[u64], mask: u64) -> u64 {
    let mut source = ZeroExtended::new(b).pull();
    let mut borrow = 0;
    for_each_slot(out, |slot| {
        (*slot, borrow) = slot.sub_borrow(source.next_limb() & mask, borrow);
    });
    borrow
}

/// Subtracts `divisor` from the equally long `part` when `part >= divisor`, and returns whether
/// it did. The top limbs decide when they differ; equal top limbs take the subtraction unless it
/// borrows, without a branch.
#[inline(always)]
pub(crate) fn subtract_if_above(part: &mut [u64], divisor: &[u64]) -> bool {
    let (Some(&top), Some(&divisor_top)) = (part.last(), divisor.last()) else {
        return false;
    };
    if top < divisor_top {
        false
    } else if top > divisor_top {
        fits(sub_assign(part, divisor));
        true
    } else {
        let above = borrow_of(part, divisor) == 0;
        fits(sub_masked(
            part,
            divisor,
            select_unpredictable(above, u64::MAX, 0),
        ));
        above
    }
}

/// Writes `limbs * 2^shift` to `out`, for `shift < 64`: each limb takes the bits that the limb
/// below it sheds, and the limb above the top one takes the top one's bits when `out` reaches it.
/// The limbs of `out` above those keep their values. Slices pair the limbs, so the loop reads
/// without bounds checks and vectorizes.
#[inline(always)]
pub(crate) fn shl_bits_into(out: &mut [u64], limbs: &[u64], shift: u32) {
    let (Some((bottom, above)), Some((&first, upper))) =
        (out.split_first_mut(), limbs.split_first())
    else {
        return;
    };
    *bottom = first << shift;
    for ((slot, &limb), &below) in above.iter_mut().zip(upper).zip(limbs) {
        *slot = funnel_left(limb, below, shift);
    }
    if let (Some(slot), Some(&top)) = (above.get_mut(upper.len()), limbs.last()) {
        *slot = funnel_left(0, top, shift);
    }
}

/// Writes `floor(limbs / 2^shift)` to `out`, for `shift < 64`: each limb takes the bits that the
/// limb above it sheds. The limbs of `out` past the top one keep their values. Slices pair the
/// limbs, so the loop reads without bounds checks and vectorizes.
#[inline(always)]
pub(crate) fn shr_bits_into(out: &mut [u64], limbs: &[u64], shift: u32) {
    let (Some((&top, lower)), Some((_, upper))) = (limbs.split_last(), limbs.split_first()) else {
        return;
    };
    for ((slot, &limb), &above) in out.iter_mut().zip(lower).zip(upper) {
        *slot = funnel_right(limb, above, shift);
    }
    if let Some(slot) = out.get_mut(lower.len()) {
        *slot = top >> shift;
    }
}

/// Writes `a + b` to `out`, all equally long, and returns the carry out of the top limb.
#[inline(always)]
pub(crate) fn add_to(out: &mut [u64], a: &[u64], b: &[u64]) -> u64 {
    let (mut a, mut b) = (ZeroExtended::new(a).pull(), ZeroExtended::new(b).pull());
    let mut carry = 0;
    for_each_slot(out, |slot| {
        (*slot, carry) = a.next_limb().add_carry(b.next_limb(), carry);
    });
    carry
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

/// Compares `a` with `b`, the shorter one zero-extended, from the most significant limb down to
/// the first that differs.
#[inline(always)]
pub(crate) fn compare(a: &[u64], b: &[u64]) -> Ordering {
    for index in (0..a.len().max(b.len())).rev() {
        let (x, y) = (limb_at(a, index), limb_at(b, index));
        if x != y {
            return x.cmp(&y);
        }
    }
    Ordering::Equal
}

/// The limb of `limbs` at `index`, `0` past its end.
#[inline(always)]
fn limb_at(limbs: &[u64], index: usize) -> u64 {
    limbs.get(index).copied().unwrap_or(0)
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

/// Drops the quotient, `0` or `1`, of a value below twice the divisor that [`subtract_if_above`]
/// brought below the divisor.
#[inline(always)]
pub(crate) fn reduced(_quotient: bool) {}

/// Drops the carry or borrow out of the top of a result that is known to fit: the carries out of
/// the steps that build it cancel.
#[inline(always)]
pub(crate) fn fits(_carry: u64) {}
