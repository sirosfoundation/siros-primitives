use generic_array::GenericArray;
use generic_array::sequence::GenericSequence;

use crate::bits::Bits;
use crate::convert::OutOfRange;
use crate::uint::Uint;

impl<N: Bits> Uint<N> {
    /// The little-endian encoding in `ceil(N / 8)` bytes.
    #[inline]
    #[must_use]
    pub fn to_le_bytes(&self) -> GenericArray<u8, N::Bytes> {
        GenericArray::generate(|index| self.byte(index))
    }

    /// The big-endian encoding in `ceil(N / 8)` bytes.
    #[inline]
    #[must_use]
    pub fn to_be_bytes(&self) -> GenericArray<u8, N::Bytes> {
        GenericArray::generate(|index| self.byte(N::BYTES - 1 - index))
    }

    /// Decodes [`Self::to_le_bytes`].
    ///
    /// # Errors
    ///
    /// [`OutOfRange`] when a bit at or above `N` is set.
    #[inline]
    pub fn from_le_bytes(bytes: &GenericArray<u8, N::Bytes>) -> Result<Self, OutOfRange> {
        Self::from_le_byte_iter(bytes.iter().copied())
    }

    /// Decodes [`Self::to_be_bytes`].
    ///
    /// # Errors
    ///
    /// [`OutOfRange`] when a bit at or above `N` is set.
    #[inline]
    pub fn from_be_bytes(bytes: &GenericArray<u8, N::Bytes>) -> Result<Self, OutOfRange> {
        Self::from_le_byte_iter(bytes.iter().rev().copied())
    }

    /// Byte `index`, counted from the least significant byte.
    #[inline(always)]
    fn byte(&self, index: usize) -> u8 {
        self.limbs
            .get(index / 8)
            .map_or(0, |limb| (limb >> (index % 8 * 8)) as u8)
    }

    /// Packs `ceil(N / 8)` little-endian bytes into limbs.
    #[inline(always)]
    fn from_le_byte_iter(mut bytes: impl Iterator<Item = u8>) -> Result<Self, OutOfRange> {
        let limbs = GenericArray::<u64, N::Limbs>::generate(|_| {
            bytes
                .by_ref()
                .take(8)
                .enumerate()
                .fold(0, |limb, (index, byte)| {
                    limb | (u64::from(byte) << (index * 8))
                })
        });
        if limbs.last().is_none_or(|top| top & !N::TOP_MASK == 0) {
            Ok(Self { limbs })
        } else {
            Err(OutOfRange { width: N::BITS })
        }
    }
}
