//! Reduction of a product modulo `f(x) = x^n + r(x)` over limbs, for the const evaluator, which
//! computes the constants of a field: the bits at and above `n` move down by `n` and add once per
//! tap, shifted by the tap, since `x^n = r` in the field.

use crate::kernel::{clear_bits, extract, head, xor_shifted};
use crate::plan::{Plan, Polynomial};

/// Reduces `acc`, a product of degree below `2n - 1`, in place: afterwards its low limbs hold the
/// element and the bits at and above `n` are zero. `scratch` holds at least as many limbs as
/// `acc`.
pub(crate) const fn const_reduce<P: Polynomial>(acc: &mut [u64], scratch: &mut [u64]) {
    let degree = Plan::<P>::DEGREE;
    let mut top = 2 * degree - 1;
    while top > degree {
        extract(scratch, acc, degree, top);
        clear_bits(acc, degree, top);
        let overflow = head(scratch, (top - degree).div_ceil(64));
        let mut taps = Plan::<P>::TAPS;
        while let [tap, rest @ ..] = taps {
            xor_shifted(acc, overflow, *tap);
            taps = rest;
        }
        let landed = top - degree + Plan::<P>::R_DEGREE;
        top = if landed > degree { landed } else { degree };
    }
}
