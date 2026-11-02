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
const WIDE_LANE: u128 = (LANE as u128) | ((LANE as u128) << 64);

/// The carry-less product `a * b` of [`clmul64`](crate::clmul64), evaluable in const contexts.
///
/// Builds without PCLMULQDQ or PMULL run it as [`clmul64`](crate::clmul64).
#[inline(always)]
#[must_use]
pub const fn const_clmul64(a: u64, b: u64) -> u128 {
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
