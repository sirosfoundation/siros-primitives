//! The `x86_64` backend: SSE2 registers, which every `x86_64` CPU has. With `pclmulqdq`,
//! PCLMULQDQ multiplies words; without it, a vector comb of PMULUDQ lane products does, on AVX2
//! registers for whole products of two registers when the build has `avx2`. With `ssse3`, a byte
//! shuffle spreads the bits of a square.
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
#[cfg(all(not(target_feature = "pclmulqdq"), target_feature = "avx2"))]
use core::arch::x86_64::{
    __m256i, _mm_unpacklo_epi64, _mm256_and_si256, _mm256_castsi256_si128, _mm256_cvtepu32_epi64,
    _mm256_extracti128_si256, _mm256_mul_epu32, _mm256_or_si256, _mm256_set1_epi64x,
    _mm256_xor_si256,
};
#[cfg(not(target_feature = "pclmulqdq"))]
use core::arch::x86_64::{_mm_move_epi64, _mm_mul_epu32, _mm_shuffle_epi32};
#[cfg(target_feature = "ssse3")]
use core::arch::x86_64::{_mm_shuffle_epi8, _mm_srli_epi16, _mm_unpackhi_epi8, _mm_unpacklo_epi8};
#[cfg(not(any(target_feature = "ssse3", target_feature = "pclmulqdq")))]
use core::arch::x86_64::{_mm_unpackhi_epi32, _mm_unpacklo_epi32};

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

