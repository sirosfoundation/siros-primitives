//! Products modulo `2^(64 * n)` of two `n`-limb operands by divide and conquer (GMP's
//! `mpn_dc_mullo_n`): the full product of the low parts and the truncated cross products of the
//! high parts, which split again.

use crate::kernel::{add_assign, split, split_mut};
use crate::recursion::{Depth, product_scratch};

use super::basecase::truncate_carry;

/// The shortest operands whose truncated product splits.
const SPLIT_LIMBS: usize = 17;

/// Whether the truncated product of two `len`-limb operands runs as the quadratic product.
#[inline(always)]
pub(crate) const fn quadratic(len: usize) -> bool {
    len < SPLIT_LIMBS
}

/// The shortest low part whose full product makes the `11/36` split pay: Karatsuba beats the
/// quadratic product clearly from there. Measured with rustc 1.99, `-C target-cpu=native`: 24 limbs
/// take 1,790 instructions split in halves against 1,968 at `11/36`, 28 limbs 2,390 at `11/36`
/// against 2,483 in halves.
const UNBALANCED_LOW: usize = 19;

/// The high part of a split of `len` limbs, after GMP's rule: half while the full product of the
/// low part would stay quadratic, `11/36` of the length once Karatsuba makes it cheaper.
#[inline(always)]
const fn high_part(len: usize) -> usize {
    if len * 25 / 36 < UNBALANCED_LOW {
        len / 2
    } else {
        len * 11 / 36
    }
}

/// The scratch limbs of a truncated product of two `len`-limb operands with `levels` splits left:
/// the full product of the low parts with its own scratch, or a cross product with the scratch of
/// its own split, whichever is longer.
#[inline(always)]
pub(crate) const fn scratch_len(len: usize, levels: usize) -> usize {
    if levels == 0 || quadratic(len) {
        return 0;
    }
    let high = high_part(len);
    let low = len - high;
    let full = 2 * low + product_scratch(low, levels);
    let cross = high + scratch_len(high, levels - 1);
    if full > cross { full } else { cross }
}

/// Writes `a * b mod 2^(64 * n)` to `out`, all three `n` limbs long: with `a = a0 + a1 β^m` and
/// `b = b0 + b1 β^m`, the low `n` limbs of the full product `a0 * b0` at depth `F`, plus `a1 * b0`
/// and `a0 * b1` modulo `β^(n - m)` at depth `C`, added at limb `m`. `scratch` holds
/// [`scratch_len`] limbs for the levels of `F`.
#[inline(always)]
pub(crate) fn split_product<F: Depth, C: Depth>(
    out: &mut [u64],
    a: &[u64],
    b: &[u64],
    scratch: &mut [u64],
) {
    let high = high_part(out.len());
    let low = out.len() - high;
    let (a0, a1) = split(a, low);
    let (b0, b1) = split(b, low);
    {
        let (full, scratch) = split_mut(scratch, 2 * low);
        F::product(full, a0, b0, scratch);
        for (slot, &limb) in out.iter_mut().zip(full.iter()) {
            *slot = limb;
        }
    }
    let (cross, scratch) = split_mut(scratch, high);
    let (_, top) = split_mut(out, low);
    C::truncated(cross, a1, split(b0, high).0, scratch);
    truncate_carry(add_assign(top, cross));
    C::truncated(cross, b1, split(a0, high).0, scratch);
    truncate_carry(add_assign(top, cross));
}
