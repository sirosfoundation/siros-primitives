//! The portable carry-less product: a comb over five interleaved bit lanes.
//!
//! Lane `k` of a word keeps its bits at the positions congruent to `k` modulo 5, at most 13 of
//! them. The integer product of two lanes has terms only at the positions congruent to the sum of
//! the lane indices, at most 13 at each. A count of at most 13 fits in the 5 bits below the next
//! such position, so no carry reaches it: the product's bit at each such position is the parity of
//! its terms, which is the carry-less product's bit.

/// The bits at the positions congruent to 0 modulo 5.
const LANE: u128 = {
    let mut lane = 0;
    let mut bit = 0;
    while bit < 128 {
        lane |= 1 << bit;
        bit += 5;
    }
    lane
};

/// The carry-less product `a * b` of [`clmul64`](crate::clmul64), evaluable in const contexts.
///
/// Builds without PCLMULQDQ or PMULL run it as [`clmul64`](crate::clmul64).
#[inline(always)]
#[must_use]
pub const fn const_clmul64(a: u64, b: u64) -> u128 {
    let mut product = 0;
    let mut i = 0;
    while i < 5 {
        let a_lane = (a & ((LANE as u64) << i)) as u128;
        let mut j = 0;
        while j < 5 {
            let b_lane = (b & ((LANE as u64) << j)) as u128;
            product ^= (a_lane * b_lane) & (LANE << ((i + j) % 5));
            j += 1;
        }
        i += 1;
    }
    product
}
