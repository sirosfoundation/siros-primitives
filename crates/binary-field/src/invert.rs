//! Inversion: `a^(2^n - 2)` through the chain of Itoh and Tsujii (1988).

use crate::arith::each;
use crate::plan::{Plan, Polynomial};

/// `Some(inverse)` when `invertible`, else `None`. The `Option` discloses `invertible`, so the
/// branch on it is the one decision of a constant-time inversion that may depend on the value.
/// The function keeps a symbol of its own, by which the constant-time check of `ct-check` exempts
/// that branch; it would exempt any other code here too.
#[inline(never)]
pub(crate) fn declassify<T>(invertible: bool, inverse: T) -> Option<T> {
    if invertible { Some(inverse) } else { None }
}

/// `a^(2^n - 2) = (a^(2^(n-1) - 1))^2`, with `a^(2^k - 1)` built along the bits of `n - 1`: a
/// doubling of `k` costs the Frobenius power `x -> x^(2^k)` and one product, an increment one
/// squaring and one product. `frobenius(x, k)` is `x^(2^k)`. The sequence depends on the degree
/// alone; `0` maps to `0`.
#[inline(always)]
pub(crate) fn itoh_tsujii<P: Polynomial, T: Copy>(
    a: T,
    frobenius: impl Fn(T, usize) -> T,
    mul: impl Fn(T, T) -> T,
) -> T {
    let exponent = Plan::<P>::DEGREE - 1;
    let bits = Plan::<P>::CHAIN_BITS;
    let mut power = a;
    let mut k = 1;
    each!(step < bits - 1 => {
        let index = bits - 2 - step;
        power = mul(frobenius(power, k), power);
        k *= 2;
        if (exponent >> index) & 1 == 1 {
            power = mul(frobenius(power, 1), a);
            k += 1;
        }
    });
    frobenius(power, 1)
}
