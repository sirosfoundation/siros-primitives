//! Numeric traits shared by the integers here and the types built on them.

use core::ops::Mul;

/// The multiplicative identity, as a compile-time constant. `ONE` is not zero:
/// [`NonZero`](crate::NonZero) takes it as its own.
pub trait One {
    /// `1`.
    const ONE: Self;
}

/// A fixed-width source of exponent bits for [`Pow`].
///
/// `WIDTH` is a compile-time constant, so the square-and-multiply fold has a constant trip count.
pub trait Exponent {
    /// The number of exponent bits.
    const WIDTH: usize;

    /// Bit `index`, counted from the least significant bit, for `index < WIDTH`.
    fn bit(&self, index: usize) -> bool;
}

macro_rules! impl_exponent_for_primitive {
    ($($primitive:ty),* $(,)?) => {
        $(
            impl Exponent for $primitive {
                const WIDTH: usize = <$primitive>::BITS as usize;

                #[inline(always)]
                fn bit(&self, index: usize) -> bool {
                    (*self >> index) & 1 != 0
                }
            }
        )*
    };
}

impl_exponent_for_primitive!(u32, u64);

/// `self * self`, for types whose square is cheaper than a general product. The output may be
/// wider than the input.
pub trait Square {
    /// The square's type.
    type Output;

    /// `self * self`.
    #[must_use]
    fn square(self) -> Self::Output;
}

/// `self^exponent`, by square-and-multiply over the exponent bits from the most significant.
pub trait Pow<E> {
    /// `self^exponent`.
    #[must_use]
    fn pow(self, exponent: E) -> Self;
}

impl<T, E> Pow<E> for T
where
    T: Copy + Mul<Output = T> + One + Square<Output = T>,
    E: Exponent,
{
    #[inline(always)]
    fn pow(self, exponent: E) -> Self {
        (0..E::WIDTH).rev().fold(T::ONE, |acc, index| {
            let square = acc.square();
            if exponent.bit(index) {
                square * self
            } else {
                square
            }
        })
    }
}
