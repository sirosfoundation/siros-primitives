//! Multiplication of any widths. The operand lengths are type-level constants, so the choice
//! between the strategies below folds away at compile time:
//!
//! - products below the Karatsuba threshold: the quadratic product, one row per limb;
//! - longer full products: Karatsuba, in blocks of the shorter operand when the lengths differ;
//! - longer products modulo `2^N` of two `N`-bit operands: the full product of the low parts and
//!   the truncated cross products of the high parts, which split again.

pub(crate) mod basecase;
pub(crate) mod karatsuba;
pub(crate) mod truncated;

use core::ops::{Add, Mul};

use generic_array::{ArrayLength, GenericArray};
use typenum::Sum;

use crate::bits::Bits;
use crate::recursion::{Depth, Halvings, by_width};
use crate::uint::Uint;
use karatsuba::{THRESHOLD, blocks};

pub(crate) use basecase::basecase;

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

    #[inline(always)]
    fn mul(self, rhs: Uint<RhsN>) -> Self::Output {
        by_width(
            N::LIMBS.min(RhsN::LIMBS),
            #[inline(always)]
            move || {
                Uint::from_generic_limbs(truncated_product::<N, _, false>(&self.limbs, &rhs.limbs))
            },
        )
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

    #[inline(always)]
    fn widening_mul(self, rhs: Uint<RhsN>) -> Self::Output {
        by_width(
            N::LIMBS.min(RhsN::LIMBS),
            #[inline(always)]
            move || {
                Uint::from_generic_limbs(full_product::<Sum<N, RhsN>, _, _, false>(
                    &self.limbs,
                    &rhs.limbs,
                ))
            },
        )
    }
}

impl<N: Bits> Uint<N> {
    /// `(self * rhs) mod 2^N`.
    #[inline(always)]
    #[must_use]
    pub fn wrapping_mul(self, rhs: Self) -> Self {
        self * rhs
    }

    /// `(self * rhs) mod 2^N`, and whether the product reaches `2^N`.
    ///
    /// The bit lengths decide the overflow unless they sum to `N + 1`. Then `self >> 1` times
    /// `rhs` fits in `N` bits, and the product is twice it plus `rhs` when `self` is odd.
    #[inline(always)]
    #[must_use]
    pub fn overflowing_mul(self, rhs: Self) -> (Self, bool) {
        let bits = self.bit_len() + rhs.bit_len();
        let undecided = bits == N::BITS + 1;
        let factor = if undecided { self >> 1usize } else { self };
        let product = factor * rhs;
        if undecided {
            let odd = if self.bit_at(0) { rhs } else { Self::ZERO };
            let (sum, carry) = (product << 1usize).carrying_add(odd, false);
            (sum, carry || product.bit_at(N::BITS - 1))
        } else {
            (product, bits > N::BITS)
        }
    }

    /// `self * rhs`, or `None` when the product reaches `2^N`.
    #[inline(always)]
    #[must_use]
    pub fn checked_mul(self, rhs: Self) -> Option<Self> {
        let (product, overflow) = self.overflowing_mul(rhs);
        (!overflow).then_some(product)
    }

    /// `self * rhs`, or [`Self::MAX`] when the product reaches `2^N`.
    #[inline(always)]
    #[must_use]
    pub fn saturating_mul(self, rhs: Self) -> Self {
        self.checked_mul(rhs).unwrap_or(Self::MAX)
    }

    /// `(self * factor + addend) mod 2^N`.
    #[inline(always)]
    #[must_use]
    pub fn mul_add(self, factor: Self, addend: Self) -> Self {
        self * factor + addend
    }
}

/// The strategy for multiplying an `A`-limb operand by a `B`-limb operand into `O` limbs, decided
/// at compile time.
struct Shape<A, B, O>(core::marker::PhantomData<(A, B, O)>);

impl<A: ArrayLength, B: ArrayLength, O: ArrayLength> Shape<A, B, O> {
    /// The shorter operand reaches the Karatsuba threshold.
    const KARATSUBA: bool = min(A::USIZE, B::USIZE) >= THRESHOLD;
    /// Both operands fill the output, which is long enough for the split to beat the quadratic
    /// product.
    const TRUNCATED_SPLIT: bool =
        A::USIZE == O::USIZE && B::USIZE == O::USIZE && !truncated::quadratic(O::USIZE);
}

const fn min(a: usize, b: usize) -> usize {
    if a < b { a } else { b }
}

/// `a * b` in the limbs of the width `P`, which holds it. `MASKED` runs every correction under
/// masks: no branch and no memory address depends on the operands.
#[inline(always)]
pub(crate) fn full_product<P: Bits, A: ArrayLength, B: ArrayLength, const MASKED: bool>(
    a: &GenericArray<u64, A>,
    b: &GenericArray<u64, B>,
) -> GenericArray<u64, P::Limbs> {
    let mut product = GenericArray::<u64, P::Limbs>::default();
    if !Shape::<A, B, P::Limbs>::KARATSUBA {
        basecase::product(&mut product, a, b);
    } else if A::USIZE == B::USIZE {
        // The difference products need about seven eighths of the product's limbs.
        karatsuba_into::<P, MASKED>(
            &mut product,
            a,
            b,
            &mut GenericArray::<u64, P::BalancedScratch>::default(),
        );
    } else {
        // The block product and its difference products.
        karatsuba_into::<P, MASKED>(
            &mut product,
            a,
            b,
            &mut GenericArray::<u64, P::ProductScratch>::default(),
        );
    }
    product
}

/// `a * b` into `product`, the `P::LIMBS` limbs of the width `P`, by [`karatsuba_product`].
#[inline(always)]
fn karatsuba_into<P: Bits, const MASKED: bool>(
    product: &mut [u64],
    a: &[u64],
    b: &[u64],
    scratch: &mut [u64],
) {
    if P::LIMBS >= a.len() + b.len() {
        karatsuba_product::<MASKED>(product, a, b, scratch);
    } else {
        // The top limb of the full product is zero; it needs a place to land.
        let mut wide = GenericArray::<u64, P::Window>::default();
        karatsuba_product::<MASKED>(&mut wide, a, b, scratch);
        for (slot, &limb) in product.iter_mut().zip(wide.iter()) {
            *slot = limb;
        }
    }
}

/// `a * b mod 2^N`; `MASKED` as for [`full_product`].
#[inline(always)]
pub(crate) fn truncated_product<N: Bits, B: ArrayLength, const MASKED: bool>(
    a: &GenericArray<u64, N::Limbs>,
    b: &GenericArray<u64, B>,
) -> GenericArray<u64, N::Limbs> {
    let mut product = GenericArray::<u64, N::Limbs>::default();
    if Shape::<N::Limbs, B, N::Limbs>::TRUNCATED_SPLIT {
        let mut scratch = GenericArray::<u64, N::TruncatedScratch>::default();
        Halvings::truncated_with::<MASKED>(&mut product, a, b, &mut scratch);
    } else {
        basecase::product(&mut product, a, b);
    }
    product
}

/// `a * b` into `product`, both operands at least [`THRESHOLD`] limbs long.
#[inline(always)]
fn karatsuba_product<const MASKED: bool>(
    product: &mut [u64],
    a: &[u64],
    b: &[u64],
    scratch: &mut [u64],
) {
    let (long, short) = if a.len() >= b.len() { (a, b) } else { (b, a) };
    if long.len() == short.len() {
        Halvings::product_with::<MASKED>(product, long, short, scratch);
    } else {
        blocks::<Halvings, MASKED>(product, long, short, scratch);
    }
}
