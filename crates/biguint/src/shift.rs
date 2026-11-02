//! Shifts by a run-time amount, chosen by width at compile time: native `u128` shifts up to two
//! limbs; above, the bit shift of the limbs written at the limb offset.

use core::ops::{Shl, Shr};

use generic_array::sequence::GenericSequence;
use generic_array::{ArrayLength, GenericArray};

use crate::bits::Bits;
use crate::kernel::{shl_bits_into, shr_bits_into, split, split_mut};
use crate::uint::Uint;

/// `(self * 2^rhs) mod 2^N`: `0` when `rhs >= N`.
impl<N: Bits> Shl<usize> for Uint<N> {
    type Output = Self;

    #[inline(always)]
    fn shl(self, rhs: usize) -> Self::Output {
        Uint::from_generic_limbs(shift_left(&self.limbs, rhs, N::BITS))
    }
}

/// `floor(self / 2^rhs)`: `0` when `rhs >= N`.
impl<N: Bits> Shr<usize> for Uint<N> {
    type Output = Self;

    #[inline(always)]
    fn shr(self, rhs: usize) -> Self::Output {
        Uint::from_generic_limbs(shift_right(&self.limbs, rhs, N::BITS))
    }
}

/// `limbs * 2^rhs`, zero when `rhs >= bits`; the caller drops the bits at and above `bits`.
#[inline(always)]
pub(crate) fn shift_left<L: ArrayLength>(
    limbs: &GenericArray<u64, L>,
    rhs: usize,
    bits: usize,
) -> GenericArray<u64, L> {
    if L::USIZE <= 2 {
        let value = wide(limbs).unbounded_shl(narrow(rhs));
        GenericArray::generate(|index| (value >> (64 * index)) as u64)
    } else {
        let mut out = GenericArray::<u64, L>::default();
        if rhs < bits {
            let (_, high) = split_mut(&mut out, rhs / 64);
            shl_bits_into(high, limbs, (rhs % 64) as u32);
        }
        out
    }
}

/// `floor(limbs / 2^rhs)`, zero when `rhs >= bits`; the limbs hold a value below `2^bits`.
#[inline(always)]
pub(crate) fn shift_right<L: ArrayLength>(
    limbs: &GenericArray<u64, L>,
    rhs: usize,
    bits: usize,
) -> GenericArray<u64, L> {
    if L::USIZE <= 2 {
        let value = wide(limbs).unbounded_shr(narrow(rhs));
        GenericArray::generate(|index| (value >> (64 * index)) as u64)
    } else {
        let mut out = GenericArray::<u64, L>::default();
        if rhs < bits {
            let (_, source) = split(limbs, rhs / 64);
            shr_bits_into(&mut out, source, (rhs % 64) as u32);
        }
        out
    }
}

/// The value of at most two limbs.
#[inline(always)]
fn wide(limbs: &[u64]) -> u128 {
    limbs
        .iter()
        .rev()
        .fold(0, |value, &limb| (value << 64) | u128::from(limb))
}

/// `rhs` as a `u128` shift amount: amounts past `u32::MAX` shift everything out as well.
#[inline(always)]
fn narrow(rhs: usize) -> u32 {
    u32::try_from(rhs).unwrap_or(u32::MAX)
}

/// `(self * 2^rhs) mod 2^N`: `0` when `rhs >= N`.
impl<N: Bits, RhsN: Bits> Shl<Uint<RhsN>> for Uint<N> {
    type Output = Self;

    #[inline(always)]
    fn shl(self, rhs: Uint<RhsN>) -> Self::Output {
        shift_amount::<N, RhsN>(rhs).map_or(Self::ZERO, |amount| self << amount)
    }
}

/// `floor(self / 2^rhs)`: `0` when `rhs >= N`.
impl<N: Bits, RhsN: Bits> Shr<Uint<RhsN>> for Uint<N> {
    type Output = Self;

    #[inline(always)]
    fn shr(self, rhs: Uint<RhsN>) -> Self::Output {
        shift_amount::<N, RhsN>(rhs).map_or(Self::ZERO, |amount| self >> amount)
    }
}

/// `rhs` as a shift amount when it is below `N`.
#[inline(always)]
fn shift_amount<N: Bits, RhsN: Bits>(rhs: Uint<RhsN>) -> Option<usize> {
    let mut limbs = rhs.limbs.into_iter();
    let low = limbs.next().unwrap_or(0);
    (limbs.all(|limb| limb == 0) && low < N::BITS as u64).then_some(low as usize)
}
