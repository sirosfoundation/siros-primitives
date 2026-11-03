//! Inversion: `a^(2^n - 2) = (a^(2^(n-1) - 1))^2` along the field's addition chain for `n - 1`
//! (Itoh and Tsujii, 1988).

use biguint::Indices;

use crate::chain::{Chains, MAX_STEPS, MaxSteps};
use crate::plan::{Plan, Polynomial};

/// `Some(inverse)` when `invertible`, else `None`. The `Option` discloses `invertible`, so the
/// branch on it is the one decision of a constant-time inversion that may depend on the value.
/// The function keeps a symbol of its own, by which the constant-time check of `ct-check` exempts
/// that branch; it would exempt any other code here too.
#[inline(never)]
pub(crate) fn declassify<T>(invertible: bool, inverse: T) -> Option<T> {
    if invertible { Some(inverse) } else { None }
}

/// The step of the final squaring, past every step of a chain.
pub(crate) const SQUARING: usize = MAX_STEPS;

/// The operations of an inversion along a chain.
pub(crate) trait Inversion {
    /// The registers of a value.
    type Value: Copy;

    /// `x^(2^power)`, for the step `step` of the chain or [`SQUARING`].
    fn frobenius(x: Self::Value, step: usize, power: usize) -> Self::Value;

    /// `a b`.
    fn mul(a: Self::Value, b: Self::Value) -> Self::Value;
}

/// `a^(2^n - 2)`: `b(c) = a^(2^c - 1)` along the field's chain, each step
/// `b(x + y) = b(x)^(2^y) b(y)` one Frobenius power and one product, then the square of `b(n - 1)`.
/// The sequence depends on the degree alone; `0` maps to `0`.
#[inline(always)]
pub(crate) fn inverse<P: Polynomial, O: Inversion>(a: O::Value) -> O::Value {
    let Some(len) = Chains::<P>::CHAIN.len else {
        return binary::<P, O>(a);
    };
    let mut values = [a; MAX_STEPS + 1];
    // One copy per step in builds without debug assertions: the values stay in registers, and
    // each step's power is a constant.
    MaxSteps::each(
        #[inline(always)]
        |index| {
            if index < len
                && let Some(step) = Chains::<P>::CHAIN.steps.get(index)
                && let (Some(&raised), Some(&kept)) =
                    (values.get(step.raised), values.get(step.kept))
                && let Some(slot) = values.get_mut(index + 1)
            {
                *slot = O::mul(O::frobenius(raised, index, step.power), kept);
            }
        },
    );
    let top = values.get(len).copied().unwrap_or(a);
    O::frobenius(top, SQUARING, 1)
}

/// `a^(2^n - 2)` along the bits of `n - 1`, for degrees whose chain is longer than [`MaxSteps`]
/// steps: a doubling of `k` costs the Frobenius power `2^k` and one product, an increment one
/// squaring and one product.
#[inline(always)]
fn binary<P: Polynomial, O: Inversion>(a: O::Value) -> O::Value {
    let exponent = Plan::<P>::DEGREE - 1;
    let bits = Plan::<P>::CHAIN_BITS;
    let mut power = a;
    let mut k = 1;
    for step in 0..bits - 1 {
        let index = bits - 2 - step;
        power = O::mul(O::frobenius(power, SQUARING, k), power);
        k *= 2;
        if (exponent >> index) & 1 == 1 {
            power = O::mul(O::frobenius(power, SQUARING, 1), a);
            k += 1;
        }
    }
    O::frobenius(power, SQUARING, 1)
}
