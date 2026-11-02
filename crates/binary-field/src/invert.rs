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
/// doubling of `k` costs `k` squarings and one product, an increment one squaring and one product.
/// The sequence depends on the degree alone; `0` maps to `0`.
#[inline(always)]
pub(crate) fn itoh_tsujii<P: Polynomial, T: Copy>(
    a: T,
    square: impl Fn(T) -> T,
    mul: impl Fn(T, T) -> T,
) -> T {
    let exponent = Plan::<P>::DEGREE - 1;
    let bits = Plan::<P>::CHAIN_BITS;
    let mut power = a;
    let mut k = 1;
    each!(step < bits - 1 => {
        let index = bits - 2 - step;
        let mut doubled = power;
        let mut count = 0;
        while count < k {
            doubled = square(doubled);
            count += 1;
        }
        power = mul(doubled, power);
        k *= 2;
        if (exponent >> index) & 1 == 1 {
            power = mul(square(power), a);
            k += 1;
        }
    });
    square(power)
}
