//! The little-endian `aarch64` backend: PMULL, which the `aes` feature provides.
//!
//! The module compiles only when the build enables `aes`, so the CPU that runs it has PMULL and
//! Advanced SIMD. Rust still requires `unsafe` for a call into a `#[target_feature]` function from
//! a function without the attribute; the four such calls are the module's only `unsafe`
//! operations. The intrinsics convert between `p128` and vectors by the little-endian memory
//! layout, so big-endian builds use the comb.
#![expect(
    unsafe_code,
    reason = "the safe API calls the `#[target_feature]` functions that hold the intrinsics"
)]

use core::arch::aarch64::{
    poly64x2_t, vaddq_p64, vextq_p64, vgetq_lane_p64, vmull_high_p64, vmull_p64,
    vreinterpretq_p64_p128, vreinterpretq_p128_p64,
};

use crate::REDUCTION;

#[inline(always)]
pub(crate) fn clmul64(a: u64, b: u64) -> u128 {
    // SAFETY: the build enables `aes`, the `cfg` of this module.
    unsafe { vmull_p64(a, b) }
}

/// The unreduced sum `low + middle * x^64 + high * x^128`.
pub(crate) struct Accumulator {
    low: poly64x2_t,
    middle: poly64x2_t,
    high: poly64x2_t,
}

impl Accumulator {
    #[inline(always)]
    pub(crate) fn from_seed(seed: u128) -> Self {
        // SAFETY: the build enables `aes`, the `cfg` of this module.
        unsafe { seeded(seed) }
    }

    #[inline(always)]
    pub(crate) fn accumulate(&mut self, lhs: u128, rhs: u128) {
        // SAFETY: the build enables `aes`, the `cfg` of this module.
        unsafe { add_product(self, lhs, rhs) }
    }

    #[inline(always)]
    pub(crate) fn finalize(self) -> u128 {
        // SAFETY: the build enables `aes`, the `cfg` of this module.
        unsafe { reduced(self) }
    }
}

#[inline]
#[target_feature(enable = "neon,aes")]
fn seeded(seed: u128) -> Accumulator {
    let zero = vreinterpretq_p64_p128(0);
    Accumulator {
        low: vreinterpretq_p64_p128(seed),
        middle: zero,
        high: zero,
    }
}

#[inline]
#[target_feature(enable = "neon,aes")]
fn add_product(sum: &mut Accumulator, lhs: u128, rhs: u128) {
    let (x, y) = (vreinterpretq_p64_p128(lhs), vreinterpretq_p64_p128(rhs));
    let swapped = vextq_p64::<1>(x, x);
    let cross = vaddq_p64(low_product(swapped, y), high_product(swapped, y));
    sum.low = vaddq_p64(sum.low, low_product(x, y));
    sum.middle = vaddq_p64(sum.middle, cross);
    sum.high = vaddq_p64(sum.high, high_product(x, y));
}

#[inline]
#[target_feature(enable = "neon,aes")]
fn reduced(sum: Accumulator) -> u128 {
    vreinterpretq_p128_p64(reduce(sum.low, reduce(sum.middle, sum.high)))
}

/// `low + high * x^64`, reduced: the high half of `high` folds into its product with
/// `x^7 + x^2 + x + 1`.
#[inline]
#[target_feature(enable = "neon,aes")]
fn reduce(low: poly64x2_t, high: poly64x2_t) -> poly64x2_t {
    let shifted = vextq_p64::<1>(vreinterpretq_p64_p128(0), high);
    let folded = high_product(high, vreinterpretq_p64_p128(u128::from(REDUCTION) << 64));
    vaddq_p64(vaddq_p64(low, shifted), folded)
}

/// The product of the low lanes of `x` and `y`.
#[inline]
#[target_feature(enable = "neon,aes")]
fn low_product(x: poly64x2_t, y: poly64x2_t) -> poly64x2_t {
    vreinterpretq_p64_p128(vmull_p64(vgetq_lane_p64::<0>(x), vgetq_lane_p64::<0>(y)))
}

/// The product of the high lanes of `x` and `y`.
#[inline]
#[target_feature(enable = "neon,aes")]
fn high_product(x: poly64x2_t, y: poly64x2_t) -> poly64x2_t {
    vreinterpretq_p64_p128(vmull_high_p64(x, y))
}
