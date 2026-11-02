//! Divide-and-conquer division (Burnikel and Ziegler, 1998), in the form of GMP's
//! `mpn_dcpi1_div_qr`: a `2n`-by-`n` division splits into two divisions by the divisor's upper
//! half and two products of half the length, which Karatsuba computes below the schoolbook's
//! quadratic cost.

use super::reciprocal::Reciprocal3By2;
use super::{QuotientSink, divide_by_limbs};
use crate::kernel::{add_assign, add_mul_row, split, split_mut, sub_assign, sub_limb};
use crate::mul::basecase;
use crate::mul::karatsuba::{self, blocks, blocks_scratch};
use crate::recursion::{Depth, divide_scratch};

/// Divisors shorter than this many limbs divide by schoolbook.
pub(crate) const THRESHOLD: usize = 24;

/// [`Depth::divide`] by halves: the upper quotient half from the upper half of the numerator and
/// the divisor's upper half, corrected by the product with the divisor's lower half, then the
/// lower quotient half the same way.
#[inline(always)]
pub(crate) fn divide_halves<D: Depth>(
    numerator: &mut [u64],
    divisor: &[u64],
    quotient: &mut [u64],
    leading: Reciprocal3By2,
    scratch: &mut [u64],
) -> u64 {
    let len = divisor.len();
    let lower = len / 2;
    let upper = len - lower;
    let (product, scratch) = split_mut(scratch, len);
    let (quotient_low, quotient_high) = split_mut(quotient, lower);

    let (divisor_low, divisor_high) = split(divisor, lower);
    let top = {
        let (_, part) = split_mut(numerator, 2 * lower);
        D::divide(part, divisor_high, quotient_high, leading, scratch)
    };
    near_product::<D>(product, quotient_high, divisor_low, scratch);
    let top = {
        let (_, window) = split_mut(numerator, lower);
        let (window, _) = split_mut(window, len);
        settle(window, product, divisor, top, upper, quotient_high)
    };

    let (divisor_bottom, divisor_top) = split(divisor, upper);
    let low_top = {
        let (_, part) = split_mut(numerator, upper);
        let (part, _) = split_mut(part, 2 * lower);
        D::divide(part, divisor_top, quotient_low, leading, scratch)
    };
    near_product::<D>(product, divisor_bottom, quotient_low, scratch);
    let (window, _) = split_mut(numerator, len);
    let low_top = settle(window, product, divisor, low_top, lower, quotient_low);
    // The corrections bring the lower half below `2^(64 * lower)`.
    lower_half_fits(low_top);
    top
}

/// Drops the top limb of the lower quotient half, which the corrections bring to zero.
#[inline(always)]
fn lower_half_fits(_top: u64) {}

/// Writes `long * short` to `out` for `long` as long as `short` or one limb longer.
#[inline(always)]
fn near_product<D: Depth>(out: &mut [u64], long: &[u64], short: &[u64], scratch: &mut [u64]) {
    let (body, extra) = split(long, short.len());
    D::product(out, body, short, scratch);
    if let Some(&top) = extra.first() {
        let (_, window) = split_mut(out, short.len());
        let (row, above) = split_mut(window, short.len());
        let carry = add_mul_row(row, short, top);
        if let Some(slot) = above.first_mut() {
            *slot = carry;
        }
    }
}

/// Subtracts `product` from `window`, as long as `divisor`, and the low part of `divisor` at limb
/// `shift` when the top quotient limb `top` is set; then adds `divisor` back while the window is
/// negative, one less in `quotient` for each. Returns the corrected top quotient limb.
#[inline(always)]
fn settle(
    window: &mut [u64],
    product: &[u64],
    divisor: &[u64],
    top: u64,
    shift: usize,
    quotient: &mut [u64],
) -> u64 {
    let mut borrow = sub_assign(window, product);
    if top != 0 {
        let (_, high) = split_mut(window, shift);
        let (divisor_low, _) = split(divisor, high.len());
        borrow += sub_assign(high, divisor_low);
    }
    let mut top = top;
    // At most twice.
    while borrow != 0 {
        top -= sub_limb(quotient, 1);
        borrow -= add_assign(window, divisor);
    }
    top
}

/// The scratch limbs of [`divide_window`] for a `len`-limb divisor with `levels` halvings left.
#[inline(always)]
const fn window_scratch(len: usize, levels: usize) -> usize {
    len + divide_scratch(len, levels)
}

