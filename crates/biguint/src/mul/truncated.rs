//! Products modulo `2^(64 * n)` of two `n`-limb operands by divide and conquer (GMP's
//! `mpn_dc_mullo_n`): the full product of the low parts and the truncated cross products of the
//! high parts, which split again.

use core::marker::PhantomData;

use crate::kernel::{add_assign, split, split_mut};
use crate::recursion::{
    Depth, Len, basecase_of_slices, exact, exact_mut, prefix_mut, product_scratch,
};

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
/// low part would stay quadratic, `11/36` of the length once Karatsuba makes it cheaper. On
/// WebAssembly always half: the 2048-bit wrapping product took Chrome 1,846 cycles and Firefox
/// 3,496 in halves against 2,325 and 3,508 at `11/36`, without wide arithmetic 4,834 and 6,102
/// against 5,600 and 6,575.
#[inline(always)]
const fn high_part(len: usize) -> usize {
    if cfg!(target_arch = "wasm32") || len * 25 / 36 < UNBALANCED_LOW {
        len / 2
    } else {
        len * 11 / 36
    }
}

/// The scratch limbs of a truncated product of two `len`-limb operands with `levels` splits left:
/// at each split, the full product of the low parts with its own scratch, or the cross products,
/// whose split nests above the high parts of the splits before it; the longest of these. A loop
/// over the chain of high parts, which the compiler evaluates for constant lengths.
#[inline(always)]
pub(crate) const fn scratch_len(len: usize, levels: usize) -> usize {
    let (mut len, mut levels) = (len, levels);
    // The high parts of the splits so far, which a cross product holds below its own scratch.
    let mut below = 0;
    let mut longest = 0;
    while levels > 0 && !quadratic(len) {
        let high = high_part(len);
        let low = len - high;
        let full = below + 2 * low + product_scratch(low, levels);
        if full > longest {
            longest = full;
        }
        below += high;
        len = high;
        levels -= 1;
    }
    if below > longest { below } else { longest }
}

/// Writes `a * b mod 2^(64 * n)` to `out`, all three `n = L` limbs long: with `a = a0 + a1 β^m`
/// and `b = b0 + b1 β^m`, the low `n` limbs of the full product `a0 * b0` at depth `F`, plus
/// `a1 * b0` and `a0 * b1` modulo `β^(n - m)` at depth `C`, added at limb `m`. `scratch` holds
/// [`scratch_len`] limbs for the levels of `F`; other lengths run the quadratic product. `MASKED`
/// as for [`Depth::product_with`].
#[inline(never)]
pub(crate) fn split_product<F: Depth, C: Depth, L: Len, const MASKED: bool>(
    out: &mut [u64],
    a: &[u64],
    b: &[u64],
    scratch: &mut [u64],
) {
    split_product_inlined::<F, C, L, MASKED>(out, a, b, scratch);
}

/// [`split_product`] inlined into its caller.
#[inline(always)]
pub(crate) fn split_product_inlined<F: Depth, C: Depth, L: Len, const MASKED: bool>(
    out: &mut [u64],
    a: &[u64],
    b: &[u64],
    scratch: &mut [u64],
) {
    let n = L::of(out.len());
    let (Some(a), Some(b)) = (exact(a, n), exact(b, n)) else {
        return basecase_of_slices(out, a, b);
    };
    let (Some(out), Some(scratch)) = (
        exact_mut(out, n),
        prefix_mut(scratch, scratch_len(n, F::LEVELS)),
    ) else {
        return basecase_of_slices(out, a, b);
    };
    let high = high_part(n);
    let low = n - high;
    let (a0, a1) = split(a, low);
    let (b0, b1) = split(b, low);
    {
        let (full, scratch) = split_mut(scratch, 2 * low);
        F::product_with::<Low<L>, MASKED>(full, a0, b0, scratch);
        for (slot, &limb) in out.iter_mut().zip(full.iter()) {
            *slot = limb;
        }
    }
    let (cross, scratch) = split_mut(scratch, high);
    let (_, top) = split_mut(out, low);
    C::truncated_with::<High<L>, MASKED>(cross, a1, split(b0, high).0, scratch);
    truncate_carry(add_assign(top, cross));
    C::truncated_with::<High<L>, MASKED>(cross, b1, split(a0, high).0, scratch);
    truncate_carry(add_assign(top, cross));
}

/// The high part of a split of `P` limbs, [`high_part`].
pub(crate) struct High<P>(PhantomData<P>);

impl<P: Len> Len for High<P> {
    const LEN: usize = high_part(P::LEN);
    const ERASED: bool = P::ERASED;
}

/// The low part of a split of `P` limbs, below [`High`].
pub(crate) struct Low<P>(PhantomData<P>);

impl<P: Len> Len for Low<P> {
    const LEN: usize = P::LEN - high_part(P::LEN);
    const ERASED: bool = P::ERASED;
}
