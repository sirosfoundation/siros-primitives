//! Reduction of a carry-less product modulo `f(x) = x^n + r(x)`: the bits at and above `n` move
//! down by `n` and multiply by `r`, since `x^n = r` in the field. The plan fixes the steps.

use crate::kernel::{clear_bits, extract, head, xor_shifted};
use crate::plan::{Plan, Polynomial};

/// Reduces `acc`, a product of degree below `2n - 1` in `WIDE` limbs, in place: afterwards its
/// low `LIMBS` limbs hold the element and the bits at and above `n` are zero. `scratch` holds at
/// least `WIDE` limbs.
#[inline(always)]
pub(crate) fn reduce<P: Polynomial>(acc: &mut [u64], scratch: &mut [u64]) {
    if Plan::<P>::FOLD_BY_CLMUL {
        fold_by_clmul::<P>(acc);
    } else {
        fold_by_shifts::<P>(acc, scratch);
    }
}

/// Each step takes at most one word of overflow and multiplies it by `r` with one carry-less word
/// product.
#[inline(always)]
fn fold_by_clmul<P: Polynomial>(acc: &mut [u64]) {
    for index in 0..Plan::<P>::STEPS {
        let (lo, hi) = Plan::<P>::step(index);
        let mut overflow = [0];
        extract(&mut overflow, acc, lo, hi);
        clear_bits(acc, lo, hi);
        let [word] = overflow;
        let product = clmul::clmul64(word, Plan::<P>::R_WORD);
        let product = [product as u64, (product >> 64) as u64];
        xor_shifted(acc, &product, lo - Plan::<P>::DEGREE);
    }
}

/// Each step takes the whole overflow and adds it once per tap, shifted by the tap. The const
/// evaluator runs it for the compile-time constants of the field.
#[inline(always)]
pub(crate) const fn fold_by_shifts<P: Polynomial>(acc: &mut [u64], scratch: &mut [u64]) {
    let mut index = 0;
    while index < Plan::<P>::SHIFT_STEPS {
        let (lo, hi) = Plan::<P>::shift_step(index);
        extract(scratch, acc, lo, hi);
        clear_bits(acc, lo, hi);
        let overflow = head(scratch, (hi - lo).div_ceil(64));
        let mut taps = Plan::<P>::TAPS;
        while let [tap, rest @ ..] = taps {
            xor_shifted(acc, overflow, lo - Plan::<P>::DEGREE + *tap);
            taps = rest;
        }
        index += 1;
    }
}
