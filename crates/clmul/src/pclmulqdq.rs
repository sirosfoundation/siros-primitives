//! The `x86_64` backend: PCLMULQDQ on SSE2 registers.
//!
//! The module compiles only when the build enables `pclmulqdq`, so the CPU that runs it has the
//! instruction and SSE2, which `x86_64` requires. Rust still requires `unsafe` for each call of an
//! intrinsic from a function without `#[target_feature]`; those calls and the conversions between
//! a register and its two words are the module's only `unsafe` operations.
#![expect(
    unsafe_code,
    reason = "the safe API calls the `#[target_feature]` intrinsics of the build's features"
)]

use core::arch::x86_64::{
    __m128i, _mm_and_si128, _mm_clmulepi64_si128, _mm_or_si128, _mm_sll_epi64, _mm_slli_si128,
    _mm_srl_epi64, _mm_srli_si128, _mm_xor_si128,
};

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

/// The carry-less product of word `i` of `a` and word `j` of `b`, each 0 or 1.
#[inline(always)]
pub(crate) fn clmul(a: Register, i: usize, b: Register, j: usize) -> Register {
    match (i, j) {
        // SAFETY: the build enables `pclmulqdq`, the `cfg` of this module.
        (0, 0) => unsafe { _mm_clmulepi64_si128::<0x00>(a, b) },
        // SAFETY: as above.
        (0, _) => unsafe { _mm_clmulepi64_si128::<0x10>(a, b) },
        // SAFETY: as above.
        (_, 0) => unsafe { _mm_clmulepi64_si128::<0x01>(a, b) },
        // SAFETY: as above.
        _ => unsafe { _mm_clmulepi64_si128::<0x11>(a, b) },
    }
}

#[inline(always)]
pub(crate) fn clmul64(a: u64, b: u64) -> u128 {
    from_limbs(words(clmul(register([a, 0]), 0, register([b, 0]), 0)))
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
