use core::ops::{Add, Mul};

use generic_array::{ArrayLength, GenericArray};
use typenum::Sum;

use crate::bits::Bits;
use crate::stream::{MulIter, ZeroExtended, add_into};
use crate::uint::Uint;

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
        Uint::from_generic_limbs(mul_into(&self.limbs, &rhs.limbs))
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
        Uint::from_generic_limbs(mul_into(&self.limbs, &rhs.limbs))
    }
}

/// Drops the carry out of the output window. The window is the product width the caller chose:
/// for a wrapping product the carry is the part above it, for a full product it is zero.
#[inline(always)]
fn truncate_carry(_carry: u64) {}

/// The schoolbook product of the little-endian limbs `a` and `b` in `O` limbs, modulo
/// `2^(64 * O)`.
///
/// The longer operand's [`MulIter`] stream is scaled by each limb of the shorter one and added
/// into the output window, which advances one limb per multiplier limb. The accumulation runs
/// over the whole window, so the loop has a constant shape that the compiler unrolls. Products
/// of at most 16 limb pairs skip the data-dependent `mul != 0` branch, whose misprediction costs
/// more than the multiply.
#[inline(always)]
fn mul_into<O: ArrayLength>(a: &[u64], b: &[u64]) -> GenericArray<u64, O> {
    let mut product = GenericArray::<u64, O>::default();
    {
        let mut window = product.as_mut_slice();
        let (a, b) = if b.len() > a.len() { (b, a) } else { (a, b) };
        for &mul in b {
            if a.len() * b.len() <= 16 || mul != 0 {
                truncate_carry(add_into(
                    MulIter::new(ZeroExtended::new(a.iter().copied()), mul),
                    window,
                ));
            }
            window = match window.split_first_mut() {
                Some((_, rest)) => rest,
                None => break,
            };
        }
    }
    product
}
