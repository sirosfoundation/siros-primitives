//! Loop kernels over little-endian limb slices, for the operand sizes that the compiler keeps as
//! loops.
//!
//! A multiply-accumulate step threads one carry limb: `a * b + c + d` is at most `2^128 - 1`, so
//! the carry stays an ordinary register and each step consumes its own flags.

use core::cmp::Ordering;
use core::hint::select_unpredictable;

use crate::limb::{BorrowingSub, CarryingAdd, mask_below};
use crate::stream::{
    LimbSource, LimbStream, Op2CarryIter, SubOp, ZeroExtended, for_each_slot, funnel_left,
    funnel_right,
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
/// subtraction owes above them, one limb per loop step. A length that the compiler knows takes
/// [`sub_mul_row_chains`].
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

/// The limbs of a group of [`sub_mul_row_chains`], whose products stay in registers.
const CHAIN_GROUP: usize = 4;

/// [`sub_mul_row`] in two carry chains of one limb of latency each: the row of products, each
/// low half plus the high half before it, and the subtraction of that row from `out`, in groups of
/// [`CHAIN_GROUP`] limbs. A length that the compiler knows unrolls the groups; one that it does not
/// know runs them limb by limb, slower than [`sub_mul_row`], which also takes rows shorter than a
/// group.
#[inline(always)]
pub(crate) fn sub_mul_row_chains(out: &mut [u64], a: &[u64], m: u64) -> u64 {
    if a.len() < CHAIN_GROUP {
        return sub_mul_row(out, a, m);
    }
    let (mut high, mut carry, mut borrow) = (0, 0, 0);
    for (slots, limbs) in out.chunks_mut(CHAIN_GROUP).zip(a.chunks(CHAIN_GROUP)) {
        let mut row = [0u64; CHAIN_GROUP];
        for (product_slot, &limb) in row.iter_mut().zip(limbs) {
            let product = u128::from(limb) * u128::from(m);
            (*product_slot, carry) = (product as u64).add_carry(high, carry);
            high = (product >> 64) as u64;
        }
        for (slot, &product) in slots.iter_mut().zip(row.iter().take(limbs.len())) {
            (*slot, borrow) = slot.sub_borrow(product, borrow);
        }
    }
    // `a * m` is the row plus `high + carry` above it, so the subtraction owes `high + carry`
    // and its borrow: at most `m`, the most that `a * m` reaches above `a`'s limbs.
    high + carry + borrow
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

/// The most limbs whose correction of a whole operand runs without a branch on the values. A
/// branch on the values mispredicts on about half the operands, at a fixed cost; the branch-free
/// form runs one more carry chain, whose time grows with the length. Up to this length, the chain
/// costs less.
pub(crate) const BRANCH_FREE_LIMBS: usize = 8;

/// Subtracts `divisor` from the equally long `part` when `part >= divisor`, and returns whether
/// it did. The top limbs decide when they differ, and the limbs below them when they are equal,
/// which is rare. Up to [`BRANCH_FREE_LIMBS`] limbs, the subtraction runs under a mask
/// ([`mask_below`]); longer, it runs when the top limbs call for it.
#[inline(always)]
pub(crate) fn subtract_if_above(part: &mut [u64], divisor: &[u64]) -> bool {
    let (Some(&top), Some(&divisor_top)) = (part.last(), divisor.last()) else {
        return false;
    };
    if divisor.len() > BRANCH_FREE_LIMBS && top != divisor_top {
        let above = top > divisor_top;
        if above {
            fits(sub_assign(part, divisor));
        }
        return above;
    }
    let above = if top == divisor_top {
        // No borrow: the borrow is below one.
        mask_below(borrow_of(part, divisor), 1)
    } else {
        mask_below(divisor_top, top)
    };
    fits(sub_masked(part, divisor, above));
    above != 0
}

/// Writes `part - divisor` to `out` when `carry` is set or `part >= divisor`, `part` otherwise, and
/// returns whether it subtracted. A set `carry` stands for a part of at least `2^(64 * k)`, whose
/// borrow cancels it. `out` sets `k`; `part` and `divisor` are at least as long. Up to
/// [`BRANCH_FREE_LIMBS`] limbs, the difference runs into `out`, and its borrow decides whether
/// `out` takes `part` back, by an unpredictable select of every limb; longer, `out` takes `part`
/// and [`subtract_if_above`] decides.
#[inline(always)]
pub(crate) fn subtract_when_above(
    out: &mut [u64],
    part: &[u64],
    divisor: &[u64],
    carry: u64,
) -> bool {
    copy(out, part);
    subtract_when_above_in_place(out, divisor, carry)
}

/// Subtracts `divisor` from `part` when `carry` is set or `part >= divisor`, and returns whether
/// it did. A set `carry` stands for a part of at least `2^(64 * k)`, whose borrow cancels it.
/// `part` sets `k`; `divisor` is at least as long. Up to [`BRANCH_FREE_LIMBS`] limbs, the
/// difference runs into registers, and its borrow decides whether `part` takes it, by an
/// unpredictable select of every limb; longer, [`subtract_if_above`] decides.
#[inline(always)]
pub(crate) fn subtract_when_above_in_place(part: &mut [u64], divisor: &[u64], carry: u64) -> bool {
    if part.len() > BRANCH_FREE_LIMBS {
        if carry != 0 {
            fits(sub_assign(part, divisor));
            return true;
        }
        return subtract_if_above(part, divisor);
    }
    let mut difference = [0u64; BRANCH_FREE_LIMBS];
    let (difference, _) = split_mut(&mut difference, part.len());
    let borrow = sub_into(difference, part, divisor);
    let take = (carry != 0) | (borrow == 0);
    for (slot, &limb) in part.iter_mut().zip(difference.iter()) {
        // A conditional move: the unpredictable select keeps LLVM from branching inside a loop.
        *slot = select_unpredictable(take, limb, *slot);
    }
    take
}

/// Adds `addend` to `part` modulo `2^(64 * k)` when `carry` is zero. `part` sets `k`; `addend` is
/// at least as long. Up to [`BRANCH_FREE_LIMBS`] limbs, the addition runs under a mask
/// ([`mask_below`]); longer, it runs when `carry` calls for it.
#[inline(always)]
pub(crate) fn add_unless_carry(part: &mut [u64], addend: &[u64], carry: u64) {
    if part.len() > BRANCH_FREE_LIMBS {
        if carry == 0 {
            fits(add_assign(part, addend));
        }
        return;
    }
    // All ones without the carry.
    let mask = mask_below(carry, 1);
    let mut addend = ZeroExtended::new(addend).pull();
    let mut carry = 0;
    for_each_slot(part, |slot| {
        (*slot, carry) = slot.add_carry(addend.next_limb() & mask, carry);
    });
}

/// Copies the limbs of `limbs` that `out` holds.
#[inline(always)]
fn copy(out: &mut [u64], limbs: &[u64]) {
    for (slot, &limb) in out.iter_mut().zip(limbs) {
        *slot = limb;
    }
}

/// Writes `a - b` to `out` and returns the borrow out of its top limb. `out` sets the length; `a`
/// and `b` are at least as long.
#[inline(always)]
pub(crate) fn sub_into(out: &mut [u64], a: &[u64], b: &[u64]) -> u64 {
    let mut difference =
        Op2CarryIter::<_, _, SubOp>::new(ZeroExtended::new(a).pull(), ZeroExtended::new(b).pull());
    for_each_slot(out, |slot| *slot = difference.next_limb());
    difference.carry()
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
/// carry out of the top. The bottom limb takes the carry whatever its value, so only the rare
/// overflow branches: a test for a zero carry mispredicts on carries that the values decide.
#[inline(always)]
pub(crate) fn add_limb(out: &mut [u64], carry: u64) -> u64 {
    let Some((bottom, rest)) = out.split_first_mut() else {
        return carry;
    };
    let (sum, overflow) = bottom.overflowing_add(carry);
    *bottom = sum;
    let mut carry = u64::from(overflow);
    for slot in rest {
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
/// returns the borrow out of the top. The bottom limb takes the borrow whatever its value, as in
/// [`add_limb`].
#[inline(always)]
pub(crate) fn sub_limb(out: &mut [u64], borrow: u64) -> u64 {
    let Some((bottom, rest)) = out.split_first_mut() else {
        return borrow;
    };
    let (difference, underflow) = bottom.overflowing_sub(borrow);
    *bottom = difference;
    let mut borrow = u64::from(underflow);
    for slot in rest {
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
