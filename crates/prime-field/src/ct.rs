//! The constant-time operations of `Fp`, through which [`ConstantTime<Fp<M>>`](ConstantTime)
//! computes: the kernels that take one path for every value.

use core::ops::{Add, Mul, Sub};

use biguint::typenum::{IsLess, True};
use biguint::{Bits, NonZero, NonZeroUint, Uint};
use constant_time::{
    ConstantTime, CtAdd, CtDiv, CtMul, CtNeg, CtNonZero, CtPow, CtSqrt, CtSquare, CtSub, Public,
};

use crate::field::{Fp, Modulus};

impl<M: Modulus> Public for Fp<M> {}

/// `self + rhs`.
impl<M: Modulus> CtAdd for Fp<M> {
    type Output = Self;

    #[inline]
    fn ct_add(self, rhs: Self) -> Self {
        self.add_with(rhs)
    }
}

/// `self - rhs`.
impl<M: Modulus> CtSub for Fp<M> {
    type Output = Self;

    #[inline]
    fn ct_sub(self, rhs: Self) -> Self {
        self.sub_with(rhs)
    }
}

/// `self * rhs`.
impl<M: Modulus> CtMul for Fp<M> {
    type Output = Self;

    #[inline]
    fn ct_mul(self, rhs: Self) -> Self {
        self.mul_with::<true>(rhs)
    }
}

/// `self + k mod p` for an integer `k` of any width and value.
impl<M: Modulus, K: Bits> CtAdd<Uint<K>> for Fp<M> {
    type Output = Self;

    #[inline(always)]
    fn ct_add(self, rhs: Uint<K>) -> Self {
        self.add_with(Self::integer_with::<true>(rhs.as_limbs()))
    }
}

/// `self - k mod p` for an integer `k` of any width and value.
impl<M: Modulus, K: Bits> CtSub<Uint<K>> for Fp<M> {
    type Output = Self;

    #[inline(always)]
    fn ct_sub(self, rhs: Uint<K>) -> Self {
        self.sub_with(Self::integer_with::<true>(rhs.as_limbs()))
    }
}

/// `self k mod p` for an integer `k` of any width and value.
impl<M: Modulus, K: Bits> CtMul<Uint<K>> for Fp<M> {
    type Output = Self;

    #[inline(always)]
    fn ct_mul(self, rhs: Uint<K>) -> Self {
        self.scale_with::<true>(rhs.as_limbs())
    }
}

/// `self / k` for a nonzero integer `k` of fewer bits than the width, over a modulus that fills
/// its width: the constant-time inverse of the element of `k`, then the product.
impl<M: Modulus, K> CtDiv<NonZeroUint<K>> for Fp<M>
where
    K: Bits + IsLess<M::Width, Output = True>,
{
    type Output = Self;

    #[inline(always)]
    fn ct_div(self, rhs: NonZeroUint<K>) -> Self {
        self.divide_with::<true>(rhs.get().as_limbs())
    }
}

/// `-self`.
impl<M: Modulus> CtNeg for Fp<M> {
    #[inline]
    fn ct_neg(self) -> Self {
        self.neg_with()
    }
}

/// `self * self`.
impl<M: Modulus> CtSquare for Fp<M> {
    #[inline]
    fn ct_square(self) -> Self {
        self.square_with::<true>()
    }
}

/// `self^exponent` for a public exponent, by sliding windows of odd powers over its bits.
impl<M: Modulus, E: Bits> CtPow<Uint<E>> for Fp<M> {
    #[inline]
    fn ct_pow(self, exponent: Uint<E>) -> Self {
        self.pow_with::<true, E>(exponent)
    }
}

/// The element, or `None` for `0`.
impl<M: Modulus> CtNonZero for Fp<M> {
    type NonZero = NonZero<Self>;

    #[inline]
    fn ct_nonzero(self) -> Option<NonZero<Self>> {
        NonZero::ct_new(self)
    }
}

/// A square root, or `None` for a quadratic non-residue. Of the two roots of a residue, which one
/// is returned is unspecified.
impl<M: Modulus> CtSqrt for Fp<M> {
    type Root = Option<ConstantTime<Self>>;

    #[inline]
    fn ct_sqrt(self) -> Option<ConstantTime<Self>> {
        self.sqrt_ct().map(ConstantTime::new)
    }
}

/// Implements an operator with a public `Fp` on the left and a [`ConstantTime`] on the right.
macro_rules! public_left {
    ($($op:ident, $method:ident, $ct:ident, $ct_method:ident;)*) => {
        $(
            impl<M: Modulus> $op<ConstantTime<Fp<M>>> for Fp<M> {
                type Output = ConstantTime<Self>;

                #[inline]
                fn $method(self, rhs: ConstantTime<Self>) -> ConstantTime<Self> {
                    ConstantTime::new($ct::$ct_method(self, rhs.reveal()))
                }
            }
        )*
    };
}

public_left!(
    Add, add, CtAdd, ct_add;
    Sub, sub, CtSub, ct_sub;
    Mul, mul, CtMul, ct_mul;
);
