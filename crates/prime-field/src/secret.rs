use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use biguint::{Bits, Uint};

use crate::field::{Fp, Modulus};
use crate::params::Limbs;

/// An element of the prime field of `M` whose operations run in constant time: no branch and no
/// memory address depends on its value.
///
/// [`Secret::new`] and [`Secret::reveal`] convert at no cost. The operators combine two
/// `Secret`s, or a `Secret` and a public [`Fp`], into a `Secret`. The exponent of
/// [`Secret::pow`] is public. [`Secret::invert`] and [`Secret::sqrt`] disclose whether their
/// result exists, and nothing else.
#[repr(transparent)]
#[derive_where::derive_where(Clone, Copy; Limbs<M>)]
pub struct Secret<M: Modulus>(Fp<M>);

impl<M: Modulus> Secret<M> {
    /// `value`, as a secret.
    #[inline(always)]
    #[must_use]
    pub const fn new(value: Fp<M>) -> Self {
        Self(value)
    }

    /// The value.
    #[inline(always)]
    #[must_use]
    pub const fn reveal(self) -> Fp<M> {
        self.0
    }

    /// `self * self`.
    #[inline(always)]
    #[must_use]
    pub fn square(self) -> Self {
        Self(self.0.square_with::<true>())
    }

    /// `self + self`.
    #[inline(always)]
    #[must_use]
    pub fn double(self) -> Self {
        Self(self.0.add_with(self.0))
    }

    /// `self^exponent` for a public exponent, by sliding windows of odd powers over its bits.
    #[inline(always)]
    #[must_use]
    pub fn pow<E: Bits>(self, exponent: Uint<E>) -> Self {
        Self(self.0.pow_with::<true, E>(exponent))
    }

    /// `self^-1`, or `None` for `0`.
    #[inline(always)]
    #[must_use]
    pub fn invert(self) -> Option<Self> {
        self.0.invert_ct().map(Self)
    }

    /// A square root of `self`, or `None` when `self` is a quadratic non-residue. Of the two
    /// roots of a residue, which one is returned is unspecified.
    #[inline(always)]
    #[must_use]
    pub fn sqrt(self) -> Option<Self> {
        self.0.sqrt_ct().map(Self)
    }
}

/// Implements an operator between two `Secret`s, and between a `Secret` and an `Fp` in either
/// order, by the constant-time kernel `$kernel`, with its assigning form.
macro_rules! operator {
    ($trait:ident, $method:ident, $assign_trait:ident, $assign:ident, $kernel:ident $(::<$ct:literal>)?) => {
        impl<M: Modulus> $trait for Secret<M> {
            type Output = Self;

            #[inline(always)]
            fn $method(self, rhs: Self) -> Self {
                Self(self.0.$kernel $(::<$ct>)? (rhs.0))
            }
        }

        impl<M: Modulus> $trait<Fp<M>> for Secret<M> {
            type Output = Self;

            #[inline(always)]
            fn $method(self, rhs: Fp<M>) -> Self {
                Self(self.0.$kernel $(::<$ct>)? (rhs))
            }
        }

        impl<M: Modulus> $trait<Secret<M>> for Fp<M> {
            type Output = Secret<M>;

            #[inline(always)]
            fn $method(self, rhs: Secret<M>) -> Secret<M> {
                Secret(self.$kernel $(::<$ct>)? (rhs.0))
            }
        }

        impl<M: Modulus> $assign_trait for Secret<M> {
            #[inline(always)]
            fn $assign(&mut self, rhs: Self) {
                self.0 = self.0.$kernel $(::<$ct>)? (rhs.0);
            }
        }

        impl<M: Modulus> $assign_trait<Fp<M>> for Secret<M> {
            #[inline(always)]
            fn $assign(&mut self, rhs: Fp<M>) {
                self.0 = self.0.$kernel $(::<$ct>)? (rhs);
            }
        }
    };
}

operator!(Add, add, AddAssign, add_assign, add_with);
operator!(Sub, sub, SubAssign, sub_assign, sub_with);
operator!(Mul, mul, MulAssign, mul_assign, mul_with::<true>);

impl<M: Modulus> Neg for Secret<M> {
    type Output = Self;

    #[inline(always)]
    fn neg(self) -> Self {
        Self(self.0.neg_with())
    }
}

impl<M: Modulus> From<Fp<M>> for Secret<M> {
    /// `value`, as a secret.
    #[inline(always)]
    fn from(value: Fp<M>) -> Self {
        Self::new(value)
    }
}
