//! Karatsuba multiplication (Karatsuba and Ofman, 1962): three half-size products in place of
//! four, in the subtractive form with the in-place interpolation of GMP's `mpn_toom22_mul`.

use core::hint::select_unpredictable;

use crate::kernel::{
    add_assign, add_limb, add_to, compare, fits, split, split_mut, sub_assign, sub_limb,
};
use crate::recursion::{Depth, product_scratch};
use crate::stream::{LimbStream, Op2CarryIter, SubOp, ZeroExtended};

use super::basecase;

/// Operands shorter than this many limbs multiply by the quadratic product.
pub(crate) const THRESHOLD: usize = 16;

/// Writes `a * b` to `out[..2n]` for `a` and `b` of `n` limbs, with the sub-products at depth
/// `D`. `scratch` holds `product_scratch(n, D::LEVELS + 1)` limbs.
///
/// With `a = a0 + a1 * β^h` and `b = b0 + b1 * β^h`, `h = ceil(n / 2)`, the product is
/// `v0 + (v0 + vinf - vm1) * β^h + vinf * β^2h` for `v0 = a0 * b0`, `vinf = a1 * b1` and
/// `vm1 = (a0 - a1) * (b0 - b1)`.
#[inline(always)]
pub(crate) fn karatsuba<D: Depth>(out: &mut [u64], a: &[u64], b: &[u64], scratch: &mut [u64]) {
    let half = a.len() - a.len() / 2;
    let (a_low, a_high) = split(a, half);
    let (b_low, b_high) = split(b, half);
    let (difference_product, scratch) = split_mut(scratch, 2 * half);
    let negative = {
        // `|a0 - a1|` and `|b0 - b1|` wait in the limbs that `v0` fills last.
        let (lower, _) = split_mut(out, 2 * half);
        let (a_difference, b_difference) = split_mut(lower, half);
        let negative = abs_difference(a_difference, a_low, a_high)
            != abs_difference(b_difference, b_low, b_high);
        D::product(difference_product, a_difference, b_difference, scratch);
        negative
    };
    {
        let (lower, upper) = split_mut(out, 2 * half);
        D::product(upper, a_high, b_high, scratch);
        D::product(lower, a_low, b_low, scratch);
    }
    interpolate(out, difference_product, negative, half);
}

/// Writes `|x - y|` to `out`, as long as `x`, and returns whether `x < y`. `y` is at most as long
/// as `x`.
#[inline(always)]
fn abs_difference(out: &mut [u64], x: &[u64], y: &[u64]) -> bool {
    let below = compare(x, y).is_lt();
    let (larger, smaller) = select_unpredictable(below, (y, x), (x, y));
    let mut difference =
        Op2CarryIter::<_, _, SubOp>::new(ZeroExtended::new(larger), ZeroExtended::new(smaller));
    for slot in out {
        *slot = difference.next_limb();
    }
    below
}

/// Turns `out = v0 | vinf` into the product, given `|vm1|` and the sign of `vm1`. With
/// `v0 = L0 | H0` and `vinf = L1 | H1` split at `half` limbs, the product is
/// `L0 + (H0 + L0 + L1) β^h + (H0 + L1 + H1) β^2h + H1 β^3h - vm1 β^h`.
#[inline(always)]
fn interpolate(out: &mut [u64], difference_product: &[u64], negative: bool, half: usize) {
    let (carry_at_2h, mut carry_at_3h) = {
        let (v0, vinf) = split_mut(out, 2 * half);
        let (l0, h0) = split_mut(v0, half);
        let (l1, h1) = split_mut(vinf, half);
        // X = H0 + L1 replaces L1; X + L0 replaces H0; X + H1 replaces X.
        let carry_x = add_assign(l1, h0);
        let carry_at_2h = carry_x + add_to(h0, l1, l0);
        let (l1_low, l1_high) = split_mut(l1, h1.len());
        let carry_at_3h = carry_x + add_limb(l1_high, add_assign(l1_low, h1));
        (carry_at_2h, carry_at_3h)
    };
    let (_, from_h) = split_mut(out, half);
    let borrow_at_3h = {
        let (middle, _) = split_mut(from_h, 2 * half);
        if negative {
            carry_at_3h += add_assign(middle, difference_product);
            0
        } else {
            sub_assign(middle, difference_product)
        }
    };
    // The carries and the borrow cancel to a product that fits in `out`.
    let (_, from_2h) = split_mut(from_h, half);
    fits(add_limb(from_2h, carry_at_2h));
    let (_, from_3h) = split_mut(from_2h, half);
    fits(add_limb(from_3h, carry_at_3h));
    fits(sub_limb(from_3h, borrow_at_3h));
}

/// The scratch limbs of [`blocks`] for a `len`-limb shorter operand: a block product and the
/// scratch of the product that fills it.
#[inline(always)]
pub(crate) const fn blocks_scratch(len: usize, levels: usize) -> usize {
    2 * len + product_scratch(len, levels)
}

/// Writes `long * short` to `out[..long.len() + short.len()]` for `long.len() >= short.len()`:
/// products of `short` with the blocks of `long` that are as long as `short`, added at their
/// offsets, then the shorter last block by the quadratic product. Without `blocks_scratch` limbs of
/// `scratch`, the whole product runs as the quadratic product.
#[inline(always)]
pub(crate) fn blocks<D: Depth>(out: &mut [u64], long: &[u64], short: &[u64], scratch: &mut [u64]) {
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
        D::product(block_product, block, short, scratch);
        add_block(out, block_product, offset, len);
        offset += len;
        rest = tail;
    }
    if !rest.is_empty() {
        let (last_product, _) = split_mut(block_product, len + rest.len());
        basecase(last_product, short, rest);
        add_block(out, last_product, offset, len);
    }
}

/// Adds the block product `block` at limb `offset` of `out`: its low `len` limbs onto the limbs
/// that the previous block left there, its other limbs onto limbs that no block has written.
#[inline(always)]
fn add_block(out: &mut [u64], block: &[u64], offset: usize, len: usize) {
    let (_, window) = split_mut(out, offset);
    let (window, _) = split_mut(window, block.len());
    let (overlap, fresh) = split_mut(window, len);
    let (block_low, block_high) = split(block, len);
    let carry = add_assign(overlap, block_low);
    for (slot, &limb) in fresh.iter_mut().zip(block_high) {
        *slot = limb;
    }
    // The blocks so far sum to less than `2^(64 * (offset + block.len()))`.
    fits(add_limb(fresh, carry));
}
