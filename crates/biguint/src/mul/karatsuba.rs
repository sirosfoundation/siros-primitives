//! Karatsuba multiplication (Karatsuba and Ofman, 1962): three half-size products in place of
//! four, in the subtractive form with the in-place interpolation of GMP's `mpn_toom22_mul`.

use core::hint::select_unpredictable;

use crate::kernel::{
    add_assign, add_flipped, add_limb, add_limb_masked, add_to, compare, fits, mask_of,
    negate_masked, split, split_mut, sub_assign, sub_into, sub_limb, sub_limb_masked,
};
use crate::recursion::{
    Depth, HalfDown, HalfUp, Len, basecase_of_slices, exact, prefix_mut, product_scratch,
};
use crate::stream::{LimbSource, LimbStream, Op2CarryIter, SubOp, ZeroExtended};

use super::basecase;

/// Operands shorter than this many limbs multiply by the quadratic product.
pub(crate) const THRESHOLD: usize = 16;

/// Writes `a * b` to `out[..2n]` for `a` and `b` of `n = L` limbs, with the sub-products at depth
/// `D`. `scratch` holds `product_scratch(n, D::LEVELS + 1)` limbs; other lengths run the
/// quadratic product.
///
/// With `a = a0 + a1 * β^h` and `b = b0 + b1 * β^h`, `h = ceil(n / 2)`, the product is
/// `v0 + (v0 + vinf - vm1) * β^h + vinf * β^2h` for `v0 = a0 * b0`, `vinf = a1 * b1` and
/// `vm1 = (a0 - a1) * (b0 - b1)`.
///
/// `MASKED` takes the signs of the differences and the carries under masks: no branch and no
/// memory address depends on the operands.
#[inline(never)]
pub(crate) fn karatsuba<D: Depth, L: Len, const MASKED: bool>(
    out: &mut [u64],
    a: &[u64],
    b: &[u64],
    scratch: &mut [u64],
) {
    karatsuba_inlined::<D, L, MASKED>(out, a, b, scratch);
}

/// [`karatsuba`] inlined into its caller.
#[inline(always)]
pub(crate) fn karatsuba_inlined<D: Depth, L: Len, const MASKED: bool>(
    out: &mut [u64],
    a: &[u64],
    b: &[u64],
    scratch: &mut [u64],
) {
    let n = L::of(a.len());
    let (Some(a), Some(b)) = (exact(a, n), exact(b, n)) else {
        return basecase_of_slices(out, a, b);
    };
    let (Some(out), Some(scratch)) = (
        prefix_mut(out, 2 * n),
        prefix_mut(scratch, product_scratch(n, D::LEVELS + 1)),
    ) else {
        return basecase_of_slices(out, a, b);
    };
    let half = n - n / 2;
    let (a_low, a_high) = split(a, half);
    let (b_low, b_high) = split(b, half);
    let (difference_product, scratch) = split_mut(scratch, 2 * half);
    let negative = {
        // `|a0 - a1|` and `|b0 - b1|` wait in the limbs that `v0` fills last.
        let (lower, _) = split_mut(out, 2 * half);
        let (a_difference, b_difference) = split_mut(lower, half);
        let negative = abs_difference::<MASKED>(a_difference, a_low, a_high)
            ^ abs_difference::<MASKED>(b_difference, b_low, b_high);
        D::product_with::<HalfUp<L>, MASKED>(
            difference_product,
            a_difference,
            b_difference,
            scratch,
        );
        negative
    };
    {
        let (lower, upper) = split_mut(out, 2 * half);
        // Equal halves run one function.
        if L::ERASED || n.is_multiple_of(2) {
            D::product_with::<HalfUp<L>, MASKED>(upper, a_high, b_high, scratch);
        } else {
            D::product_with::<HalfDown<L>, MASKED>(upper, a_high, b_high, scratch);
        }
        D::product_with::<HalfUp<L>, MASKED>(lower, a_low, b_low, scratch);
    }
    interpolate::<MASKED>(out, difference_product, negative, half);
}

/// Writes `|x - y|` to `out`, as long as `x`, and returns `1` when `x < y`, `0` otherwise. `y` is
/// at most as long as `x`. `MASKED` negates the difference under the mask of its borrow; otherwise
/// a comparison picks the operands.
#[inline(always)]
fn abs_difference<const MASKED: bool>(out: &mut [u64], x: &[u64], y: &[u64]) -> u64 {
    if MASKED {
        let borrow = sub_into(out, x, y);
        negate_masked(out, mask_of(borrow));
        return borrow;
    }
    let below = compare(x, y).is_lt();
    let (larger, smaller) = select_unpredictable(below, (y, x), (x, y));
    let mut difference = Op2CarryIter::<_, _, SubOp>::new(
        ZeroExtended::new(larger).pull(),
        ZeroExtended::new(smaller).pull(),
    );
    for slot in out {
        *slot = difference.next_limb();
    }
    u64::from(below)
}

