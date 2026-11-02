//! The `x86_64` backend: PCLMULQDQ. In builds that also enable `sse4.1`, LLVM moves the high half
//! of a register out with `pextrq`.
//!
//! The module compiles only when the build enables `pclmulqdq`, so the CPU that runs it has the
//! instruction. Rust still requires `unsafe` for a call into a `#[target_feature]` function from a
//! function without the attribute; the four such calls are the module's only `unsafe` operations.
#![expect(
    unsafe_code,
    reason = "the safe API calls the `#[target_feature]` functions that hold the intrinsics"
)]

use core::arch::x86_64::{
    __m128i, _mm_clmulepi64_si128, _mm_cvtsi128_si64, _mm_set_epi64x, _mm_slli_si128,
    _mm_unpackhi_epi64, _mm_xor_si128,
};

use crate::{REDUCTION, from_limbs, to_limbs};

#[inline(always)]
pub(crate) fn clmul64(a: u64, b: u64) -> u128 {
    // SAFETY: the build enables `pclmulqdq`, the `cfg` of this module.
    unsafe { word_product(a, b) }
}

/// The unreduced sum `low + middle * x^64 + high * x^128`.
pub(crate) struct Accumulator {
    low: __m128i,
    middle: __m128i,
    high: __m128i,
}

impl Accumulator {
    #[inline(always)]
    pub(crate) fn from_seed(seed: u128) -> Self {
        // SAFETY: the build enables `pclmulqdq`, the `cfg` of this module.
        unsafe { seeded(seed) }
    }

    #[inline(always)]
    pub(crate) fn accumulate(&mut self, lhs: u128, rhs: u128) {
        // SAFETY: the build enables `pclmulqdq`, the `cfg` of this module.
        unsafe { add_product(self, lhs, rhs) }
    }

    #[inline(always)]
    pub(crate) fn finalize(self) -> u128 {
        // SAFETY: the build enables `pclmulqdq`, the `cfg` of this module.
        unsafe { reduced(self) }
    }
}

#[inline]
#[target_feature(enable = "pclmulqdq")]
fn word_product(a: u64, b: u64) -> u128 {
    unpack(_mm_clmulepi64_si128::<0x00>(
        pack(u128::from(a)),
        pack(u128::from(b)),
    ))
}

#[inline]
#[target_feature(enable = "pclmulqdq")]
fn seeded(seed: u128) -> Accumulator {
    let zero = pack(0);
    Accumulator {
        low: pack(seed),
        middle: zero,
        high: zero,
    }
}

#[inline]
#[target_feature(enable = "pclmulqdq")]
fn add_product(sum: &mut Accumulator, lhs: u128, rhs: u128) {
    let (x, y) = (pack(lhs), pack(rhs));
    let cross = _mm_xor_si128(
        _mm_clmulepi64_si128::<0x01>(x, y),
        _mm_clmulepi64_si128::<0x10>(x, y),
    );
    sum.low = _mm_xor_si128(sum.low, _mm_clmulepi64_si128::<0x00>(x, y));
    sum.middle = _mm_xor_si128(sum.middle, cross);
    sum.high = _mm_xor_si128(sum.high, _mm_clmulepi64_si128::<0x11>(x, y));
}

#[inline]
#[target_feature(enable = "pclmulqdq")]
fn reduced(sum: Accumulator) -> u128 {
    unpack(reduce(sum.low, reduce(sum.middle, sum.high)))
}

/// `low + high * x^64`, reduced: the high half of `high` folds into its product with
/// `x^7 + x^2 + x + 1`.
#[inline]
#[target_feature(enable = "pclmulqdq")]
fn reduce(low: __m128i, high: __m128i) -> __m128i {
    let folded = _mm_clmulepi64_si128::<0x01>(high, pack(u128::from(REDUCTION)));
    _mm_xor_si128(_mm_xor_si128(low, _mm_slli_si128::<8>(high)), folded)
}

/// `value` in a register, its low half in lane 0.
#[inline]
#[target_feature(enable = "pclmulqdq")]
fn pack(value: u128) -> __m128i {
    let [low, high] = to_limbs(value);
    _mm_set_epi64x(high.cast_signed(), low.cast_signed())
}

/// The value of a register whose lane 0 holds the low half.
#[inline]
#[target_feature(enable = "pclmulqdq")]
fn unpack(lanes: __m128i) -> u128 {
    let low = _mm_cvtsi128_si64(lanes);
    let high = _mm_cvtsi128_si64(_mm_unpackhi_epi64(lanes, lanes));
    from_limbs([low.cast_unsigned(), high.cast_unsigned()])
}
