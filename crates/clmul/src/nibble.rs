//! GF(2)-linear maps of nibbles to bytes, applied to every byte of a polynomial by a vector byte
//! shuffle.

use crate::backend::{self, Register};

/// A GF(2)-linear map of nibbles to bytes in one vector register: entry `m` is the sum of the
/// images of the bits of `m`, so entry 0 is 0.
///
/// ```
/// use biguint::typenum::U8;
/// use biguint::Uint;
/// use clmul::{NibbleMap, Poly};
///
/// // The square of a byte in GF(2)[x]: bit `k` of each nibble to bit `2 k`.
/// let low = NibbleMap::from_images([0x01, 0x04, 0x10, 0x40]);
/// assert_eq!(low.entries()[0b1011], 0x45);
/// let a = Poly::<U8>::from_uint(Uint::from_u64(0b11));
/// assert_eq!(a.map_low_nibbles(low), Poly::from_uint(Uint::from_u64(0b101)));
/// ```
#[derive(Clone, Copy, Debug)]
pub struct NibbleMap {
    /// The 16 entries, entry `m` in byte `m`.
    entries: Register,
    /// The image of bit `k` in every byte of a word.
    images: [u64; 4],
}

impl NibbleMap {
    /// The map that sends bit `k` of a nibble to `images[k]`.
    #[must_use]
    pub const fn from_images(images: [u8; 4]) -> Self {
        let mut entries = [0u8; 16];
        let mut nibble = 0;
        while nibble < 16 {
            let mut entry = 0;
            let mut bit = 0;
            while bit < 4 {
                if (nibble >> bit) & 1 == 1
                    && let Some(&image) = images.as_slice().split_at(bit).1.first()
                {
                    entry ^= image;
                }
                bit += 1;
            }
            if let Some(slot) = entries.as_mut_slice().split_at_mut(nibble).1.first_mut() {
                *slot = entry;
            }
            nibble += 1;
        }
        let [i0, i1, i2, i3] = images;

        Self {
            entries: backend::register([
                u64::from_le_bytes(first_half(entries)),
                u64::from_le_bytes(second_half(entries)),
            ]),
            images: [splat(i0), splat(i1), splat(i2), splat(i3)],
        }
    }

    /// The 16 entries, entry `m` the image of the nibble `m`.
    #[must_use]
    pub const fn entries(self) -> [u8; 16] {
        let [low, high] = backend::words(self.entries);
        let (low, high) = (low.to_le_bytes(), high.to_le_bytes());
        let mut out = [0u8; 16];
        let mut index = 0;
        while index < 8 {
            if let (Some(&a), Some(&b)) = (
                low.as_slice().split_at(index).1.first(),
                high.as_slice().split_at(index).1.first(),
            ) {
                if let Some(slot) = out.as_mut_slice().split_at_mut(index).1.first_mut() {
                    *slot = a;
                }
                if let Some(slot) = out.as_mut_slice().split_at_mut(index + 8).1.first_mut() {
                    *slot = b;
                }
            }
            index += 1;
        }
        out
    }

    /// Every byte of `nibbles`, each below 16, replaced by its image: a byte shuffle where the
    /// build has one, the sum of the images selected by the bits elsewhere.
    #[inline(always)]
    pub(crate) fn apply(self, nibbles: Register) -> Register {
        backend::map_nibbles(self.entries, self.images, nibbles)
    }
}

/// `byte` in every byte of a word.
const fn splat(byte: u8) -> u64 {
    u64::from_ne_bytes([byte; 8])
}

/// Bytes 0 to 7.
const fn first_half(bytes: [u8; 16]) -> [u8; 8] {
    let [b0, b1, b2, b3, b4, b5, b6, b7, ..] = bytes;
    [b0, b1, b2, b3, b4, b5, b6, b7]
}

/// Bytes 8 to 15.
const fn second_half(bytes: [u8; 16]) -> [u8; 8] {
    let [.., b8, b9, b10, b11, b12, b13, b14, b15] = bytes;
    [b8, b9, b10, b11, b12, b13, b14, b15]
}

/// Every byte of the word `nibbles`, each below 16, replaced by its image under the map whose image
/// of bit `k` fills every byte of `images[k]`: the bit selects the image by a mask, in constant
/// time.
#[cfg(any(
    all(target_arch = "x86_64", not(target_feature = "ssse3")),
    not(any(
        target_arch = "x86_64",
        all(
            target_arch = "aarch64",
            target_endian = "little",
            target_feature = "aes"
        )
    ))
))]
#[inline(always)]
pub(crate) fn map_word(images: [u64; 4], nibbles: u64) -> u64 {
    const BYTES: u64 = 0x0101_0101_0101_0101;
    let [i0, i1, i2, i3] = images;
    let select = |bit: u32, image: u64| ((nibbles >> bit) & BYTES).wrapping_mul(0xFF) & image;
    select(0, i0) ^ select(1, i1) ^ select(2, i2) ^ select(3, i3)
}
