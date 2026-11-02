//! Reduction of a carry-less product modulo `f(x) = x^n + r(x)`: the bits at and above `n` move
//! down by `n` and multiply by `r`, since `x^n = r` in the field. The plan fixes the steps.
//!
//! Each step's bit range is an associated const of its type-level index, so that the shifts and
//! the limb counts of its kernels are immediates. The first [`UNROLLED`] steps expand in place;
//! a degree with more steps runs the rest in a loop.

use core::marker::PhantomData;

use crate::kernel::{clear_bits, extract, head, xor_shifted};
use crate::plan::{Plan, Polynomial};

/// The number of fold steps that expand in place: enough for the chunked plan of a degree up to
/// `64 * UNROLLED`.
const UNROLLED: usize = 64;

/// The bit ranges of the fold step `I` of the field with the polynomial `P`.
struct Step<P, const I: usize>(PhantomData<P>);

impl<P: Polynomial, const I: usize> Step<P, I> {
    /// The bits that the step of the chosen reduction moves down; empty beyond the last step.
    const BOUNDS: (usize, usize) = Plan::<P>::step(I);
    /// The bits that the step of the shift-and-add reduction moves down.
    const SHIFT_BOUNDS: (usize, usize) = Plan::<P>::shift_step(I);
}

/// Reduces `acc`, a product of degree below `2n - 1` in `WIDE` limbs, in place: afterwards its
/// low `LIMBS` limbs hold the element and the bits at and above `n` are zero. `scratch` holds at
/// least `WIDE` limbs.
#[inline(always)]
pub(crate) fn reduce<P: Polynomial>(acc: &mut [u64], scratch: &mut [u64]) {
    macro_rules! unrolled {
        ($($index:literal)*) => {
            $(
                if $index < Plan::<P>::STEPS {
                    if Plan::<P>::FOLD_BY_CLMUL {
                        clmul_step::<P>(acc, Step::<P, $index>::BOUNDS);
                    } else {
                        shift_step::<P>(acc, scratch, Step::<P, $index>::SHIFT_BOUNDS);
                    }
                }
            )*
        };
    }
    unrolled! {
        0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31
        32 33 34 35 36 37 38 39 40 41 42 43 44 45 46 47 48 49 50 51 52 53 54 55 56 57 58 59 60 61
        62 63
    }
    for index in UNROLLED..Plan::<P>::STEPS {
        if Plan::<P>::FOLD_BY_CLMUL {
            clmul_step::<P>(acc, Plan::<P>::step(index));
        } else {
            shift_step::<P>(acc, scratch, Plan::<P>::shift_step(index));
        }
    }
}

/// Moves the bits `lo..hi` of `acc`, at most one word, down by `n`, multiplied by `r` with one
/// carry-less word product.
#[inline(always)]
fn clmul_step<P: Polynomial>(acc: &mut [u64], (lo, hi): (usize, usize)) {
    let mut overflow = [0];
    extract(&mut overflow, acc, lo, hi);
    clear_bits(acc, lo, hi);
    let [word] = overflow;
    let product = clmul::clmul64(word, Plan::<P>::R_WORD);
    let product = [product as u64, (product >> 64) as u64];
    xor_shifted(acc, &product, lo - Plan::<P>::DEGREE);
}

/// Moves the bits `lo..hi` of `acc` down by `n`, added once per tap, shifted by the tap. `scratch`
/// receives the bits on the way.
#[inline(always)]
const fn shift_step<P: Polynomial>(acc: &mut [u64], scratch: &mut [u64], (lo, hi): (usize, usize)) {
    extract(scratch, acc, lo, hi);
    clear_bits(acc, lo, hi);
    let overflow = head(scratch, (hi - lo).div_ceil(64));
    let mut taps = Plan::<P>::TAPS;
    while let [tap, rest @ ..] = taps {
        xor_shifted(acc, overflow, lo - Plan::<P>::DEGREE + *tap);
        taps = rest;
    }
}

/// The shift-and-add reduction in a loop, for the const evaluator, which runs the compile-time
/// constants of the field.
#[inline(always)]
pub(crate) const fn const_reduce<P: Polynomial>(acc: &mut [u64], scratch: &mut [u64]) {
    let mut index = 0;
    while index < Plan::<P>::SHIFT_STEPS {
        shift_step::<P>(acc, scratch, Plan::<P>::shift_step(index));
        index += 1;
    }
}
