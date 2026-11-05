//! The portable carry-less product: integer products of interleaved bit lanes.
//!
//! Lane `k` of a word keeps its bits at the positions congruent to `k` modulo 4. The integer
//! product of a lane of `a` and a lane of `b` has terms only at the positions congruent to the sum
//! of the lane indices. With `a` cut to its low 60 bits, each of its lanes holds 15 bits, so at
//! most 15 terms meet at a position: the count fits in the 4 bits below the next such position,
//! and the product's bit there is the parity of its terms, the carry-less product's bit. The four
//! products of each output lane sum before one mask keeps that lane. The top 4 bits of `a` meet a
//! lane of `b` once per position at most, so their integer products by the lanes of `b` are
//! carry-less products already.

/// The bits at the positions congruent to 0 modulo 4.
const LANE: u64 = 0x1111_1111_1111_1111;

/// `LANE` in both words.
#[cfg(any(not(target_arch = "wasm32"), test))]
const WIDE_LANE: u128 = (LANE as u128) | ((LANE as u128) << 64);

/// The carry-less product `a * b` of [`clmul64`](crate::clmul64), evaluable in const contexts.
///
/// Builds without PCLMULQDQ or PMULL run it as [`clmul64`](crate::clmul64).
#[inline]
#[must_use]
pub const fn const_clmul64(a: u64, b: u64) -> u128 {
    clmul64(a, b)
}

/// [`const_clmul64`], for the backends' word products. WebAssembly takes [`karatsuba32`]: its
/// 64-bit products are `i64.mul`, which keeps the low half, and a widening product is a call.
#[cfg(not(target_arch = "wasm32"))]
#[inline(always)]
pub(crate) const fn clmul64(a: u64, b: u64) -> u128 {
    comb64(a, b)
}

/// [`const_clmul64`], for the backends' word products.
#[cfg(target_arch = "wasm32")]
#[inline(always)]
pub(crate) const fn clmul64(a: u64, b: u64) -> u128 {
    karatsuba32(a, b)
}

/// The carry-less product of `a`, below `2^32`, and `b`: the products of `a` and the two 32-bit
/// halves of `b`.
#[cfg(any(target_arch = "wasm32", test))]
#[inline(always)]
pub(crate) const fn clmul64_narrow(a: u64, b: u64) -> u128 {
    let a = a as u32;
    let low = clmul32(a, b as u32) as u128;
    low ^ ((clmul32(a, (b >> 32) as u32) as u128) << 32)
}

/// The carry-less product as Karatsuba's three products of 32-bit halves, each from [`clmul32`].
#[cfg(any(target_arch = "wasm32", test))]
#[inline(always)]
const fn karatsuba32(a: u64, b: u64) -> u128 {
    let (a0, a1) = (a as u32, (a >> 32) as u32);
    let (b0, b1) = (b as u32, (b >> 32) as u32);
    let low = clmul32(a0, b0);
    let high = clmul32(a1, b1);
    let middle = clmul32(a0 ^ a1, b0 ^ b1) ^ low ^ high;
    ((low as u128) | ((high as u128) << 64)) ^ ((middle as u128) << 32)
}

/// The carry-less product of two 32-bit words by the lanes of [`comb64`]: a lane of a 32-bit word
/// holds 8 bits, so at most 8 terms meet at a position, and each integer product fits 64 bits.
#[cfg(any(target_arch = "wasm32", test))]
#[inline(always)]
const fn clmul32(a: u32, b: u32) -> u64 {
    let (a, b) = (a as u64, b as u64);
    let lane = LANE & 0xFFFF_FFFF;
    let [x0, x1, x2, x3] = [a & lane, a & (lane << 1), a & (lane << 2), a & (lane << 3)];
    let [y0, y1, y2, y3] = [b & lane, b & (lane << 1), b & (lane << 2), b & (lane << 3)];
    let lane0 =
        x0.wrapping_mul(y0) ^ x1.wrapping_mul(y3) ^ x2.wrapping_mul(y2) ^ x3.wrapping_mul(y1);
    let lane1 =
        x0.wrapping_mul(y1) ^ x1.wrapping_mul(y0) ^ x2.wrapping_mul(y3) ^ x3.wrapping_mul(y2);
    let lane2 =
        x0.wrapping_mul(y2) ^ x1.wrapping_mul(y1) ^ x2.wrapping_mul(y0) ^ x3.wrapping_mul(y3);
    let lane3 =
        x0.wrapping_mul(y3) ^ x1.wrapping_mul(y2) ^ x2.wrapping_mul(y1) ^ x3.wrapping_mul(y0);
    (lane0 & LANE) | (lane1 & (LANE << 1)) | (lane2 & (LANE << 2)) | (lane3 & (LANE << 3))
}

