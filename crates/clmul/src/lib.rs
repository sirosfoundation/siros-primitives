#![doc = include_str!("../README.md")]
#![no_std]
#![doc(test(attr(forbid(unsafe_code))))]
#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable,
        clippy::todo,
        clippy::unimplemented,
        clippy::indexing_slicing
    )
)]

mod comb;
mod limbs;
#[cfg(feature = "biguint")]
mod poly;
#[cfg(feature = "biguint")]
mod uint;

// The backend of the build's target: its registers, `clmul64` and the GF(2^128) `Accumulator`.
#[cfg(target_arch = "x86_64")]
#[path = "x86_64.rs"]
mod backend;

#[cfg(all(
    target_arch = "aarch64",
    target_endian = "little",
    target_feature = "aes"
))]
#[path = "pmull.rs"]
mod backend;

#[cfg(not(any(
    target_arch = "x86_64",
    all(
        target_arch = "aarch64",
        target_endian = "little",
        target_feature = "aes"
    )
)))]
#[path = "portable.rs"]
mod backend;

/// Whether the build's target features give [`clmul64`] a hardware instruction, PCLMULQDQ or PMULL.
/// Code that chooses between carry-less products and shift-and-add on the cost of a word product
/// reads it.
pub const HARDWARE: bool = cfg!(any(
    all(target_arch = "x86_64", target_feature = "pclmulqdq"),
    all(
        target_arch = "aarch64",
        target_endian = "little",
        target_feature = "aes"
    )
));

pub use comb::const_clmul64;
pub use limbs::{clmul_into, const_clmul_into};
#[cfg(feature = "biguint")]
pub use poly::{Lane, Poly, Width};
#[cfg(feature = "biguint")]
pub use uint::{CarrylessMul, const_carryless_mul, const_widening_carryless_mul};

/// The carry-less product `a * b`: bit `i` of a word is the coefficient of `x^i` of a polynomial
/// over GF(2), and the result is the product polynomial, of degree at most 126.
///
/// The build's target features select PCLMULQDQ, PMULL or the comb of [`const_clmul64`].
#[inline(always)]
#[must_use]
pub fn clmul64(a: u64, b: u64) -> u128 {
    backend::clmul64(a, b)
}

/// `a * b` in `GF(2^128) = GF(2)[x] / (x^128 + x^7 + x^2 + x + 1)`.
///
/// An element is a pair of little-endian limbs; bit `i` of its 128-bit value is the coefficient of
/// `x^i`. GCM numbers the bits of a block the other way round: its element is
/// `u128::from_be_bytes(block).reverse_bits()`.
#[inline(always)]
#[must_use]
pub fn gf2p128_mul(a: [u64; 2], b: [u64; 2]) -> [u64; 2] {
    let mut sum = Gf2p128Accumulator::zero();
    sum.accumulate(a, b);
    sum.finalize()
}

/// A sum of GF(2^128) products that [`finalize`](Self::finalize) reduces once.
///
/// [`accumulate`](Self::accumulate) adds a product without reducing it, so a sum of `k` products
/// costs one reduction. The elements are those of [`gf2p128_mul`].
#[must_use = "an unreduced GF(2^128) product sum must be finalized"]
pub struct Gf2p128Accumulator {
    sum: backend::Accumulator,
}

impl Gf2p128Accumulator {
    /// The empty sum.
    #[inline(always)]
    pub fn zero() -> Self {
        Self::from_seed([0, 0])
    }

    /// The sum that holds the element `seed`.
    #[inline(always)]
    pub fn from_seed(seed: [u64; 2]) -> Self {
        Self {
            sum: backend::Accumulator::from_seed(from_limbs(seed)),
        }
    }

    /// Adds the product `lhs * rhs`.
    #[inline(always)]
    pub fn accumulate(&mut self, lhs: [u64; 2], rhs: [u64; 2]) {
        self.sum.accumulate(from_limbs(lhs), from_limbs(rhs));
    }

    /// The sum, reduced to an element.
    #[inline(always)]
    #[must_use]
    pub fn finalize(self) -> [u64; 2] {
        to_limbs(self.sum.finalize())
    }
}

/// `x^128 mod (x^128 + x^7 + x^2 + x + 1)`: the reduction folds `t * x^128` into `t * REDUCTION`.
const REDUCTION: u64 = 0x87;

/// The little-endian limbs of `value`.
#[inline(always)]
const fn to_limbs(value: u128) -> [u64; 2] {
    [value as u64, (value >> 64) as u64]
}

/// The value of the little-endian `limbs`.
#[inline(always)]
const fn from_limbs(limbs: [u64; 2]) -> u128 {
    let [low, high] = limbs;
    (low as u128) | ((high as u128) << 64)
}
