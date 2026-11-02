//! The little-endian `aarch64` backend: PMULL, which the `aes` feature provides, on Advanced SIMD
//! registers.
//!
//! The module compiles only when the build enables `aes`, so the CPU that runs it has PMULL and
//! Advanced SIMD. Rust still requires `unsafe` for each call of an intrinsic from a function
//! without `#[target_feature]`; those calls and the conversions between a register and its two
//! words are the module's only `unsafe` operations. The conversions follow the little-endian
//! layout, so big-endian builds use the comb.
#![expect(
    unsafe_code,
    reason = "the safe API calls the `#[target_feature]` intrinsics of the build's features"
)]

use core::arch::aarch64::{
    int64x2_t, uint64x2_t, vandq_u64, vdupq_n_s64, veorq_u64, vextq_u64, vgetq_lane_u64,
    vmull_high_p64, vmull_p64, vorrq_u64, vreinterpretq_p64_u64, vreinterpretq_u64_p128, vshlq_u64,
};

use crate::{REDUCTION, from_limbs, to_limbs};

/// A vector register: two 64-bit words, word 0 in lane 0.
pub(crate) type Register = uint64x2_t;

/// The register of zeros.
pub(crate) const ZERO: Register = register([0, 0]);

/// The register of the words `[low, high]`.
#[inline(always)]
pub(crate) const fn register(words: [u64; 2]) -> Register {
    // SAFETY: both types are 16 bytes, every bit pattern is a value of each, and on little-endian
    // `aarch64` the first word is lane 0.
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
    // SAFETY: Advanced SIMD, which `aes` implies.
    unsafe { veorq_u64(a, b) }
}

/// `a & b`.
#[inline(always)]
pub(crate) fn and(a: Register, b: Register) -> Register {
    // SAFETY: Advanced SIMD.
    unsafe { vandq_u64(a, b) }
}

/// `a | b`.
#[inline(always)]
pub(crate) fn or(a: Register, b: Register) -> Register {
    // SAFETY: Advanced SIMD.
    unsafe { vorrq_u64(a, b) }
}

/// The shift count `bits` in both lanes: a positive count shifts left, a negative one right.
#[inline(always)]
fn counts(bits: i64) -> int64x2_t {
    // SAFETY: Advanced SIMD, which `aes` implies.
    unsafe { vdupq_n_s64(bits) }
}

/// Each word shifted left by `bits`, below 64.
#[inline(always)]
pub(crate) fn shl_words(a: Register, bits: u32) -> Register {
    // SAFETY: Advanced SIMD. A constant count folds into the immediate form.
    unsafe { vshlq_u64(a, counts(i64::from(bits))) }
}

/// Each word shifted right by `bits`, below 64.
#[inline(always)]
pub(crate) fn shr_words(a: Register, bits: u32) -> Register {
    // SAFETY: Advanced SIMD; a negative count shifts right, filling with zeros.
    unsafe { vshlq_u64(a, counts(-i64::from(bits))) }
}

/// `[0, low]`: word 0 moved up to word 1.
#[inline(always)]
pub(crate) fn up(a: Register) -> Register {
    // SAFETY: Advanced SIMD.
    unsafe { vextq_u64::<1>(ZERO, a) }
}

/// `[high, 0]`: word 1 moved down to word 0.
#[inline(always)]
pub(crate) fn down(a: Register) -> Register {
    // SAFETY: Advanced SIMD.
    unsafe { vextq_u64::<1>(a, ZERO) }
}

/// Word `I` of `a`.
#[inline(always)]
fn word<const I: i32>(a: Register) -> u64 {
    // SAFETY: Advanced SIMD; `I` is 0 or 1.
    unsafe { vgetq_lane_u64::<I>(a) }
}

/// The product of two words in a register.
#[inline(always)]
fn product(a: u64, b: u64) -> Register {
    // SAFETY: the build enables `aes`, the `cfg` of this module, which provides PMULL.
    let product = unsafe { vmull_p64(a, b) };
    // SAFETY: Advanced SIMD.
    unsafe { vreinterpretq_u64_p128(product) }
}

/// The carry-less product of word `i` of `a` and word `j` of `b`, each 0 or 1.
#[inline(always)]
pub(crate) fn clmul(a: Register, i: usize, b: Register, j: usize) -> Register {
    match (i, j) {
        (0, 0) => product(word::<0>(a), word::<0>(b)),
        (0, _) => product(word::<0>(a), word::<1>(b)),
        (_, 0) => product(word::<1>(a), word::<0>(b)),
        _ => {
            // SAFETY: Advanced SIMD.
            let a = unsafe { vreinterpretq_p64_u64(a) };
            // SAFETY: Advanced SIMD.
            let b = unsafe { vreinterpretq_p64_u64(b) };
            // SAFETY: the build enables `aes`.
            let product = unsafe { vmull_high_p64(a, b) };
            // SAFETY: Advanced SIMD.
            unsafe { vreinterpretq_u64_p128(product) }
        }
    }
}

#[inline(always)]
pub(crate) fn clmul64(a: u64, b: u64) -> u128 {
    from_limbs(words(product(a, b)))
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
