//! The `x86_64` backend: SSE2 registers, which every `x86_64` CPU has. With `pclmulqdq`,
//! PCLMULQDQ multiplies words; without it, the comb multiplies the words of the registers. With
//! `ssse3`, a byte shuffle spreads the bits of a square.
//!
//! Rust requires `unsafe` for each call of an intrinsic from a function without
//! `#[target_feature]`, even when the build enables the feature; those calls and the conversions
//! between a register and its two words are the module's only `unsafe` operations.
#![expect(
    unsafe_code,
    reason = "the safe API calls the `#[target_feature]` intrinsics of the build's features"
)]

#[cfg(target_feature = "pclmulqdq")]
use core::arch::x86_64::_mm_clmulepi64_si128;
use core::arch::x86_64::{
    __m128i, _mm_and_si128, _mm_or_si128, _mm_sll_epi64, _mm_slli_si128, _mm_srl_epi64,
    _mm_srli_si128, _mm_xor_si128,
};
#[cfg(target_feature = "ssse3")]
use core::arch::x86_64::{_mm_shuffle_epi8, _mm_srli_epi16, _mm_unpackhi_epi8, _mm_unpacklo_epi8};
#[cfg(not(any(target_feature = "ssse3", target_feature = "pclmulqdq")))]
use core::arch::x86_64::{_mm_unpackhi_epi32, _mm_unpacklo_epi32};

#[cfg(not(target_feature = "pclmulqdq"))]
use crate::comb::const_clmul64;
use crate::{REDUCTION, from_limbs, to_limbs};

/// A vector register: two 64-bit words, word 0 in the low half.
pub(crate) type Register = __m128i;

/// The register of zeros.
pub(crate) const ZERO: Register = register([0, 0]);

/// The register of the words `[low, high]`.
#[inline(always)]
pub(crate) const fn register(words: [u64; 2]) -> Register {
    // SAFETY: both types are 16 bytes, every bit pattern is a value of each, and on little-endian
    // `x86_64` the first word is the low half of the register.
    unsafe { core::mem::transmute::<[u64; 2], Register>(words) }
}

/// The words `[low, high]` of `register`.
#[inline(always)]
pub(crate) const fn words(register: Register) -> [u64; 2] {
    // SAFETY: as in `register`.
    unsafe { core::mem::transmute::<Register, [u64; 2]>(register) }
}

/// `a ^ b`.
#[inline(always)]
pub(crate) fn xor(a: Register, b: Register) -> Register {
    // SAFETY: SSE2, which `x86_64` requires.
    unsafe { _mm_xor_si128(a, b) }
}

/// `a & b`.
#[inline(always)]
pub(crate) fn and(a: Register, b: Register) -> Register {
    // SAFETY: SSE2.
    unsafe { _mm_and_si128(a, b) }
}

/// `a | b`.
#[inline(always)]
pub(crate) fn or(a: Register, b: Register) -> Register {
    // SAFETY: SSE2.
    unsafe { _mm_or_si128(a, b) }
}

/// Each word shifted left by `bits`, below 64.
#[inline(always)]
pub(crate) fn shl_words(a: Register, bits: u32) -> Register {
    // SAFETY: SSE2. A constant count folds into the immediate form.
    unsafe { _mm_sll_epi64(a, register([u64::from(bits), 0])) }
}

/// Each word shifted right by `bits`, below 64.
#[inline(always)]
pub(crate) fn shr_words(a: Register, bits: u32) -> Register {
    // SAFETY: SSE2.
    unsafe { _mm_srl_epi64(a, register([u64::from(bits), 0])) }
}

/// `[0, low]`: word 0 moved up to word 1.
#[inline(always)]
pub(crate) fn up(a: Register) -> Register {
    // SAFETY: SSE2.
    unsafe { _mm_slli_si128::<8>(a) }
}

/// `[high, 0]`: word 1 moved down to word 0.
#[inline(always)]
pub(crate) fn down(a: Register) -> Register {
    // SAFETY: SSE2.
    unsafe { _mm_srli_si128::<8>(a) }
}

/// The carry-less product of word `i` of `a` and word `j` of `b`, each 0 or 1, by PCLMULQDQ.
#[cfg(target_feature = "pclmulqdq")]
#[inline(always)]
pub(crate) fn clmul(a: Register, i: usize, b: Register, j: usize) -> Register {
    match (i, j) {
        // SAFETY: the build enables `pclmulqdq`, the `cfg` of this function.
        (0, 0) => unsafe { _mm_clmulepi64_si128::<0x00>(a, b) },
        // SAFETY: as above.
        (0, _) => unsafe { _mm_clmulepi64_si128::<0x10>(a, b) },
        // SAFETY: as above.
        (_, 0) => unsafe { _mm_clmulepi64_si128::<0x01>(a, b) },
        // SAFETY: as above.
        _ => unsafe { _mm_clmulepi64_si128::<0x11>(a, b) },
    }
}

/// The carry-less product of word `i` of `a` and word `j` of `b`, each 0 or 1, by the comb on the
/// two words.
#[cfg(not(target_feature = "pclmulqdq"))]
#[inline(always)]
pub(crate) fn clmul(a: Register, i: usize, b: Register, j: usize) -> Register {
    let ([a_low, a_high], [b_low, b_high]) = (words(a), words(b));
    let x = if i == 0 { a_low } else { a_high };
    let y = if j == 0 { b_low } else { b_high };
    register(to_limbs(const_clmul64(x, y)))
}

