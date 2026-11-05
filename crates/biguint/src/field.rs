//! The inversion of nonzero field elements: [`NonZero`] of a [`Field`] inverts without a zero
//! test, and a product of two stays nonzero.

use core::ops::Mul;

use constant_time::{CtInvert, CtNonZero, declassify};
use num_traits::Inv;

use crate::nonzero::{Divisor, NonZero};

/// A field element type, through which [`NonZero`] inverts it. `Zero::is_zero` runs in time
/// independent of the value.
#[doc(hidden)]
pub trait Field:
    Divisor<Stored = Self> + Mul<Output = Self> + CtNonZero<NonZero = NonZero<Self>>
{
    /// `self^-1`; never called on `0`.
    #[must_use]
    fn invert_nonzero(self) -> Self;

    /// `self^-1` in time independent of the value; never called on `0`.
    #[must_use]
    fn ct_invert_nonzero(self) -> Self;
}

impl<T: Field> NonZero<T> {
    /// `self^-1`.
    #[inline(always)]
    #[must_use]
    pub fn invert(self) -> Self {
        Self(self.0.invert_nonzero())
    }
}

impl<T: Divisor<Stored = T>> NonZero<T> {
    /// `value`, or `None` when it is zero, for a `T` whose `is_zero` runs in time independent of
    /// the value: only the `Option` depends on it, built in [`declassify`]. The implementations of
    /// [`CtNonZero`] call it.
    #[doc(hidden)]
    #[inline(always)]
    #[must_use]
    pub fn ct_new(value: T) -> Option<Self> {
        declassify(!value.is_zero(), Self(value))
    }
}

/// `self^-1`.
impl<T: Field> Inv for NonZero<T> {
    type Output = Self;

    #[inline(always)]
    fn inv(self) -> Self {
        self.invert()
    }
}

/// A product of nonzero field elements is nonzero.
impl<T: Field> Mul for NonZero<T> {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self {
        Self(self.0 * rhs.0)
    }
}

/// `self^-1`.
impl<T: Field> CtInvert for NonZero<T> {
    #[inline(always)]
    fn ct_invert(self) -> Self {
        Self(self.0.ct_invert_nonzero())
    }
}
