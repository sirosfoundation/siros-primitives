//! The constant-time operations of `Gf2`, through which [`ConstantTime<Gf2<P>>`](ConstantTime)
//! computes: carry-less products, shifts and masks, which take one path for every value.

use core::ops::{Add, Mul, Sub};

use constant_time::{ConstantTime, CtAdd, CtInvert, CtMul, CtNeg, CtSqrt, CtSquare, CtSub, Public};

use crate::arith::Field;
use crate::element::Gf2;
use crate::invert::declassify;
use crate::plan::Polynomial;
use crate::representation::Storage;

impl<P: Polynomial> Public for Gf2<P> {}

/// `self + rhs`: the XOR of the coefficients.
impl<P: Polynomial> CtAdd for Gf2<P> {
    type Output = Self;

    #[inline(always)]
    fn ct_add(self, rhs: Self) -> Self {
        Self::of(self.value().add(rhs.value()))
    }
}

/// `self - rhs`, which equals `self + rhs`.
impl<P: Polynomial> CtSub for Gf2<P> {
    type Output = Self;

    #[inline(always)]
    fn ct_sub(self, rhs: Self) -> Self {
        self.ct_add(rhs)
    }
}

/// `self * rhs`.
impl<P: Polynomial> CtMul for Gf2<P> {
    type Output = Self;

    #[inline(always)]
    fn ct_mul(self, rhs: Self) -> Self {
        Self::of(Field::<P>::ct_mul(self.value(), rhs.value()))
    }
}

/// `-self`, which equals `self`.
impl<P: Polynomial> CtNeg for Gf2<P> {
    #[inline(always)]
    fn ct_neg(self) -> Self {
        self
    }
}

/// `self * self`.
impl<P: Polynomial> CtSquare for Gf2<P> {
    #[inline(always)]
    fn ct_square(self) -> Self {
        Self::of(Field::<P>::ct_square(self.value()))
    }
}

/// The multiplicative inverse, or `None` for `0`. Only the `Option` depends on the value.
impl<P: Polynomial> CtInvert for Gf2<P> {
    #[inline(always)]
    fn ct_invert(self) -> Option<Self> {
        let value = self.value();
        let inverse = Self::of(Field::<P>::ct_invert(value));
        declassify(!value.is_zero(), inverse)
    }
}

/// The square root, which every element has.
impl<P: Polynomial> CtSqrt for Gf2<P> {
    type Root = ConstantTime<Self>;

    #[inline(always)]
    fn ct_sqrt(self) -> ConstantTime<Self> {
        ConstantTime::new(Self::of(Field::<P>::ct_sqrt(self.value())))
    }
}

/// Implements an operator with a public `Gf2` on the left and a [`ConstantTime`] on the right.
macro_rules! public_left {
    ($($op:ident, $method:ident, $ct:ident, $ct_method:ident;)*) => {
        $(
            impl<P: Polynomial> $op<ConstantTime<Gf2<P>>> for Gf2<P> {
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
