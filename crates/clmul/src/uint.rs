//! Carry-less products of `biguint` integers.

use core::ops::Add;

use biguint::typenum::Sum;
use biguint::{Bits, Uint};

use crate::limbs::{clmul_into, const_clmul_into};

/// Carry-less products of [`Uint`] values: a `Uint<N>` holds the polynomial over GF(2) whose
/// coefficient of `x^i` is bit `i`. `Uint<N>` implements `CarrylessMul<N>`.
///
/// ```
/// use biguint::{U128, U256};
/// use clmul::CarrylessMul;
///
/// // (x + 1)^2 = x^2 + 1.
/// let x_plus_1 = U128::from_u64(0b11);
/// assert_eq!(x_plus_1.carryless_mul(x_plus_1), U128::from_u64(0b101));
///
/// // x^64 * x^64 = x^128, which only the widening product holds.
/// let x64 = U128::ONE << 64usize;
/// assert_eq!(x64.carryless_mul(x64), U128::ZERO);
/// assert_eq!(x64.widening_carryless_mul(x64), U256::ONE << 128usize);
/// ```
pub trait CarrylessMul<N: Bits> {
    /// `self * rhs mod x^N`: the product truncated to the width of `self`, as `Uint`'s `*`
    /// truncates.
    #[must_use]
    fn carryless_mul<M: Bits>(self, rhs: Uint<M>) -> Uint<N>;

    /// The whole product `self * rhs`, in `N + M` bits, as
    /// [`WideningMul`](biguint::WideningMul) widens.
    #[must_use]
    fn widening_carryless_mul<M>(self, rhs: Uint<M>) -> Uint<Sum<N, M>>
    where
        N: Add<M>,
        M: Bits,
        Sum<N, M>: Bits;
}

/// The word products come from [`clmul64`](crate::clmul64).
impl<N: Bits> CarrylessMul<N> for Uint<N> {
    #[inline]
    fn carryless_mul<M: Bits>(self, rhs: Uint<M>) -> Self {
        product(&self, &rhs)
    }

    #[inline]
    fn widening_carryless_mul<M>(self, rhs: Uint<M>) -> Uint<Sum<N, M>>
    where
        N: Add<M>,
        M: Bits,
        Sum<N, M>: Bits,
    {
        product(&self, &rhs)
    }
}

/// [`CarrylessMul::carryless_mul`], evaluable in const contexts: the word products come from
/// [`const_clmul64`](crate::const_clmul64).
#[inline(always)]
#[must_use]
pub const fn const_carryless_mul<N: Bits, M: Bits>(a: Uint<N>, b: Uint<M>) -> Uint<N> {
    const_product(&a, &b)
}

/// [`CarrylessMul::widening_carryless_mul`], evaluable in const contexts: the word products come
/// from [`const_clmul64`](crate::const_clmul64).
#[inline(always)]
#[must_use]
pub const fn const_widening_carryless_mul<N, M>(a: Uint<N>, b: Uint<M>) -> Uint<Sum<N, M>>
where
    N: Bits + Add<M>,
    M: Bits,
    Sum<N, M>: Bits,
{
    const_product(&a, &b)
}

/// Defines `$name`, the product of a `Uint<N>` and a `Uint<M>` modulo `x^P`, over the slice
/// product `$slice_product`.
macro_rules! uint_product {
    ([$($qualifier:tt)*] fn $name:ident uses $slice_product:path) => {
        #[inline(always)]
        $($qualifier)* fn $name<P: Bits, N: Bits, M: Bits>(a: &Uint<N>, b: &Uint<M>) -> Uint<P> {
            let mut limbs = Uint::<P>::ZERO.into_limbs();
            $slice_product(limbs.as_mut_slice(), a.as_limbs(), b.as_limbs());
            Uint::from_generic_limbs(limbs)
        }
    };
}

uint_product!([] fn product uses clmul_into);
uint_product!([const] fn const_product uses const_clmul_into);
