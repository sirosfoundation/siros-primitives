use core::ops::Rem;

use typenum::U0;

use crate::bits::Bits;
use crate::shift::shift_right;
use crate::uint::Uint;

impl<N: Bits> Uint<N> {
    /// Rotates the `N` bits left by `shift mod N`.
    #[inline(always)]
    #[must_use]
    pub fn rotate_left(self, shift: usize) -> Self {
        let shift = shift % N::BITS;
        if shift == 0 {
            self
        } else {
            (self << shift) | (self >> (N::BITS - shift))
        }
    }

    /// Rotates the `N` bits right by `shift mod N`.
    #[inline(always)]
    #[must_use]
    pub fn rotate_right(self, shift: usize) -> Self {
        let shift = shift % N::BITS;
        if shift == 0 {
            self
        } else {
            (self >> shift) | (self << (N::BITS - shift))
        }
    }

    /// Reverses the order of the `N / 8` bytes.
    #[inline(always)]
    #[must_use]
    pub fn swap_bytes(self) -> Self
    where
        N: Rem<typenum::U8, Output = U0>,
    {
        // Reversing the bytes of all the limbs moves the zero bytes above `N` to the bottom;
        // the shift drops them.
        let mut reversed = self.limbs;
        reversed.reverse();
        reversed
            .iter_mut()
            .for_each(|limb| *limb = limb.swap_bytes());
        Self::from_generic_limbs(shift_right(
            &reversed,
            N::LIMBS * 64 - N::BITS,
            N::LIMBS * 64,
        ))
    }
}
