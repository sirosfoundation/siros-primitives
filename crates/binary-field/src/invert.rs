//! Inversion: `a^(2^n - 2) = (a^(2^(n-1) - 1))^2` along the field's addition chain for `n - 1`
//! (Itoh and Tsujii, 1988).

use crate::chain::{Chains, MAX_STEPS};
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
///
/// `#[inline]`, the one kernel without `#[inline(always)]`: LLVM optimizes the inversion as a
/// function of its own before it decides to inline it. The always-inliner copied it unoptimized
/// into each caller, where promoting its chain's values to registers took SROA 14 minutes per
/// field in one release test function.
#[inline]
pub(crate) fn inverse<P: Polynomial, O: Inversion>(a: O::Value) -> O::Value {
    let Some(len) = Chains::<P>::CHAIN.len else {
        return binary::<P, O>(a);
    };
    let mut values = [a; MAX_STEPS + 1];
    steps::<P, O>(&mut values, len);
    let top = values.get(len).copied().unwrap_or(a);
    O::frobenius(top, SQUARING, 1)
}

/// The `len` steps of the chain of the field with the polynomial `P` over `values`. Builds without
/// debug assertions run one call per step, each with a literal index: the values stay in
/// registers, and each step's power is a constant. Debug builds loop.
#[inline(always)]
fn steps<P: Polynomial, O: Inversion>(values: &mut [O::Value; MAX_STEPS + 1], len: usize) {
    if cfg!(debug_assertions) {
        for index in 0..len {
            step::<P, O>(values, index);
        }
        return;
    }
    // One call per index below `MAX_STEPS`.
    if len > 0 {
        step::<P, O>(values, 0);
    }
    if len > 1 {
        step::<P, O>(values, 1);
    }
    if len > 2 {
        step::<P, O>(values, 2);
    }
    if len > 3 {
        step::<P, O>(values, 3);
    }
    if len > 4 {
        step::<P, O>(values, 4);
    }
    if len > 5 {
        step::<P, O>(values, 5);
    }
    if len > 6 {
        step::<P, O>(values, 6);
    }
    if len > 7 {
        step::<P, O>(values, 7);
    }
    if len > 8 {
        step::<P, O>(values, 8);
    }
    if len > 9 {
        step::<P, O>(values, 9);
    }
    if len > 10 {
        step::<P, O>(values, 10);
    }
    if len > 11 {
        step::<P, O>(values, 11);
    }
    if len > 12 {
        step::<P, O>(values, 12);
    }
    if len > 13 {
        step::<P, O>(values, 13);
    }
    if len > 14 {
        step::<P, O>(values, 14);
    }
    if len > 15 {
        step::<P, O>(values, 15);
    }
    if len > 16 {
        step::<P, O>(values, 16);
    }
    if len > 17 {
        step::<P, O>(values, 17);
    }
    if len > 18 {
        step::<P, O>(values, 18);
    }
    if len > 19 {
        step::<P, O>(values, 19);
    }
    if len > 20 {
        step::<P, O>(values, 20);
    }
    if len > 21 {
        step::<P, O>(values, 21);
    }
    if len > 22 {
        step::<P, O>(values, 22);
    }
    if len > 23 {
        step::<P, O>(values, 23);
    }
}

/// Step `index` of the chain of the field with the polynomial `P` over `values`.
#[inline(always)]
fn step<P: Polynomial, O: Inversion>(values: &mut [O::Value; MAX_STEPS + 1], index: usize) {
    if let Some(step) = Chains::<P>::CHAIN.steps.get(index)
        && let (Some(&raised), Some(&kept)) = (values.get(step.raised), values.get(step.kept))
        && let Some(slot) = values.get_mut(index + 1)
    {
        *slot = O::mul(O::frobenius(raised, index, step.power), kept);
    }
}

/// `a^(2^n - 2)` along the bits of `n - 1`, for degrees whose chain is longer than [`MAX_STEPS`]
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