/// Leaves `window mod divisor` in the low `divisor.len()` limbs of `window` and zeros above them,
/// and gives the quotient limbs to `quotient`: blocks of `divisor.len()` quotient limbs from the
/// top, the first block shorter when the quotient length is not a multiple. The top
/// `divisor.len()` limbs of `window` are below `divisor`. An unoptimized build calls it, as it does
/// the product kernels.
#[cfg_attr(debug_assertions, inline(never))]
#[cfg_attr(not(debug_assertions), inline(always))]
pub(super) fn divide_window<D: Depth>(
    window: &mut [u64],
    divisor: &[u64],
    leading: Reciprocal3By2,
    quotient: &mut (impl QuotientSink + ?Sized),
    scratch: &mut [u64],
) {
    let len = divisor.len();
    if scratch.len() < window_scratch(len, D::LEVELS) {
        divide_by_limbs::<false>(window, divisor, leading, quotient);
        return;
    }
    let total = window.len().saturating_sub(len);
    if total == 0 {
        return;
    }
    let (block_quotient, scratch) = split_mut(scratch, len);
    let first = total - (total - 1) / len * len;
    let mut position = total - first;
    divide_block::<D>(
        window,
        position,
        first,
        divisor,
        block_quotient,
        leading,
        scratch,
    );
    emit(quotient, position, split(block_quotient, first).0);
    while position >= len {
        position -= len;
        divide_block::<D>(
            window,
            position,
            len,
            divisor,
            block_quotient,
            leading,
            scratch,
        );
        emit(quotient, position, block_quotient);
    }
}

/// Gives `limbs` to `quotient` as the quotient limbs from `position` up.
#[inline(always)]
fn emit(quotient: &mut (impl QuotientSink + ?Sized), position: usize, limbs: &[u64]) {
    for (index, &limb) in limbs.iter().enumerate() {
        quotient.put(position + index, limb);
    }
}

/// Divides the `count + divisor.len()` limbs of `window` from limb `position` by `divisor`, their
/// top `divisor.len()` limbs below it: `count` quotient limbs into `block_quotient`.
#[inline(always)]
fn divide_block<D: Depth>(
    window: &mut [u64],
    position: usize,
    count: usize,
    divisor: &[u64],
    block_quotient: &mut [u64],
    leading: Reciprocal3By2,
    scratch: &mut [u64],
) {
    let len = divisor.len();
    let (_, part) = split_mut(window, position);
    let (part, _) = split_mut(part, count + len);
    let (quotient, _) = split_mut(block_quotient, count);
    if count == len {
        quotient_fits(D::divide(part, divisor, quotient, leading, scratch));
    } else if count >= THRESHOLD && scratch.len() >= partial_scratch(len, count, D::LEVELS) {
        // The top `2 * count` limbs by the divisor's top `count` limbs, then the correction by
        // the product with the divisor's other limbs.
        let (divisor_low, divisor_top) = split(divisor, len - count);
        let (product, scratch) = split_mut(scratch, len);
        let top = {
            let (_, top_part) = split_mut(part, len - count);
            D::divide(top_part, divisor_top, quotient, leading, scratch)
        };
        any_product::<D>(product, quotient, divisor_low, scratch);
        let (window, _) = split_mut(part, len);
        quotient_fits(settle(window, product, divisor, top, count, quotient));
    } else {
        divide_by_limbs::<false>(part, divisor, leading, quotient);
    }
}

/// Drops the top quotient limb of a block whose top `divisor.len()` limbs are below the divisor:
/// it is zero.
#[inline(always)]
fn quotient_fits(_top: u64) {}

/// The scratch limbs of a block of `count < len` quotient limbs: the correction product, then the
/// larger of the division by the divisor's top `count` limbs and the product's own scratch.
#[inline(always)]
const fn partial_scratch(len: usize, count: usize, levels: usize) -> usize {
    let division = divide_scratch(count, levels);
    let shorter = if count < len - count {
        count
    } else {
        len - count
    };
    let product = blocks_scratch(shorter, levels);
    len + if division > product {
        division
    } else {
        product
    }
}

/// Writes `a * b` to `out[..a.len() + b.len()]` for operands of any lengths.
#[inline(always)]
fn any_product<D: Depth>(out: &mut [u64], a: &[u64], b: &[u64], scratch: &mut [u64]) {
    let (long, short) = if a.len() >= b.len() { (a, b) } else { (b, a) };
    if short.len() < karatsuba::THRESHOLD {
        basecase(out, long, short);
    } else if long.len() == short.len() {
        D::product(out, long, short, scratch);
    } else {
        blocks::<D>(out, long, short, scratch);
    }
}
