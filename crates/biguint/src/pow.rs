//! `Uint`'s place in the [`Pow`](crate::Pow) blanket implementation.

use crate::bits::Bits;
use crate::num::{Exponent, One, Square};
use crate::uint::Uint;

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
