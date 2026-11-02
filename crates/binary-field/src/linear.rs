//! Constant GF(2)-linear maps of the elements of the fields up to degree 16, in a general register,
//! in constant time: each output byte holds the parities of an input byte under eight row masks.
//! A product spreads the byte to every byte of a word, a mask picks each row's bits, three
//! shift-and-adds leave each row's parity in the low bit of its byte, and a second product
//! gathers the eight parities into one byte (Warren, Hacker's Delight, 7-1 and 7-6).

/// A map of 16 bits to 16 bits: `blocks[i][o]` maps input byte `i` to output byte `o`, and byte
/// `r` of a block holds the input bits whose sum is bit `r` of the output byte.
pub(crate) type Blocks = [[u64; 2]; 2];

/// One byte in every byte of a word.
const SPREAD: u64 = 0x0101_0101_0101_0101;

/// Gathers bit `8 r` of a word to bit `56 + r`, for every `r` below 8: the terms of the product
/// fall on distinct positions, so no carry reaches the top byte.
const GATHER: u64 = 0x0102_0408_1020_4080;

/// The parities of `byte` under the eight rows of `rows`: bit `r` of the result.
#[inline(always)]
const fn parities(rows: u64, byte: u64) -> u64 {
    let hit = rows & byte.wrapping_mul(SPREAD);
    let mut parity = hit ^ (hit >> 4);
    parity ^= parity >> 2;
    parity ^= parity >> 1;
    (parity & SPREAD).wrapping_mul(GATHER) >> 56
}

/// The image of `a`, below `2^degree`, under `blocks`.
#[inline(always)]
pub(crate) const fn apply(blocks: &Blocks, a: u64, degree: usize) -> u64 {
    let [[low_low, low_high], [high_low, high_high]] = *blocks;
    let low = a & 0xFF;
    if degree <= 8 {
        parities(low_low, low)
    } else {
        let high = a >> 8;
        parities(low_low, low)
            ^ parities(high_low, high)
            ^ ((parities(low_high, low) ^ parities(high_high, high)) << 8)
    }
}

/// The blocks of the map that sends `x^j` to `images[j]`.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its array's length"
)]
pub(crate) const fn blocks(images: &[u64; 16]) -> Blocks {
    let mut blocks = [[0; 2]; 2];
    let mut input = 0;
    while input < 16 {
        let mut output = 0;
        while output < 16 {
            if (images[input] >> output) & 1 == 1 {
                let (block, row) = (output / 8, output % 8);
                let row_bits = &mut blocks[input / 8][block];
                *row_bits |= 1 << (8 * row + input % 8);
            }
            output += 1;
        }
        input += 1;
    }
    blocks
}
