//! Products modulo `2^(64 * n)` of two `n`-limb operands by divide and conquer (GMP's
//! `mpn_dc_mullo_n`): the full product of the low parts by Karatsuba and the truncated cross
//! products of the high parts.

use crate::kernel::{add_assign, split, split_mut};
use crate::recursion::{Depth, product_scratch};

use super::basecase::{basecase, rows, truncate_carry};

/// Truncated products of two operands of at least this many limbs split into a full product and
/// two truncated cross products.
pub(crate) const THRESHOLD: usize = 48;

/// The high part of a split: `11/36` of the length, GMP's split below its Toom-3 range.
#[inline(always)]
const fn high_part(len: usize) -> usize {
    len * 11 / 36
}

/// The scratch limbs of [`truncated_karatsuba`] for `len`-limb operands with `levels` halvings
/// left: the full product of the low parts with its own scratch.
#[inline(always)]
const fn scratch_len(len: usize, levels: usize) -> usize {
    let low = len - high_part(len);
    2 * low + product_scratch(low, levels)
}

/// Writes `a * b mod 2^(64 * n)` to `out`, all three `n` limbs long: with `a = a0 + a1 β^m` and
/// `b = b0 + b1 β^m`, the low `n` limbs of the full product `a0 * b0`, plus `a1 * b0` and
/// `a0 * b1` modulo `β^(n - m)` at limb `m`. Without `scratch_len` limbs of `scratch`, the product
/// runs by rows.
#[inline(always)]
pub(crate) fn truncated_karatsuba<D: Depth>(
    out: &mut [u64],
    a: &[u64],
    b: &[u64],
    scratch: &mut [u64],
) {
    let len = out.len();
    if scratch.len() < scratch_len(len, D::LEVELS) {
        rows(out, a, b);
        return;
    }
    let high = high_part(len);
    let low = len - high;
    let (a0, a1) = split(a, low);
    let (b0, b1) = split(b, low);
    {
        let (full, scratch) = split_mut(scratch, 2 * low);
        D::product(full, a0, b0, scratch);
        for (slot, &limb) in out.iter_mut().zip(full.iter()) {
            *slot = limb;
        }
    }
    let (cross, _) = split_mut(scratch, high);
    let (_, top) = split_mut(out, low);
    basecase(cross, a1, split(b0, high).0);
    truncate_carry(add_assign(top, cross));
    basecase(cross, b1, split(a0, high).0);
    truncate_carry(add_assign(top, cross));
}
