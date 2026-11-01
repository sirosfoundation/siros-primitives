use core::ops::{Shl, Shr};

use crate::bits::Bits;
use crate::stream::LimbStream;
use crate::uint::Uint;

/// `(self * 2^rhs) mod 2^N`: `0` when `rhs >= N`.
impl<N: Bits> Shl<usize> for Uint<N> {
    type Output = Self;

    #[inline(always)]
    fn shl(self, rhs: usize) -> Self::Output {
        Uint::from_stream(ShiftLeftStream::new(self.limbs.as_slice(), rhs))
    }
}

/// `floor(self / 2^rhs)`: `0` when `rhs >= N`.
impl<N: Bits> Shr<usize> for Uint<N> {
    type Output = Self;

    #[inline(always)]
    fn shr(self, rhs: usize) -> Self::Output {
        Uint::from_stream(ShiftRightStream::new(self.limbs.as_slice(), rhs))
    }
}

/// `(self * 2^rhs) mod 2^N`: `0` when `rhs >= N`.
impl<N: Bits, RhsN: Bits> Shl<Uint<RhsN>> for Uint<N> {
    type Output = Self;

    #[inline(always)]
    fn shl(self, rhs: Uint<RhsN>) -> Self::Output {
        shift_amount::<N, RhsN>(rhs).map_or(Self::ZERO, |amount| self << amount)
    }
}

/// `floor(self / 2^rhs)`: `0` when `rhs >= N`.
impl<N: Bits, RhsN: Bits> Shr<Uint<RhsN>> for Uint<N> {
    type Output = Self;

    #[inline(always)]
    fn shr(self, rhs: Uint<RhsN>) -> Self::Output {
        shift_amount::<N, RhsN>(rhs).map_or(Self::ZERO, |amount| self >> amount)
    }
}

/// `rhs` as a shift amount when it is below `N`.
#[inline(always)]
fn shift_amount<N: Bits, RhsN: Bits>(rhs: Uint<RhsN>) -> Option<usize> {
    let mut limbs = rhs.limbs.into_iter();
    let low = limbs.next().unwrap_or(0);
    (limbs.all(|limb| limb == 0) && low < N::BITS as u64).then_some(low as usize)
}

/// The limbs of `limbs * 2^shift`, then `0` forever.
pub(crate) struct ShiftLeftStream<'a> {
    limbs: &'a [u64],
    limb_shift: usize,
    bit_shift: usize,
    index: usize,
}

impl<'a> ShiftLeftStream<'a> {
    #[inline(always)]
    pub(crate) const fn new(limbs: &'a [u64], shift: usize) -> Self {
        Self {
            limbs,
            limb_shift: shift / 64,
            bit_shift: shift % 64,
            index: 0,
        }
    }
}

impl LimbStream for ShiftLeftStream<'_> {
    #[inline(always)]
    fn next_limb(&mut self) -> u64 {
        let Some(source_index) = self.index.checked_sub(self.limb_shift) else {
            self.index += 1;
            return 0;
        };

        let low = self.limbs.get(source_index).copied().unwrap_or(0);
        let high = if self.bit_shift == 0 {
            0
        } else {
            self.limbs
                .get(source_index.wrapping_sub(1))
                .copied()
                .unwrap_or(0)
                >> (64 - self.bit_shift)
        };
        self.index += 1;
        (low << self.bit_shift) | high
    }
}

/// The limbs of `floor(limbs / 2^shift)`, then `0` forever.
pub(crate) struct ShiftRightStream<'a> {
    limbs: &'a [u64],
    limb_shift: usize,
    bit_shift: usize,
    index: usize,
}

impl<'a> ShiftRightStream<'a> {
    #[inline(always)]
    pub(crate) const fn new(limbs: &'a [u64], shift: usize) -> Self {
        Self {
            limbs,
            limb_shift: shift / 64,
            bit_shift: shift % 64,
            index: 0,
        }
    }
}

impl LimbStream for ShiftRightStream<'_> {
    #[inline(always)]
    fn next_limb(&mut self) -> u64 {
        let source_index = self.index + self.limb_shift;
        let low = self.limbs.get(source_index).copied().unwrap_or(0);
        let high = if self.bit_shift == 0 {
            0
        } else {
            source_index
                .checked_add(1)
                .and_then(|index| self.limbs.get(index))
                .copied()
                .unwrap_or(0)
                << (64 - self.bit_shift)
        };
        self.index += 1;
        (low >> self.bit_shift) | high
    }
}