/// The squares of the two words of `a`, `(a0^2, a1^2)`: bit `i` of a word moves to bit `2 i`.
/// Each byte's square is the spread of its two nibbles, which a byte shuffle reads from a
/// 16-entry table, interleaved.
#[cfg(target_feature = "ssse3")]
#[inline(always)]
pub(crate) fn spread(a: Register) -> (Register, Register) {
    /// The spread of every nibble value, byte `n` holding `n` with bit `i` at bit `2 i`.
    const NIBBLES: Register = register([0x1514_1110_0504_0100, 0x5554_5150_4544_4140]);
    /// The low nibble of every byte.
    const LOW: Register = register([0x0F0F_0F0F_0F0F_0F0F; 2]);
    // SAFETY: SSE2.
    let shifted = unsafe { _mm_srli_epi16::<4>(a) };
    // SAFETY: SSSE3, which the `cfg` of this function requires.
    let low = unsafe { _mm_shuffle_epi8(NIBBLES, and(a, LOW)) };
    // SAFETY: as above.
    let high = unsafe { _mm_shuffle_epi8(NIBBLES, and(shifted, LOW)) };
    // SAFETY: SSE2.
    let first = unsafe { _mm_unpacklo_epi8(low, high) };
    // SAFETY: SSE2.
    let second = unsafe { _mm_unpackhi_epi8(low, high) };
    (first, second)
}

/// The squares of the two words of `a`, `(a0^2, a1^2)`, by the carry-less product of each word with
/// itself.
#[cfg(all(not(target_feature = "ssse3"), target_feature = "pclmulqdq"))]
#[inline(always)]
pub(crate) fn spread(a: Register) -> (Register, Register) {
    (clmul(a, 0, a, 0), clmul(a, 1, a, 1))
}

/// The squares of the two words of `a`, `(a0^2, a1^2)`: the halves of each word, zero-extended
/// to the two words of a register, spread together.
#[cfg(not(any(target_feature = "ssse3", target_feature = "pclmulqdq")))]
#[inline(always)]
pub(crate) fn spread(a: Register) -> (Register, Register) {
    // SAFETY: SSE2.
    let first = unsafe { _mm_unpacklo_epi32(a, ZERO) };
    // SAFETY: SSE2.
    let second = unsafe { _mm_unpackhi_epi32(a, ZERO) };
    (spread_halves(first), spread_halves(second))
}

/// Bit `i` of each word, for `i` below 32, at bit `2 i`: each step moves the upper half of every
/// block up by half the block (Warren, Hacker's Delight, 7-2).
#[cfg(not(any(target_feature = "ssse3", target_feature = "pclmulqdq")))]
#[inline(always)]
fn spread_halves(a: Register) -> Register {
    let mut x = a;
    x = and(
        or(x, shl_words(x, 16)),
        register([0x0000_FFFF_0000_FFFF; 2]),
    );
    x = and(or(x, shl_words(x, 8)), register([0x00FF_00FF_00FF_00FF; 2]));
    x = and(or(x, shl_words(x, 4)), register([0x0F0F_0F0F_0F0F_0F0F; 2]));
    x = and(or(x, shl_words(x, 2)), register([0x3333_3333_3333_3333; 2]));
    and(or(x, shl_words(x, 1)), register([0x5555_5555_5555_5555; 2]))
}

#[cfg(target_feature = "pclmulqdq")]
#[inline(always)]
pub(crate) fn clmul64(a: u64, b: u64) -> u128 {
    from_limbs(words(clmul(register([a, 0]), 0, register([b, 0]), 0)))
}

#[cfg(not(target_feature = "pclmulqdq"))]
#[inline(always)]
pub(crate) fn clmul64(a: u64, b: u64) -> u128 {
    const_clmul64(a, b)
}

/// The unreduced sum `low + middle * x^64 + high * x^128`.
pub(crate) struct Accumulator {
    low: Register,
    middle: Register,
    high: Register,
}

impl Accumulator {
    #[inline(always)]
    pub(crate) fn from_seed(seed: u128) -> Self {
        Self {
            low: register(to_limbs(seed)),
            middle: ZERO,
            high: ZERO,
        }
    }

    #[inline(always)]
    pub(crate) fn accumulate(&mut self, lhs: u128, rhs: u128) {
        let (x, y) = (register(to_limbs(lhs)), register(to_limbs(rhs)));
        let cross = xor(clmul(x, 1, y, 0), clmul(x, 0, y, 1));
        self.low = xor(self.low, clmul(x, 0, y, 0));
        self.middle = xor(self.middle, cross);
        self.high = xor(self.high, clmul(x, 1, y, 1));
    }

    #[inline(always)]
    pub(crate) fn finalize(self) -> u128 {
        from_limbs(words(reduce(self.low, reduce(self.middle, self.high))))
    }
}

/// `low + high * x^64`, reduced: the high half of `high` folds into its product with
/// `x^7 + x^2 + x + 1`.
#[inline(always)]
fn reduce(low: Register, high: Register) -> Register {
    let folded = clmul(high, 1, register([REDUCTION, 0]), 0);
    xor(xor(low, up(high)), folded)
}
