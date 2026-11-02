//! Elements whose operations take one path for every value.

use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use crate::arith::{Element, Field};
use crate::element::Gf2;
use crate::invert::declassify;
use crate::plan::Polynomial;
use crate::representation::Storage;

/// An element of `GF(2^n)` whose operations take time that depends on no value: no branch and no
/// memory address depends on it.
///
/// It holds a [`Gf2<P>`] in the same registers. The sum, the difference, the product, `square`,
/// `sqrt` and `invert` run on carry-less products, shifts and masks; the operators also take a
/// public `Gf2<P>` on either side. `invert` discloses whether the element is zero, through the
/// `Option` it returns, and nothing else.
///
/// ```
/// use binary_field::{Gf2p128, Secret};
///
/// let key = Secret::new(Gf2p128::from_u128(0x1234_5678).unwrap());
/// let public = Gf2p128::X;
/// let product = key * public;
/// assert_eq!((product * key.invert().unwrap()).reveal(), public);
/// assert_eq!(key.sqrt().square().reveal(), key.reveal());
/// ```
#[repr(transparent)]
#[derive_where::derive_where(Clone, Copy; Element<P>)]
pub struct Secret<P: Polynomial> {
    element: Gf2<P>,
}

impl<P: Polynomial> Secret<P> {
    /// The secret of `element`.
    #[inline(always)]
    #[must_use]
    pub const fn new(element: Gf2<P>) -> Self {
        Self { element }
    }

    /// The element.
    #[inline(always)]
    #[must_use]
    pub const fn reveal(self) -> Gf2<P> {
        self.element
    }

    /// The secret of the registers `value`.
    #[inline(always)]
    const fn of(value: Element<P>) -> Self {
        Self::new(Gf2::of(value))
    }

    /// The registers.
    #[inline(always)]
    const fn value(self) -> Element<P> {
        self.element.value()
    }

    /// `self * self`.
    #[inline(always)]
    #[must_use]
    pub fn square(self) -> Self {
        Self::of(Field::<P>::ct_square(self.value()))
    }

    /// The square root.
    #[inline(always)]
    #[must_use]
    pub fn sqrt(self) -> Self {
        Self::of(Field::<P>::ct_sqrt(self.value()))
    }

    /// The multiplicative inverse, or `None` for `0`. Only the `Option` depends on the value.
    #[inline(always)]
    #[must_use]
    pub fn invert(self) -> Option<Self> {
        let value = self.value();
        let inverse = Self::of(Field::<P>::ct_invert(value));
        declassify(!value.is_zero(), inverse)
    }
}

/// Implements an operator between two secrets, and between a secret and a public element on
/// either side, with its assignment forms.
macro_rules! operator {
    ($trait:ident, $method:ident, $assign_trait:ident, $assign:ident, |$a:ident, $b:ident| $body:expr) => {
        impl<P: Polynomial> $trait for Secret<P> {
            type Output = Self;

            #[inline(always)]
            fn $method(self, rhs: Self) -> Self {
                let ($a, $b) = (self.value(), rhs.value());
                Self::of($body)
            }
        }

        impl<P: Polynomial> $trait<Gf2<P>> for Secret<P> {
            type Output = Self;

            #[inline(always)]
            fn $method(self, rhs: Gf2<P>) -> Self {
                $trait::$method(self, Self::new(rhs))
            }
        }

        impl<P: Polynomial> $trait<Secret<P>> for Gf2<P> {
            type Output = Secret<P>;

            #[inline(always)]
            fn $method(self, rhs: Secret<P>) -> Secret<P> {
                $trait::$method(Secret::new(self), rhs)
            }
        }

        impl<P: Polynomial> $assign_trait for Secret<P> {
            #[inline(always)]
            fn $assign(&mut self, rhs: Self) {
                *self = $trait::$method(*self, rhs);
            }
        }

        impl<P: Polynomial> $assign_trait<Gf2<P>> for Secret<P> {
            #[inline(always)]
            fn $assign(&mut self, rhs: Gf2<P>) {
                *self = $trait::$method(*self, rhs);
            }
        }
    };
}

operator!(Add, add, AddAssign, add_assign, |a, b| a.add(b));
operator!(Sub, sub, SubAssign, sub_assign, |a, b| a.add(b));
operator!(Mul, mul, MulAssign, mul_assign, |a, b| Field::<P>::ct_mul(
    a, b
));

impl<P: Polynomial> Neg for Secret<P> {
    type Output = Self;

    #[inline(always)]
    fn neg(self) -> Self {
        self
    }
}

impl<P: Polynomial, T> From<T> for Secret<P>
where
    Gf2<P>: From<T>,
{
    /// The secret of the element of `value`.
    #[inline(always)]
    fn from(value: T) -> Self {
        Self::new(Gf2::from(value))
    }
}