/// `register`, whose bits at and above `bits` are zero: one vector register whatever the width.
#[inline(always)]
pub(crate) const fn narrow(register: Register, _bits: usize) -> Register {
    register
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

/// Every byte of `nibbles`, each below 16, replaced by its entry in `entries`: PSHUFB.
#[cfg(all(feature = "biguint", target_feature = "ssse3"))]
#[inline(always)]
pub(crate) fn map_nibbles(entries: Register, _: [u64; 4], nibbles: Register) -> Register {
    // SAFETY: SSSE3, which the `cfg` of this function requires.
    unsafe { _mm_shuffle_epi8(entries, nibbles) }
}

/// Every byte of `nibbles`, each below 16, replaced by its image under the linear map whose image
/// of bit `k` fills `images[k]`, word by word.
#[cfg(all(feature = "biguint", not(target_feature = "ssse3")))]
#[inline(always)]
pub(crate) fn map_nibbles(_: Register, images: [u64; 4], nibbles: Register) -> Register {
    register(words(nibbles).map(|word| crate::nibble::map_word(images, word)))
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

/// The carry-less product of word `i` of `a` and word `j` of `b`, each 0 or 1, by the vector comb.
#[cfg(not(target_feature = "pclmulqdq"))]
#[inline(always)]
pub(crate) fn clmul(a: Register, i: usize, b: Register, j: usize) -> Register {
    let x = if i == 0 { a } else { down(a) };
    let y = if j == 0 { b } else { down(b) };
    comb_product(x, y)
}

// The vector comb of builds without PCLMULQDQ. A 32-bit piece splits into four bit lanes, lane `k`
// the bits at the positions congruent to `k` modulo 4, 8 of them. PMULUDQ multiplies two lanes as
// integers: at most 8 terms meet at a position, and their count fits the 4 bits up to the next
// position of the lane, so the bit at a position of output lane `k` of the sum over the lane pairs
// `(i, k - i)` is the carry-less product's. Karatsuba's three pieces give a product of two words
// from three piece products, sixteen lane products each, and one of two registers from nine.

/// The four lanes of a 32-bit piece, in the low dword of each word.
#[cfg(not(target_feature = "pclmulqdq"))]
const PIECE_LANES: [u64; 4] = [0x1111_1111, 0x2222_2222, 0x4444_4444, 0x8888_8888];

/// The four lanes of the 64-bit product of two pieces.
#[cfg(not(target_feature = "pclmulqdq"))]
const PRODUCT_LANES: [u64; 4] = [
    0x1111_1111_1111_1111,
    0x2222_2222_2222_2222,
    0x4444_4444_4444_4444,
    0x8888_8888_8888_8888,
];

/// The integer products of the low dwords of the words of `a` and `b`, word by word.
#[cfg(not(target_feature = "pclmulqdq"))]
#[inline(always)]
fn mul_dwords(a: Register, b: Register) -> Register {
    // SAFETY: SSE2.
    unsafe { _mm_mul_epu32(a, b) }
}

/// The dwords of `a` in the order that `IMM` selects, as PSHUFD.
#[cfg(not(target_feature = "pclmulqdq"))]
#[inline(always)]
fn shuffle_dwords<const IMM: i32>(a: Register) -> Register {
    // SAFETY: SSE2.
    unsafe { _mm_shuffle_epi32::<IMM>(a) }
}

/// `[low, 0]`: word 0 alone.
#[cfg(not(target_feature = "pclmulqdq"))]
#[inline(always)]
fn low_word(a: Register) -> Register {
    // SAFETY: SSE2.
    unsafe { _mm_move_epi64(a) }
}

/// The products of the pieces in the low dwords of the two words of `x` and `y`, word by word:
/// sixteen PMULUDQ.
#[cfg(not(target_feature = "pclmulqdq"))]
#[inline(always)]
fn piece_pairs(x: Register, y: Register) -> Register {
    let lanes = |value: Register| PIECE_LANES.map(|mask| and(value, register([mask; 2])));
    let (xs, ys) = (lanes(x), lanes(y));
    let product = |i: usize, j: usize| match (xs.get(i), ys.get(j)) {
        (Some(&a), Some(&b)) => mul_dwords(a, b),
        _ => ZERO,
    };
    let lane = |k: usize| {
        let sum = xor(
            xor(product(0, k), product(1, (k + 3) % 4)),
            xor(product(2, (k + 2) % 4), product(3, (k + 1) % 4)),
        );
        PRODUCT_LANES
            .get(k)
            .map_or(ZERO, |&mask| and(sum, register([mask; 2])))
    };
    or(or(lane(0), lane(1)), or(lane(2), lane(3)))
}

/// The product of the pieces in dword 0 of `x` and `y`, in word 0: output lane `k` sums the lane
/// pairs `(0, k)` and `(2, k - 2)` in word 0 and `(1, k - 1)` and `(3, k - 3)` in word 1, eight
/// PMULUDQ in all.
#[cfg(not(target_feature = "pclmulqdq"))]
#[inline(always)]
fn piece(x: Register, y: Register) -> Register {
    let lanes =
        |low: usize, high: usize| match (PIECE_LANES.get(low % 4), PIECE_LANES.get(high % 4)) {
            (Some(&low), Some(&high)) => register([low, high]),
            _ => ZERO,
        };
    let (x, y) = (shuffle_dwords::<0x00>(x), shuffle_dwords::<0x00>(y));
    let (even, odd) = (and(x, lanes(0, 1)), and(x, lanes(2, 3)));
    let lane = |k: usize| {
        let sum = xor(
            mul_dwords(even, and(y, lanes(k, k + 3))),
            mul_dwords(odd, and(y, lanes(k + 2, k + 1))),
        );
        PRODUCT_LANES
            .get(k)
            .map_or(ZERO, |&mask| and(sum, register([mask; 2])))
    };
    let both = or(or(lane(0), lane(1)), or(lane(2), lane(3)));
    xor(both, down(both))
}

/// `low + (middle - low - high) x^32 + high x^64` for the products `low` and `high` in words 0 and
/// 1 of `ends` and the product `middle` of the sums of their pieces in word 0 of `middle`.
#[cfg(not(target_feature = "pclmulqdq"))]
#[inline(always)]
fn join_pieces(ends: Register, middle: Register) -> Register {
    let cross = low_word(xor(middle, xor(ends, shuffle_dwords::<0x4E>(ends))));
    // SAFETY: SSE2.
    xor(ends, unsafe { _mm_slli_si128::<4>(cross) })
}

/// The carry-less product of word `i` of `a`, below `2^32`, and word `j` of `b`, each 0 or 1: the
/// piece of `a` by the two pieces of `b`, sixteen PMULUDQ.
#[cfg(not(target_feature = "pclmulqdq"))]
#[inline(always)]
pub(crate) fn clmul_narrow(a: Register, i: usize, b: Register, j: usize) -> Register {
    let x = if i == 0 { a } else { down(a) };
    let y = if j == 0 { b } else { down(b) };
    let pairs = piece_pairs(shuffle_dwords::<0x00>(x), shuffle_dwords::<0x10>(y));
    // SAFETY: SSE2.
    xor(low_word(pairs), unsafe { _mm_slli_si128::<4>(down(pairs)) })
}

/// The carry-less product of word `i` of `a`, below `2^32`, and word `j` of `b`.
#[cfg(target_feature = "pclmulqdq")]
#[inline(always)]
pub(crate) fn clmul_narrow(a: Register, i: usize, b: Register, j: usize) -> Register {
    clmul(a, i, b, j)
}

/// The product of word 0 of `a` and word 0 of `b`: their pieces in the two words of a register,
/// and the sums of the pieces in a third product.
#[cfg(not(target_feature = "pclmulqdq"))]
#[inline(always)]
fn comb_product(a: Register, b: Register) -> Register {
    // Dwords 0 and 1 to the low dwords of the two words.
    let (x, y) = (shuffle_dwords::<0x10>(a), shuffle_dwords::<0x10>(b));
    let sums = |value: Register| xor(value, shr_words(value, 32));
    join_pieces(piece_pairs(x, y), piece(sums(a), sums(b)))
}

/// An AVX2 register of four words.
#[cfg(all(not(target_feature = "pclmulqdq"), target_feature = "avx2"))]
type Wide = __m256i;

/// `a ^ b` on four words.
#[cfg(all(not(target_feature = "pclmulqdq"), target_feature = "avx2"))]
#[inline(always)]
fn xor_wide(a: Wide, b: Wide) -> Wide {
    // SAFETY: AVX2, which the `cfg` of this function requires.
    unsafe { _mm256_xor_si256(a, b) }
}

/// `a | b` on four words.
#[cfg(all(not(target_feature = "pclmulqdq"), target_feature = "avx2"))]
#[inline(always)]
fn or_wide(a: Wide, b: Wide) -> Wide {
    // SAFETY: AVX2.
    unsafe { _mm256_or_si256(a, b) }
}

/// Each word of `a` and `mask`.
#[cfg(all(not(target_feature = "pclmulqdq"), target_feature = "avx2"))]
#[inline(always)]
fn mask_wide(a: Wide, mask: u64) -> Wide {
    // SAFETY: AVX2.
    let splat = unsafe { _mm256_set1_epi64x(mask as i64) };
    // SAFETY: AVX2.
    unsafe { _mm256_and_si256(a, splat) }
}

/// The integer products of the low dwords of the four words of `a` and `b`, word by word.
#[cfg(all(not(target_feature = "pclmulqdq"), target_feature = "avx2"))]
#[inline(always)]
fn mul_dwords_wide(a: Wide, b: Wide) -> Wide {
    // SAFETY: AVX2.
    unsafe { _mm256_mul_epu32(a, b) }
}

/// The products of the pieces in the low dwords of the four words of `x` and `y`, word by word:
/// sixteen VPMULUDQ.
#[cfg(all(not(target_feature = "pclmulqdq"), target_feature = "avx2"))]
#[inline(always)]
fn piece_quads(x: Wide, y: Wide) -> Wide {
    let lanes = |value: Wide| PIECE_LANES.map(|mask| mask_wide(value, mask));
    let (xs, ys) = (lanes(x), lanes(y));
    let product = |i: usize, j: usize| match (xs.get(i), ys.get(j)) {
        (Some(&a), Some(&b)) => mul_dwords_wide(a, b),
        _ => mask_wide(x, 0),
    };
    let lane = |k: usize| {
        let sum = xor_wide(
            xor_wide(product(0, k), product(1, (k + 3) % 4)),
            xor_wide(product(2, (k + 2) % 4), product(3, (k + 1) % 4)),
        );
        mask_wide(sum, PRODUCT_LANES.get(k).copied().unwrap_or(0))
    };
    or_wide(or_wide(lane(0), lane(1)), or_wide(lane(2), lane(3)))
}

/// The product of `a` and `b` as Karatsuba's `[low, middle, high]`, whose sum
/// `low + middle x^64 + high x^128` it is: the nine piece products of Karatsuba on words and on
/// pieces, eight of them in two registers of four words and the ninth alone.
#[cfg(all(not(target_feature = "pclmulqdq"), target_feature = "avx2"))]
#[inline(always)]
pub(crate) fn karatsuba(a: Register, b: Register) -> [Register; 3] {
    // `[a0 + a1, a2 + a3, m0, m1]` for the dwords `a0` to `a3` of `a` and those of the sum `m` of
    // its words, and `m0 + m1` in dword 0.
    let sums = |value: Register| {
        let halves = xor(value, shr_words(value, 32));
        let words = xor(value, down(value));
        // SAFETY: SSE2.
        let pairs = unsafe { _mm_unpacklo_epi64(shuffle_dwords::<0x08>(halves), words) };
        (pairs, xor(words, shr_words(words, 32)))
    };
    let ((a_sums, a_middle), (b_sums, b_middle)) = (sums(a), sums(b));
    // SAFETY: AVX2: each dword to a word of its own.
    let widen = |value: Register| unsafe { _mm256_cvtepu32_epi64(value) };
    let halves = |value: Wide| {
        // SAFETY: AVX2.
        let low = unsafe { _mm256_castsi256_si128(value) };
        // SAFETY: AVX2.
        let high = unsafe { _mm256_extracti128_si256::<1>(value) };
        (low, high)
    };
    let (ends_low, ends_high) = halves(piece_quads(widen(a), widen(b)));
    let (sums_low, sums_high) = halves(piece_quads(widen(a_sums), widen(b_sums)));
    let low = join_pieces(ends_low, sums_low);
    let high = join_pieces(ends_high, down(sums_low));
    let middle = join_pieces(sums_high, piece(a_middle, b_middle));
    [low, xor(middle, xor(low, high)), high]
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

/// The comb on general registers: the moves of the words to a vector register and back cost more
/// than the vector comb saves.
#[cfg(not(target_feature = "pclmulqdq"))]
#[inline(always)]
pub(crate) fn clmul64(a: u64, b: u64) -> u128 {
    crate::comb::clmul64(a, b)
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
