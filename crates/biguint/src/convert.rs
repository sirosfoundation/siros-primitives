use typenum::U1;

use crate::bits::{Bits, Bits8, Bits16, Bits32, Bits64, Bits128};
use crate::uint::Uint;

/// The value needs more bits than the target width holds.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("the value needs more than {width} bits")]
pub struct OutOfRange {
    /// The target width in bits.
    pub width: usize,
}

impl<N: Bits> Uint<N> {
    /// The same value in `M` bits.
    ///
    /// # Errors
    ///
    /// [`OutOfRange`] when the value needs more than `M` bits.
    #[inline(always)]
    pub fn try_resize<M: Bits>(self) -> Result<Uint<M>, OutOfRange> {
        if self.bit_len() <= M::BITS {
            Ok(self.resize())
        } else {
            Err(OutOfRange { width: M::BITS })
        }
    }

    /// `value`, or `None` when it needs more than `N` bits.
    #[inline(always)]
    #[must_use]
    pub fn checked_from_u128(value: u128) -> Option<Self> {
        Uint::<Bits128>::from(value).try_resize().ok()
    }

    /// The value nearest to `self`, ties to even; infinity from `2^1024 - 2^970` up.
    #[inline(always)]
    #[must_use]
    pub fn to_f64(&self) -> f64 {
        let (word, shift) = self.sticky_top();
        // A power of two in the normal range scales exactly.
        match u64::try_from(shift) {
            Ok(shift @ 0..=1023) => word as f64 * f64::from_bits((1023 + shift) << 52),
            _ => f64::INFINITY,
        }
    }

    /// The value nearest to `self`, ties to even; infinity from `2^128 - 2^103` up.
    #[inline(always)]
    #[must_use]
    pub fn to_f32(&self) -> f32 {
        let (word, shift) = self.sticky_top();
        match u32::try_from(shift) {
            Ok(shift @ 0..=127) => word as f32 * f32::from_bits((127 + shift) << 23),
            _ => f32::INFINITY,
        }
    }

    /// The integer part of `value`, or `None` for NaN, `value <= -1` and an integer part that
    /// needs more than `N` bits.
    #[inline(always)]
    #[must_use]
    pub fn checked_from_f64(value: f64) -> Option<Self> {
        const FRACTION_BITS: u64 = f64::MANTISSA_DIGITS as u64 - 1;
        if value.is_nan() || value <= -1.0 || value == f64::INFINITY {
            return None;
        }
        if value < 1.0 {
            return Some(Self::ZERO);
        }
        let bits = value.to_bits();
        // `value >= 1`: the biased exponent is at least 1023, and the integer part has
        // `exponent + 1` bits.
        let exponent = (bits >> FRACTION_BITS) - 1023;
        let significand = bits & ((1 << FRACTION_BITS) - 1) | 1 << FRACTION_BITS;
        if exponent >= N::BITS as u64 {
            None
        } else if exponent >= FRACTION_BITS {
            Some(Self::from_u64(significand) << (exponent - FRACTION_BITS) as usize)
        } else {
            Some(Self::from_u64(significand >> (FRACTION_BITS - exponent)))
        }
    }

    /// The top 64 bits from the top set bit, with the lowest set when any bit below them is,
    /// and their shift: a float conversion rounds the word as it rounds the value.
    #[inline(always)]
    fn sticky_top(&self) -> (u64, usize) {
        let shift = self.bit_len().saturating_sub(64);
        let top = (*self >> shift).as_limbs().first().copied().unwrap_or(0);
        (top | u64::from(self.trailing_zeros() < shift), shift)
    }
}

macro_rules! from_primitive {
    ($($primitive:ty => $bits:ty),* $(,)?) => {
        $(
            /// The same value, in a `Uint` of the primitive's width.
            impl From<$primitive> for Uint<$bits> {
                #[inline(always)]
                fn from(value: $primitive) -> Self {
                    Self::from_u128(u128::from(value))
                }
            }
        )*
    };
}

from_primitive!(
    bool => U1,
    u8 => Bits8,
    u16 => Bits16,
    u32 => Bits32,
    u64 => Bits64,
    u128 => Bits128,
);

/// Refuses a value above `u64::MAX`.
impl<N: Bits> TryFrom<Uint<N>> for u64 {
    type Error = OutOfRange;

    #[inline(always)]
    fn try_from(value: Uint<N>) -> Result<Self, OutOfRange> {
        let [low] = value.try_resize::<Bits64>()?.into_limbs().into_array();
        Ok(low)
    }
}

/// Refuses a value above `u128::MAX`.
impl<N: Bits> TryFrom<Uint<N>> for u128 {
    type Error = OutOfRange;

    #[inline(always)]
    fn try_from(value: Uint<N>) -> Result<Self, OutOfRange> {
        let [low, high] = value.try_resize::<Bits128>()?.into_limbs().into_array();
        Ok(u128::from(low) | (u128::from(high) << 64))
    }
}
