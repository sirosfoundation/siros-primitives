//! The constant-time operations of `Fp`, through which [`ConstantTime<Fp<M>>`](ConstantTime)
//! computes: the kernels that take one path for every value.

use core::ops::{Add, Mul, Sub};

use biguint::{Bits, Uint};
use constant_time::{
    ConstantTime, CtAdd, CtInvert, CtMul, CtNeg, CtPow, CtSqrt, CtSquare, CtSub, Public,
};

use crate::field::{Fp, Modulus};

impl<M: Modulus> Public for Fp<M> {}

/// `self + rhs`.
impl<M: Modulus> CtAdd for Fp<M> {
    type Output = Self;

    #[inline(always)]
    fn ct_add(self, rhs: Self) -> Self {
        self.add_with(rhs)
    }
}

/// `self - rhs`.
impl<M: Modulus> CtSub for Fp<M> {
    type Output = Self;

    #[inline(always)]
    fn ct_sub(self, rhs: Self) -> Self {
        self.sub_with(rhs)
    }
}

/// `self * rhs`.
impl<M: Modulus> CtMul for Fp<M> {
    type Output = Self;

    #[inline(always)]
    fn ct_mul(self, rhs: Self) -> Self {
        self.mul_with::<true>(rhs)
    }
}

/// `-self`.
impl<M: Modulus> CtNeg for Fp<M> {
    #[inline(always)]
    fn ct_neg(self) -> Self {
        self.neg_with()
    }
}

/// `self * self`.
impl<M: Modulus> CtSquare for Fp<M> {
    #[inline(always)]
    fn ct_square(self) -> Self {
        self.square_with::<true>()
    }
}

/// `self^exponent` for a public exponent, by sliding windows of odd powers over its bits.
impl<M: Modulus, E: Bits> CtPow<Uint<E>> for Fp<M> {
    #[inline(always)]
    fn ct_pow(self, exponent: Uint<E>) -> Self {
        self.pow_with::<true, E>(exponent)
    }
}

/// `self^-1`, or `None` for `0`.
impl<M: Modulus> CtInvert for Fp<M> {
    #[inline(always)]
    fn ct_invert(self) -> Option<Self> {
        self.invert_ct()
    }
}

/// A square root, or `None` for a quadratic non-residue. Of the two roots of a residue, which one
/// is returned is unspecified.
impl<M: Modulus> CtSqrt for Fp<M> {
    type Root = Option<ConstantTime<Self>>;

    #[inline(always)]
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

                #[inline(always)]
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