/// The carry-less square of a 32-bit word, bit `i` at bit `2 i`, from the integer squares of its
/// lanes: the products of two distinct lanes meet twice in an output lane of [`clmul32`] and
/// cancel, so 4 products remain, and their 3 cycles of latency replace a chain of 5 shifts and
/// masks.
#[cfg(any(target_arch = "wasm32", test))]
#[inline(always)]
pub(crate) const fn square32(a: u32) -> u64 {
    let a = a as u64;
    let lane = LANE & 0xFFFF_FFFF;
    let [x0, x1, x2, x3] = [a & lane, a & (lane << 1), a & (lane << 2), a & (lane << 3)];
    let even = x0.wrapping_mul(x0) ^ x2.wrapping_mul(x2);
    let odd = x1.wrapping_mul(x1) ^ x3.wrapping_mul(x3);
    (even & LANE) | (odd & (LANE << 2))
}

/// The comb of 64-bit words on widening integer products.
#[cfg(any(not(target_arch = "wasm32"), test))]
#[inline(always)]
const fn comb64(a: u64, b: u64) -> u128 {
    let low = a & ((1 << 60) - 1);
    let x = [
        (low & LANE) as u128,
        (low & (LANE << 1)) as u128,
        (low & (LANE << 2)) as u128,
        (low & (LANE << 3)) as u128,
    ];
    let y = [
        (b & LANE) as u128,
        (b & (LANE << 1)) as u128,
        (b & (LANE << 2)) as u128,
        (b & (LANE << 3)) as u128,
    ];
    let [x0, x1, x2, x3] = x;
    let [y0, y1, y2, y3] = y;
    let lane0 = (x0 * y0) ^ (x1 * y3) ^ (x2 * y2) ^ (x3 * y1);
    let lane1 = (x0 * y1) ^ (x1 * y0) ^ (x2 * y3) ^ (x3 * y2);
    let lane2 = (x0 * y2) ^ (x1 * y1) ^ (x2 * y0) ^ (x3 * y3);
    let lane3 = (x0 * y3) ^ (x1 * y2) ^ (x2 * y1) ^ (x3 * y0);
    let product = (lane0 & WIDE_LANE)
        | (lane1 & (WIDE_LANE << 1))
        | (lane2 & (WIDE_LANE << 2))
        | (lane3 & (WIDE_LANE << 3));
    let top = (a >> 60) as u128;
    let high = (top * y0) ^ (top * y1) ^ (top * y2) ^ (top * y3);
    product ^ (high << 60)
}

#[cfg(test)]
mod tests {
    use super::{clmul64_narrow, comb64, karatsuba32, square32};

    /// The WebAssembly products and squares agree with the comb, which the integration tests check
    /// against the bit-by-bit product, on every target.
    #[test]
    fn webassembly_products_match_the_comb() {
        let mut state = 0x9E37_79B9_7F4A_7C15_u64;
        let mut next = || {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            state ^ (state >> 29)
        };
        for _ in 0..4096 {
            let (a, b) = (next(), next());
            for (a, b) in [(a, b), (u64::MAX, b), (a, u64::MAX), (u64::MAX, u64::MAX)] {
                assert_eq!(karatsuba32(a, b), comb64(a, b));
                let narrow = a & 0xFFFF_FFFF;
                assert_eq!(clmul64_narrow(narrow, b), comb64(narrow, b));
                assert_eq!(u128::from(square32(narrow as u32)), comb64(narrow, narrow));
            }
        }
    }
}
