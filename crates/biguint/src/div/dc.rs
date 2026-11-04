//! Divide-and-conquer division (Burnikel and Ziegler, 1998), in the form of GMP's
//! `mpn_dcpi1_div_qr`: a `2n`-by-`n` division splits into two divisions by the divisor's upper
//! half and two products of half the length, which Karatsuba computes below the schoolbook's
//! quadratic cost.

use core::marker::PhantomData;

use super::reciprocal::Reciprocal3By2;
use super::{QuotientSink, divide_by_limbs};
use crate::kernel::{add_assign, add_mul_row, split, split_mut, sub_assign, sub_limb};
use crate::mul::basecase;
use crate::mul::karatsuba::{self, blocks, blocks_scratch};
use crate::recursion::{
    Depth, Difference, HalfDown, HalfUp, Len, divide_scratch, exact, exact_mut, prefix_mut,
    schoolbook_of_slices,
};

/// Divisors shorter than this many limbs divide by schoolbook.
pub(crate) const THRESHOLD: usize = 24;

/// [`Depth::divide`] by halves for a divisor of `L` limbs: the upper quotient half from the upper
/// half of the numerator and the divisor's upper half, corrected by the product with the
/// divisor's lower half, then the lower quotient half the same way. Other lengths divide by
/// schoolbook.
#[inline(never)]
pub(crate) fn divide_halves<D: Depth, L: Len>(
    numerator: &mut [u64],
    divisor: &[u64],
    quotient: &mut [u64],
    leading: Reciprocal3By2,
    scratch: &mut [u64],
) -> u64 {
    divide_halves_inlined::<D, L>(numerator, divisor, quotient, leading, scratch)
}

/// [`divide_halves`] inlined into its caller.
#[inline(always)]
pub(crate) fn divide_halves_inlined<D: Depth, L: Len>(
    numerator: &mut [u64],
    divisor: &[u64],
    quotient: &mut [u64],
    leading: Reciprocal3By2,
    scratch: &mut [u64],
) -> u64 {
    let len = L::of(divisor.len());
    let (Some(numerator), Some(divisor), Some(quotient)) = (
        exact_mut(numerator, 2 * len),
        exact(divisor, len),
        exact_mut(quotient, len),
    ) else {
        return schoolbook_of_slices(numerator, divisor, quotient, leading);
    };
    let Some(scratch) = prefix_mut(scratch, divide_scratch(len, D::LEVELS + 1)) else {
        return schoolbook_of_slices(numerator, divisor, quotient, leading);
    };
    let lower = len / 2;
    let upper = len - lower;
    let (product, scratch) = split_mut(scratch, len);
    let (quotient_low, quotient_high) = split_mut(quotient, lower);

    let (divisor_low, divisor_high) = split(divisor, lower);
    let top = {
        let (_, part) = split_mut(numerator, 2 * lower);
        D::divide::<HalfUp<L>>(part, divisor_high, quotient_high, leading, scratch)
    };
    near_product::<D, HalfDown<L>>(product, quotient_high, divisor_low, scratch);
    let top = {
        let (_, window) = split_mut(numerator, lower);
        let (window, _) = split_mut(window, len);
        settle(window, product, divisor, top, upper, quotient_high)
    };

    let (divisor_bottom, divisor_top) = split(divisor, upper);
    let low_top = {
        let (_, part) = split_mut(numerator, upper);
        let (part, _) = split_mut(part, 2 * lower);
        D::divide::<HalfDown<L>>(part, divisor_top, quotient_low, leading, scratch)
    };
    near_product::<D, HalfDown<L>>(product, divisor_bottom, quotient_low, scratch);
    let (window, _) = split_mut(numerator, len);
    let low_top = settle(window, product, divisor, low_top, lower, quotient_low);
    // The corrections bring the lower half below `2^(64 * lower)`.
    lower_half_fits(low_top);
    top
}

/// Drops the top limb of the lower quotient half, which the corrections bring to zero.
#[inline(always)]
fn lower_half_fits(_top: u64) {}