/// Turns `out = v0 | vinf` into the product, given `|vm1|` and the sign of `vm1`. With
/// `v0 = L0 | H0` and `vinf = L1 | H1` split at `half` limbs, the product is
/// `L0 + (H0 + L0 + L1) β^h + (H0 + L1 + H1) β^2h + H1 β^3h - vm1 β^h`. `negative` is `1` when
/// `vm1` is negative. `MASKED` adds or subtracts `|vm1|` under a mask and ripples every carry
/// through every limb.
#[inline(always)]
fn interpolate<const MASKED: bool>(
    out: &mut [u64],
    difference_product: &[u64],
    negative: u64,
    half: usize,
) {
    let (carry_at_2h, mut carry_at_3h) = {
        let (v0, vinf) = split_mut(out, 2 * half);
        let (l0, h0) = split_mut(v0, half);
        let (l1, h1) = split_mut(vinf, half);
        // X = H0 + L1 replaces L1; X + L0 replaces H0; X + H1 replaces X.
        let carry_x = add_assign(l1, h0);
        let carry_at_2h = carry_x + add_to(h0, l1, l0);
        let (l1_low, l1_high) = split_mut(l1, h1.len());
        let carry_at_3h = carry_x + carry_limb::<MASKED>(l1_high, add_assign(l1_low, h1));
        (carry_at_2h, carry_at_3h)
    };
    let (_, from_h) = split_mut(out, half);
    let borrow_at_3h = {
        let (middle, _) = split_mut(from_h, 2 * half);
        if MASKED {
            // `middle + dp` when negative, `middle + !dp + 1 = middle - dp + 2^(64 k)` otherwise:
            // the carry out of the sum is the carry of the addition, or one less than the borrow
            // of the subtraction.
            let mask = mask_of(negative);
            let carry = add_flipped(middle, difference_product, !mask);
            carry_at_3h += carry & mask;
            (carry ^ 1) & !mask & 1
        } else if negative != 0 {
            carry_at_3h += add_assign(middle, difference_product);
            0
        } else {
            sub_assign(middle, difference_product)
        }
    };
    // The carries and the borrow cancel to a product that fits in `out`.
    let (_, from_2h) = split_mut(from_h, half);
    fits(carry_limb::<MASKED>(from_2h, carry_at_2h));
    let (_, from_3h) = split_mut(from_2h, half);
    fits(carry_limb::<MASKED>(from_3h, carry_at_3h));
    fits(borrow_limb::<MASKED>(from_3h, borrow_at_3h));
}

/// The scratch limbs of [`blocks`] for a `len`-limb shorter operand: a block product and the
/// scratch of the product that fills it.
#[inline(always)]
pub(crate) const fn blocks_scratch(len: usize, levels: usize) -> usize {
    2 * len + product_scratch(len, levels)
}

/// Writes `long * short` to `out[..long.len() + short.len()]` for `long.len() >= short.len()`,
/// `short` of `Short` limbs: products of `short` with the blocks of `long` that are as long as
/// `short`, added at their offsets, then the shorter last block by the quadratic product. Without
/// `blocks_scratch` limbs of `scratch`, the whole product runs as the quadratic product.
/// `MASKED` as for [`karatsuba`].
#[inline(always)]
pub(crate) fn blocks<D: Depth, Short: Len, const MASKED: bool>(
    out: &mut [u64],
    long: &[u64],
    short: &[u64],
    scratch: &mut [u64],
) {
    let len = short.len();
    if scratch.len() < blocks_scratch(len, D::LEVELS) {
        basecase(out, long, short);
        return;
    }
    let (block_product, scratch) = split_mut(scratch, 2 * len);
    // The first block adds onto zeros.
    for slot in split_mut(out, len).0 {
        *slot = 0;
    }
    let mut offset = 0;
    let mut rest = long;
    while rest.len() >= len {
        let (block, tail) = split(rest, len);
        D::product_with::<Short, MASKED>(block_product, block, short, scratch);
        add_block::<MASKED>(out, block_product, offset, len);
        offset += len;
        rest = tail;
    }
    if !rest.is_empty() {
        let (last_product, _) = split_mut(block_product, len + rest.len());
        basecase(last_product, short, rest);
        add_block::<MASKED>(out, last_product, offset, len);
    }
}

/// Adds the block product `block` at limb `offset` of `out`: its low `len` limbs onto the limbs
/// that the previous block left there, its other limbs onto limbs that no block has written.
#[inline(always)]
fn add_block<const MASKED: bool>(out: &mut [u64], block: &[u64], offset: usize, len: usize) {
    let (_, window) = split_mut(out, offset);
    let (window, _) = split_mut(window, block.len());
    let (overlap, fresh) = split_mut(window, len);
    let (block_low, block_high) = split(block, len);
    let carry = add_assign(overlap, block_low);
    for (slot, &limb) in fresh.iter_mut().zip(block_high) {
        *slot = limb;
    }
    // The blocks so far sum to less than `2^(64 * (offset + block.len()))`.
    fits(carry_limb::<MASKED>(fresh, carry));
}

/// [`add_limb`], or under `MASKED` [`add_limb_masked`].
#[inline(always)]
fn carry_limb<const MASKED: bool>(out: &mut [u64], carry: u64) -> u64 {
    if MASKED {
        add_limb_masked(out, carry)
    } else {
        add_limb(out, carry)
    }
}

/// [`sub_limb`], or under `MASKED` [`sub_limb_masked`].
#[inline(always)]
fn borrow_limb<const MASKED: bool>(out: &mut [u64], borrow: u64) -> u64 {
    if MASKED {
        sub_limb_masked(out, borrow)
    } else {
        sub_limb(out, borrow)
    }
}
