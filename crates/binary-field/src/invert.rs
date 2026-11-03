//! Inversion: `a^(2^n - 2) = (a^(2^(n-1) - 1))^2` along the field's addition chain for `n - 1`
//! (Itoh and Tsujii, 1988).

#[cfg(not(debug_assertions))]
use core::marker::PhantomData;

use crate::arith::each;
use crate::chain::{Chains, MAX_STEPS};
use crate::plan::{Plan, Polynomial};

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

/// Step `S` of the chain of the field with the polynomial `P`, as constants.
#[cfg(not(debug_assertions))]
struct Step<P, const S: usize>(PhantomData<P>);

#[cfg(not(debug_assertions))]
impl<P: Polynomial, const S: usize> Step<P, S> {
    /// Whether the chain has the step.
    const LIVE: bool = match Chains::<P>::CHAIN.len {
        Some(len) => S < len,
        None => false,
    };
    /// The step: the indices of its values and its power.
    const STEP: crate::chain::Step = match Chains::<P>::CHAIN.steps.split_at(S).1 {
        [step, ..] => *step,
        [] => crate::chain::Step {
            raised: 0,
            kept: 0,
            power: 0,
        },
    };
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
    // Optimized builds unroll every step, so that the values stay in registers; debug builds loop.
    #[cfg(not(debug_assertions))]
    {
        macro_rules! steps {
            ($($step:literal)*) => {$(
                if Step::<P, $step>::LIVE {
                    let step = Step::<P, $step>::STEP;
                    if let (Some(&raised), Some(&kept)) =
                        (values.get(step.raised), values.get(step.kept))
                        && let Some(slot) = values.get_mut($step + 1)
                    {
                        *slot = O::mul(O::frobenius(raised, $step, step.power), kept);
                    }
                }
            )*};
        }
        steps!(0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23);
    }
    #[cfg(debug_assertions)]
    chain_loop::<P, O>(&mut values, len);
    let top = values.get(len).copied().unwrap_or(a);
    O::frobenius(top, SQUARING, 1)
}

/// The steps of the chain of the field with the polynomial `P` in a loop, over the values of a
/// chain of `len` steps: the steps of debug builds.
#[cfg(debug_assertions)]
fn chain_loop<P: Polynomial, O: Inversion>(values: &mut [O::Value; MAX_STEPS + 1], len: usize) {
    let mut index = 0;
    while index < len {
        if let Some(step) = Chains::<P>::CHAIN.steps.get(index)
            && let (Some(&raised), Some(&kept)) = (values.get(step.raised), values.get(step.kept))
            && let Some(slot) = values.get_mut(index + 1)
        {
            *slot = O::mul(O::frobenius(raised, index, step.power), kept);
        }
        index += 1;
    }
}

/// `a^(2^n - 2)` along the bits of `n - 1`, for degrees whose chain is longer than the unrolled
/// steps: a doubling of `k` costs the Frobenius power `2^k` and one product, an increment one
/// squaring and one product.
#[inline(always)]
fn binary<P: Polynomial, O: Inversion>(a: O::Value) -> O::Value {
    let exponent = Plan::<P>::DEGREE - 1;
    let bits = Plan::<P>::CHAIN_BITS;
    let mut power = a;
    let mut k = 1;
    each!(step < bits - 1 => {
        let index = bits - 2 - step;
        power = O::mul(O::frobenius(power, SQUARING, k), power);
        k *= 2;
        if (exponent >> index) & 1 == 1 {
            power = O::mul(O::frobenius(power, SQUARING, 1), a);
            k += 1;
        }
    });
    O::frobenius(power, SQUARING, 1)
}
