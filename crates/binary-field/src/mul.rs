//! Carry-less products of limb slices: the schoolbook over word products below the Karatsuba
//! threshold, Karatsuba above it. Each recursion level is its own type, so the halved lengths fold
//! to constants down to the schoolbook.

use core::marker::PhantomData;

use clmul::clmul_into;

use crate::kernel::{copy, split, split_mut, xor_into, xor_shifted};

/// A recursion budget: how many times a product may halve before the schoolbook.
pub(crate) trait Depth {
    /// Writes `a * b` to `out`, modulo `x^(64 * out.len())`, for `a` and `b` of equal length.
    /// Operands of `threshold` limbs and more halve while the budget and `scratch` allow.
    fn product(out: &mut [u64], a: &[u64], b: &[u64], scratch: &mut [u64], threshold: usize);
}

/// The schoolbook.
pub(crate) struct Bottom;

/// One halving above `D`.
pub(crate) struct Level<D>(PhantomData<D>);

/// Five halvings: Karatsuba from 256 limbs down to the threshold.
pub(crate) type Levels = Level<Level<Level<Level<Level<Bottom>>>>>;

impl Depth for Bottom {
    #[inline(always)]
    fn product(out: &mut [u64], a: &[u64], b: &[u64], _scratch: &mut [u64], _threshold: usize) {
        clmul_into(out, a, b);
    }
}

impl<D: Depth> Depth for Level<D> {
    #[inline(always)]
    fn product(out: &mut [u64], a: &[u64], b: &[u64], scratch: &mut [u64], threshold: usize) {
        if a.len() >= threshold && a.len() > 1 && scratch.len() >= 2 * a.len().div_ceil(2) {
            karatsuba::<D>(out, a, b, scratch, threshold);
        } else {
            clmul_into(out, a, b);
        }
    }
}

/// Writes `a * b` to `out` for `a` and `b` of `len` limbs, with `a = a0 + a1 * x^(64h)`,
/// `h = ceil(len / 2)`: the product is `a0 b0 + ((a0 + a1)(b0 + b1) + a0 b0 + a1 b1) x^(64h) +
/// a1 b1 x^(128h)` (Karatsuba and Ofman, 1962; the middle term needs no subtraction in
/// characteristic 2). `out` holds at least `2 len - 1` limbs and `scratch` at least `2h`.
#[inline(always)]
fn karatsuba<D: Depth>(
    out: &mut [u64],
    a: &[u64],
    b: &[u64],
    scratch: &mut [u64],
    threshold: usize,
) {
    let half = a.len().div_ceil(2);
    let (a_low, a_high) = split(a, half);
    let (b_low, b_high) = split(b, half);
    let (middle, scratch) = split_mut(scratch, 2 * half);
    {
        // The sums wait in the limbs that the low product overwrites.
        let (a_sum, rest) = split_mut(out, half);
        let (b_sum, _) = split_mut(rest, half);
        copy(a_sum, a_low);
        xor_into(a_sum, a_high, false);
        copy(b_sum, b_low);
        xor_into(b_sum, b_high, false);
        D::product(middle, a_sum, b_sum, scratch, threshold);
    }
    let (low, high) = split_mut(out, 2 * half);
    D::product(low, a_low, b_low, scratch, threshold);
    D::product(high, a_high, b_high, scratch, threshold);
    xor_into(middle, low, false);
    xor_into(middle, high, false);
    xor_shifted(out, middle, 64 * half);
}
