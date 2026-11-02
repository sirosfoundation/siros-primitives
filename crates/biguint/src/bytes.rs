//! Byte encodings, limb by limb: a little-endian encoding copies each limb's bytes, and a
//! big-endian one swaps them. Only the top limb of a width that is not a multiple of 64 bits
//! contributes part of its bytes.

use generic_array::GenericArray;
use generic_array::sequence::GenericSequence;

use crate::bits::Bits;
use crate::convert::OutOfRange;
use crate::uint::Uint;

impl<N: Bits> Uint<N> {
    /// The little-endian encoding in `ceil(N / 8)` bytes.
    #[inline(always)]
    #[must_use]
    pub fn to_le_bytes(&self) -> GenericArray<u8, N::Bytes> {
        let mut bytes = GenericArray::<u8, N::Bytes>::default();
        let (chunks, top) = bytes.as_chunks_mut::<8>();
        for (chunk, limb) in chunks.iter_mut().zip(&self.limbs) {
            *chunk = limb.to_le_bytes();
        }
        // The low bytes of the top limb, which `chunks` leave out.
        if let Some(limb) = self.limbs.get(chunks.len()) {
            for (slot, &byte) in top.iter_mut().zip(&limb.to_le_bytes()) {
                *slot = byte;
            }
        }
        bytes
    }

    /// The big-endian encoding in `ceil(N / 8)` bytes.
    #[inline(always)]
    #[must_use]
    pub fn to_be_bytes(&self) -> GenericArray<u8, N::Bytes> {
        let mut bytes = GenericArray::<u8, N::Bytes>::default();
        let (top, chunks) = bytes.as_rchunks_mut::<8>();
        for (chunk, limb) in chunks.iter_mut().rev().zip(&self.limbs) {
            *chunk = limb.to_be_bytes();
        }
        // The low bytes of the top limb lead, with its high zero bytes left out.
        if let Some(limb) = self.limbs.get(chunks.len()) {
            for (slot, &byte) in top.iter_mut().rev().zip(limb.to_le_bytes().iter()) {
                *slot = byte;
            }
        }
        bytes
    }

    /// Decodes [`Self::to_le_bytes`].
    ///
    /// # Errors
    ///
    /// [`OutOfRange`] when a bit at or above `N` is set.
    #[inline(always)]
    pub fn from_le_bytes(bytes: &GenericArray<u8, N::Bytes>) -> Result<Self, OutOfRange> {
        let (chunks, top) = bytes.as_chunks::<8>();
        // Past the chunks, the low bytes of the top limb.
        Self::checked(GenericArray::generate(|index| {
            chunks.get(index).map_or_else(
                || {
                    top.iter()
                        .rev()
                        .fold(0, |value, &byte| (value << 8) | u64::from(byte))
                },
                |chunk| u64::from_le_bytes(*chunk),
            )
        }))
    }

    /// Decodes [`Self::to_be_bytes`].
    ///
    /// # Errors
    ///
    /// [`OutOfRange`] when a bit at or above `N` is set.
    #[inline(always)]
    pub fn from_be_bytes(bytes: &GenericArray<u8, N::Bytes>) -> Result<Self, OutOfRange> {
        let (top, chunks) = bytes.as_rchunks::<8>();
        // Chunk `index` from the end; past the chunks, the low bytes of the top limb.
        Self::checked(GenericArray::generate(|index| {
            chunks
                .len()
                .checked_sub(index + 1)
                .and_then(|chunk| chunks.get(chunk))
                .map_or_else(
                    || {
                        top.iter()
                            .fold(0, |value, &byte| (value << 8) | u64::from(byte))
                    },
                    |chunk| u64::from_be_bytes(*chunk),
                )
        }))
    }

    /// The value of `limbs`, refused when a bit at or above `N` is set.
    #[inline(always)]
    fn checked(limbs: GenericArray<u64, N::Limbs>) -> Result<Self, OutOfRange> {
        if limbs.last().is_none_or(|top| top & !N::TOP_MASK == 0) {
            Ok(Self { limbs })
        } else {
            Err(OutOfRange { width: N::BITS })
        }
    }
}