/// Writes `long * short` to `out` for `short` of `Short` limbs and `long` as long or one limb
/// longer.
#[inline(always)]
fn near_product<D: Depth, Short: Len>(
    out: &mut [u64],
    long: &[u64],
    short: &[u64],
    scratch: &mut [u64],
) {
    let (body, extra) = split(long, short.len());
    D::product::<Short>(out, body, short, scratch);
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

/// The quotient limbs of the first block of a `W`-limb window divided by an `L`-limb divisor:
/// the window's quotient limbs modulo `L`, or `L` when `L` divides them.
pub(crate) struct FirstBlock<W, L>(PhantomData<(W, L)>);

impl<W: Len, L: Len> Len for FirstBlock<W, L> {
    const LEN: usize = first_block(W::LEN, L::LEN);
    const ERASED: bool = W::ERASED || L::ERASED;
}

/// The quotient limbs of the first block of a `window`-limb window divided by a `len`-limb
/// divisor, as for [`FirstBlock`].
#[inline(always)]
const fn first_block(window: usize, len: usize) -> usize {
    let total = window.saturating_sub(len);
    if total == 0 {
        0
    } else {
        total - (total - 1) / len * len
    }
}

/// Leaves `window mod divisor` in the low `L` limbs of the `W`-limb `window` and zeros above
/// them, and gives the quotient limbs to `quotient`: blocks of `L` quotient limbs from the top,
/// the first block shorter when the quotient length is not a multiple. The top `L` limbs of
/// `window` are below the `L`-limb `divisor`. Other lengths divide by schoolbook.
#[inline(always)]
pub(super) fn divide_window<D: Depth, W: Len, L: Len>(
    window: &mut [u64],
    divisor: &[u64],
    leading: Reciprocal3By2,
    quotient: &mut (impl QuotientSink + ?Sized),
    scratch: &mut [u64],
) {
    let (width, len) = (W::of(window.len()), L::of(divisor.len()));
    let (Some(window), Some(divisor), Some(scratch)) = (
        exact_mut(window, width),
        exact(divisor, len),
        prefix_mut(scratch, window_scratch(len, D::LEVELS)),
    ) else {
        return divide_by_limbs::<false>(window, divisor, leading, quotient);
    };
    let total = width.saturating_sub(len);
    if total == 0 {
        return;
    }
    let (block_quotient, scratch) = split_mut(scratch, len);
    let first = FirstBlock::<W, L>::of(first_block(width, len));
    let mut position = total - first;
    divide_block::<D, L, FirstBlock<W, L>>(
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
        divide_block::<D, L, L>(
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

/// Divides the `Count + L` limbs of `window` from limb `position` by the `L`-limb `divisor`,
/// their top `L` limbs below it: `Count` quotient limbs into `block_quotient`. `count` gives an
/// erased `Count`.
#[inline(always)]
fn divide_block<D: Depth, L: Len, Count: Len>(
    window: &mut [u64],
    position: usize,
    count: usize,
    divisor: &[u64],
    block_quotient: &mut [u64],
    leading: Reciprocal3By2,
    scratch: &mut [u64],
) {
    let len = L::of(divisor.len());
    let count = Count::of(count);
    let (_, part) = split_mut(window, position);
    let (part, _) = split_mut(part, count + len);
    let (quotient, _) = split_mut(block_quotient, count);
    if count == len {
        quotient_fits(D::divide::<L>(part, divisor, quotient, leading, scratch));
    } else if count >= THRESHOLD && scratch.len() >= partial_scratch(len, count, D::LEVELS) {
        // The top `2 * count` limbs by the divisor's top `count` limbs, then the correction by
        // the product with the divisor's other limbs.
        let (divisor_low, divisor_top) = split(divisor, len - count);
        let (product, scratch) = split_mut(scratch, len);
        let top = {
            let (_, top_part) = split_mut(part, len - count);
            D::divide::<Count>(top_part, divisor_top, quotient, leading, scratch)
        };
        any_product::<D, Count, Difference<L, Count>>(product, quotient, divisor_low, scratch);
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

/// Writes `a * b` to `out[..A + B]` for `a` of `A` and `b` of `B` limbs.
#[inline(always)]
fn any_product<D: Depth, A: Len, B: Len>(
    out: &mut [u64],
    a: &[u64],
    b: &[u64],
    scratch: &mut [u64],
) {
    let (m, k) = (A::of(a.len()), B::of(b.len()));
    if m.min(k) < karatsuba::THRESHOLD {
        basecase(out, a, b);
    } else if m == k {
        D::product::<A>(out, a, b, scratch);
    } else if m > k {
        blocks::<D, B, false>(out, a, b, scratch);
    } else {
        blocks::<D, A, false>(out, b, a, scratch);
    }
}
