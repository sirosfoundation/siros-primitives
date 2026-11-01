//! Multiplication of any widths. The operand lengths are type-level constants, so the choice
//! between the strategies below folds away at compile time:
//!
//! - products whose stream the compiler unrolls into straight-line code: the stream product;
//! - longer products below the Karatsuba threshold: rows of fused multiply-accumulate steps with
//!   one carry limb;
//! - longer full products: Karatsuba, in blocks of the shorter operand when the lengths differ;
//! - longer products modulo `2^N` of two `N`-bit operands: the full product of the low parts by
//!   Karatsuba and the truncated cross products of the high parts.

pub(crate) mod basecase;
pub(crate) mod karatsuba;
mod truncated;

use core::ops::{Add, Mul};

use generic_array::{ArrayLength, GenericArray};
use typenum::Sum;

use crate::bits::Bits;
use crate::recursion::{Depth, Halvings};
use crate::uint::Uint;
use basecase::{FULL_STREAM_STEPS, TRUNCATED_STREAM_STEPS, stream, stream_steps};
use karatsuba::{THRESHOLD, blocks};
use truncated::truncated_karatsuba;

pub(crate) use basecase::basecase;
use basecase::rows;

/// Multiplication into a width that holds the full product.
pub trait WideningMul<Rhs = Self> {
    /// The product type.
    type Output;

    /// The exact product `self * rhs`.
    #[must_use]
    fn widening_mul(self, rhs: Rhs) -> Self::Output;
}

/// `(self * rhs) mod 2^N`.
impl<N: Bits, RhsN: Bits> Mul<Uint<RhsN>> for Uint<N> {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Uint<RhsN>) -> Self::Output {
        Uint::from_generic_limbs(truncated_product::<N, _>(&self.limbs, &rhs.limbs))
    }
}

/// The product of an `N`-bit and an `M`-bit integer, in `N + M` bits.
impl<N, RhsN> WideningMul<Uint<RhsN>> for Uint<N>
where
    N: Bits + Add<RhsN>,
    RhsN: Bits,
    Sum<N, RhsN>: Bits,
{
    type Output = Uint<Sum<N, RhsN>>;

    #[inline]
    fn widening_mul(self, rhs: Uint<RhsN>) -> Self::Output {
        Uint::from_generic_limbs(full_product::<Sum<N, RhsN>, _, _>(&self.limbs, &rhs.limbs))
    }
}

/// The strategy for multiplying an `A`-limb operand by a `B`-limb operand into `O` limbs, decided
/// at compile time.
struct Shape<A, B, O>(core::marker::PhantomData<(A, B, O)>);

impl<A: ArrayLength, B: ArrayLength, O: ArrayLength> Shape<A, B, O> {
    const STREAM_STEPS: usize = stream_steps(A::USIZE, B::USIZE, O::USIZE);
    /// A full product unrolls as a stream.
    const FULL_STREAM: bool = Self::STREAM_STEPS <= FULL_STREAM_STEPS;
    /// A truncated product unrolls as a stream.
    const TRUNCATED_STREAM: bool = Self::STREAM_STEPS <= TRUNCATED_STREAM_STEPS;
    /// The shorter operand reaches the Karatsuba threshold.
    const KARATSUBA: bool = !Self::FULL_STREAM && min(A::USIZE, B::USIZE) >= THRESHOLD;
    /// Both operands fill the output and reach the truncated Karatsuba threshold.
    const TRUNCATED_KARATSUBA: bool = !Self::TRUNCATED_STREAM
        && A::USIZE == O::USIZE
        && B::USIZE == O::USIZE
        && O::USIZE >= truncated::THRESHOLD;
}

const fn min(a: usize, b: usize) -> usize {
    if a < b { a } else { b }
}

/// `a * b` in the limbs of the width `P`, which holds it.
#[inline(always)]
pub(crate) fn full_product<P: Bits, A: ArrayLength, B: ArrayLength>(
    a: &GenericArray<u64, A>,
    b: &GenericArray<u64, B>,
) -> GenericArray<u64, P::Limbs> {
    let mut product = GenericArray::<u64, P::Limbs>::default();
    if Shape::<A, B, P::Limbs>::FULL_STREAM {
        stream(&mut product, a, b);
    } else if !Shape::<A, B, P::Limbs>::KARATSUBA {
        rows(&mut product, a, b);
    } else {
        let mut scratch = GenericArray::<u64, P::ProductScratch>::default();
        if P::LIMBS >= A::USIZE + B::USIZE {
            karatsuba_product(&mut product, a, b, &mut scratch);
        } else {
            // The top limb of the full product is zero; it needs a place to land.
            let mut wide = GenericArray::<u64, P::Window>::default();
            karatsuba_product(&mut wide, a, b, &mut scratch);
            for (slot, &limb) in product.iter_mut().zip(wide.iter()) {
                *slot = limb;
            }
        }
    }
    product
}

/// `a * b mod 2^N`.
#[inline(always)]
fn truncated_product<N: Bits, B: ArrayLength>(
    a: &GenericArray<u64, N::Limbs>,
    b: &GenericArray<u64, B>,
) -> GenericArray<u64, N::Limbs> {
    let mut product = GenericArray::<u64, N::Limbs>::default();
    if Shape::<N::Limbs, B, N::Limbs>::TRUNCATED_STREAM {
        stream(&mut product, a, b);
    } else if Shape::<N::Limbs, B, N::Limbs>::TRUNCATED_KARATSUBA {
        let mut scratch = GenericArray::<u64, N::TruncatedScratch>::default();
        truncated_karatsuba::<Halvings>(&mut product, a, b, &mut scratch);
    } else {
        rows(&mut product, a, b);
    }
    product
}

/// `a * b` into `product`, both operands at least [`THRESHOLD`] limbs long.
#[inline(always)]
fn karatsuba_product(product: &mut [u64], a: &[u64], b: &[u64], scratch: &mut [u64]) {
    let (long, short) = if a.len() >= b.len() { (a, b) } else { (b, a) };
    if long.len() == short.len() {
        Halvings::product(product, long, short, scratch);
    } else {
        blocks::<Halvings>(product, long, short, scratch);
    }
}
