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
