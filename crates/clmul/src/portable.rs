//! The backend of builds without PCLMULQDQ or PMULL: a register is two `u64` words, and the word
//! products come from the comb.

pub(crate) use crate::comb::clmul64;
use crate::{REDUCTION, to_limbs};

/// A register: two 64-bit words, word 0 first.
pub(crate) type Register = [u64; 2];

/// The register of zeros.
pub(crate) const ZERO: Register = [0, 0];

/// The register of the words `[low, high]`.
#[inline(always)]
pub(crate) const fn register(words: [u64; 2]) -> Register {
    words
}

/// `register`, whose bits at and above `bits` are zero, with a high word of a literal zero for
/// `bits` up to 64: the operations on that word then fold away.
#[inline(always)]
pub(crate) const fn narrow(register: Register, bits: usize) -> Register {
    if bits <= 64 {
        [register[0], 0]
    } else {
        register
    }
}

/// The words `[low, high]` of `register`.
#[inline(always)]
pub(crate) const fn words(register: Register) -> [u64; 2] {
    register
}

/// `a ^ b`.
#[inline(always)]
pub(crate) const fn xor(a: Register, b: Register) -> Register {
    [a[0] ^ b[0], a[1] ^ b[1]]
}

/// `a & b`.
#[inline(always)]
pub(crate) const fn and(a: Register, b: Register) -> Register {
    [a[0] & b[0], a[1] & b[1]]
}

/// `a | b`.
#[inline(always)]
pub(crate) const fn or(a: Register, b: Register) -> Register {
    [a[0] | b[0], a[1] | b[1]]
}

/// Each word shifted left by `bits`, below 64.
#[inline(always)]
pub(crate) const fn shl_words(a: Register, bits: u32) -> Register {
    [a[0] << bits, a[1] << bits]
}

/// Each word shifted right by `bits`, below 64.
#[inline(always)]
pub(crate) const fn shr_words(a: Register, bits: u32) -> Register {
    [a[0] >> bits, a[1] >> bits]
}

/// `[0, low]`: word 0 moved up to word 1.
#[inline(always)]
pub(crate) const fn up(a: Register) -> Register {
    [0, a[0]]
}

/// `[high, 0]`: word 1 moved down to word 0.
#[inline(always)]
pub(crate) const fn down(a: Register) -> Register {
    [a[1], 0]
}

/// The carry-less product of word `i` of `a` and word `j` of `b`, each 0 or 1.
#[inline(always)]
pub(crate) const fn clmul(a: Register, i: usize, b: Register, j: usize) -> Register {
    let x = if i == 0 { a[0] } else { a[1] };
    let y = if j == 0 { b[0] } else { b[1] };
    to_limbs(clmul64(x, y))
}

/// Every byte of `nibbles`, each below 16, replaced by its image under the linear map whose image
/// of bit `k` fills `images[k]`, word by word.
#[cfg(feature = "biguint")]
#[inline(always)]
pub(crate) fn map_nibbles(_: Register, images: [u64; 4], nibbles: Register) -> Register {
    nibbles.map(|word| crate::nibble::map_word(images, word))
}

/// The carry-less product of word `i` of `a`, below `2^32`, and word `j` of `b`.
#[cfg(not(target_arch = "wasm32"))]
#[inline(always)]
pub(crate) const fn clmul_narrow(a: Register, i: usize, b: Register, j: usize) -> Register {
    clmul(a, i, b, j)
}

/// The carry-less product of word `i` of `a`, below `2^32`, and word `j` of `b`: the products of
/// the word of `a` and the two 32-bit halves of the word of `b`.
#[cfg(target_arch = "wasm32")]
#[inline(always)]
pub(crate) const fn clmul_narrow(a: Register, i: usize, b: Register, j: usize) -> Register {
    let x = if i == 0 { a[0] } else { a[1] };
    let y = if j == 0 { b[0] } else { b[1] };
    to_limbs(crate::comb::clmul64_narrow(x, y))
}

/// The squares of the two words of `a`, `(a0^2, a1^2)`: bit `i` of a word moves to bit `2 i`.
#[inline(always)]
pub(crate) const fn spread(a: Register) -> (Register, Register) {
    (spread_word(a[0]), spread_word(a[1]))
}

/// The square of `word`, its bits at twice their positions, as two words.
#[inline(always)]
const fn spread_word(word: u64) -> Register {
    [spread_half(word as u32), spread_half((word >> 32) as u32)]
}

/// Bit `i` of `half` at bit `2 i`, from the products of its bit lanes.
#[cfg(target_arch = "wasm32")]
#[inline(always)]
const fn spread_half(half: u32) -> u64 {
    crate::comb::square32(half)
}

/// Bit `i` of `half` at bit `2 i`: each step moves the upper half of every block up by half the
/// block (Warren, Hacker's Delight, 7-2).
#[cfg(not(target_arch = "wasm32"))]
#[inline(always)]
const fn spread_half(half: u32) -> u64 {
    let mut x = half as u64;
    x = (x | (x << 16)) & 0x0000_FFFF_0000_FFFF;
    x = (x | (x << 8)) & 0x00FF_00FF_00FF_00FF;
    x = (x | (x << 4)) & 0x0F0F_0F0F_0F0F_0F0F;
    x = (x | (x << 2)) & 0x3333_3333_3333_3333;
    (x | (x << 1)) & 0x5555_5555_5555_5555
}

/// The unreduced sum `low + middle * x^64 + high * x^128`.
pub(crate) struct Accumulator {
    low: u128,
    middle: u128,
    high: u128,
}

impl Accumulator {
    #[inline(always)]
    pub(crate) fn from_seed(seed: u128) -> Self {
        Self {
            low: seed,
            middle: 0,
            high: 0,
        }
    }

    #[inline(always)]
    pub(crate) fn accumulate(&mut self, lhs: u128, rhs: u128) {
        let ([lhs_low, lhs_high], [rhs_low, rhs_high]) = (to_limbs(lhs), to_limbs(rhs));
        self.low ^= clmul64(lhs_low, rhs_low);
        self.middle ^= clmul64(lhs_low, rhs_high) ^ clmul64(lhs_high, rhs_low);
        self.high ^= clmul64(lhs_high, rhs_high);
    }

    #[inline(always)]
    pub(crate) fn finalize(self) -> u128 {
        reduce(self.low, reduce(self.middle, self.high))
    }
}

/// `low + high * x^64`, reduced: the part of `high` at and above `x^64` folds into its product with
/// `x^7 + x^2 + x + 1`, [`REDUCTION`], by one shift per term.
#[inline(always)]
fn reduce(low: u128, high: u128) -> u128 {
    let top = high >> 64;
    let mut folded = low ^ (high << 64);
    let mut term = 0;
    while term < u64::BITS - REDUCTION.leading_zeros() {
        if (REDUCTION >> term) & 1 == 1 {
            folded ^= top << term;
        }
        term += 1;
    }
    folded
}
