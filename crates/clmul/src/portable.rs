//! The backend of builds without PCLMULQDQ or PMULL: the comb, with the GF(2^128) sum in three
//! `u128` lanes.

pub(crate) use crate::comb::const_clmul64 as clmul64;
use crate::{REDUCTION, to_limbs};

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
/// `x^7 + x^2 + x + 1`.
#[inline(always)]
fn reduce(low: u128, high: u128) -> u128 {
    low ^ (high << 64) ^ clmul64((high >> 64) as u64, REDUCTION)
}
