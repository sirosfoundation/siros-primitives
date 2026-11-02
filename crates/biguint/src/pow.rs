//! Powers of a `Uint`: [`Uint::wrapping_pow`], and its place in the [`Pow`](crate::Pow) blanket
//! implementation.

use crate::bits::Bits;
use crate::num::{Exponent, One, Square};
use crate::uint::Uint;

impl<N: Bits> Uint<N> {
    /// `self^exponent mod 2^N`, by square-and-multiply from the exponent's top set bit.
    #[inline(always)]
    #[must_use]
    pub fn wrapping_pow(self, exponent: u32) -> Self {
        (0..u32::BITS - exponent.leading_zeros())
            .rev()
            .fold(Self::ONE, |power, bit| {
                let square = power * power;
                if exponent >> bit & 1 == 1 {
                    square * self
                } else {
                    square
                }
            })
    }
}

impl<N: Bits> One for Uint<N> {
    const ONE: Self = Self::ONE;
}

/// `(self * self) mod 2^N`.
impl<N: Bits> Square for Uint<N> {
    type Output = Self;

    #[inline(always)]
    fn square(self) -> Self::Output {
        self * self
    }
}

impl<N: Bits> Exponent for Uint<N> {
    const WIDTH: usize = N::BITS;

    #[inline(always)]
    fn bit(&self, index: usize) -> bool {
        self.bit_at(index)
    }
}
