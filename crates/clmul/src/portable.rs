//! The backend of builds without PCLMULQDQ or PMULL: a register is two `u64` words, and the word
//! products come from the comb.

pub(crate) use crate::comb::const_clmul64 as clmul64;
use crate::{REDUCTION, to_limbs};

/// A register: two 64-bit words, word 0 first.
pub(crate) type Register = [u64; 2];

/// The register of zeros.
pub(crate) const ZERO: Register = [0, 0];

/// The register of the words `[low, high]`.
#[inline(always)]
pub(crate) const fn register(words: [u64; 2]) -> Register {
    words
}

/// The words `[low, high]` of `register`.
#[inline(always)]
pub(crate) const fn words(register: Register) -> [u64; 2] {
    register
}

/// `a ^ b`.
#[inline(always)]
pub(crate) const fn xor(a: Register, b: Register) -> Register {
    [a[0] ^ b[0], a[1] ^ b[1]]
}

/// `a & b`.
#[inline(always)]
pub(crate) const fn and(a: Register, b: Register) -> Register {
    [a[0] & b[0], a[1] & b[1]]
}

/// `a | b`.
#[inline(always)]
pub(crate) const fn or(a: Register, b: Register) -> Register {
    [a[0] | b[0], a[1] | b[1]]
}

/// Each word shifted left by `bits`, below 64.
#[inline(always)]
pub(crate) const fn shl_words(a: Register, bits: u32) -> Register {
    [a[0] << bits, a[1] << bits]
}

/// Each word shifted right by `bits`, below 64.
#[inline(always)]
pub(crate) const fn shr_words(a: Register, bits: u32) -> Register {
    [a[0] >> bits, a[1] >> bits]
}

/// `[0, low]`: word 0 moved up to word 1.
#[inline(always)]
pub(crate) const fn up(a: Register) -> Register {
    [0, a[0]]
}

/// `[high, 0]`: word 1 moved down to word 0.
#[inline(always)]
pub(crate) const fn down(a: Register) -> Register {
    [a[1], 0]
}

/// The carry-less product of word `i` of `a` and word `j` of `b`, each 0 or 1.
#[inline(always)]
pub(crate) const fn clmul(a: Register, i: usize, b: Register, j: usize) -> Register {
    let x = if i == 0 { a[0] } else { a[1] };
    let y = if j == 0 { b[0] } else { b[1] };
    to_limbs(clmul64(x, y))
}

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
