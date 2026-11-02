//! Shifts by a run-time amount, chosen by width at compile time: native `u128` shifts up to two
//! limbs; above, the bit shift of the limbs written at the limb offset.

use core::ops::{Shl, Shr};

use generic_array::sequence::GenericSequence;
use generic_array::{ArrayLength, GenericArray};

use crate::bits::Bits;
use crate::kernel::{shl_bits_into, shr_bits_into, split, split_mut};
use crate::uint::Uint;

mod sealed {
    /// Seals [`super::ShiftAmount`].
    pub trait Sealed {}
}

/// A shift amount: every primitive integer and every `Uint`.
///
/// One `Shl` and one `Shr` implementation take any amount, so their output is known before the
/// amount's type: an unsuffixed literal amount takes `i32`, as for the primitive integers.
pub trait ShiftAmount: sealed::Sealed + Copy {
    /// The amount, or `None` when it is negative or above `usize::MAX`.
    fn to_usize(self) -> Option<usize>;
}

macro_rules! primitive_amounts {
    ($($primitive:ty),* $(,)?) => {
        $(
            impl sealed::Sealed for $primitive {}

            impl ShiftAmount for $primitive {
                #[inline(always)]
                fn to_usize(self) -> Option<usize> {
                    usize::try_from(self).ok()
                }
            }
        )*
    };
}

primitive_amounts!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);

impl<N: Bits> sealed::Sealed for Uint<N> {}

impl<N: Bits> ShiftAmount for Uint<N> {
    #[inline(always)]
    fn to_usize(self) -> Option<usize> {
        let mut limbs = self.limbs.into_iter();
        let low = limbs.next().unwrap_or(0);
        if limbs.all(|limb| limb == 0) {
            usize::try_from(low).ok()
        } else {
            None
        }
    }
}

/// `(self * 2^rhs) mod 2^N`: `0` when `rhs >= N` or `rhs < 0`.
impl<N: Bits, A: ShiftAmount> Shl<A> for Uint<N> {
    type Output = Self;

    #[inline(always)]
    fn shl(self, rhs: A) -> Self::Output {
        rhs.to_usize().map_or(Self::ZERO, |rhs| {
            Uint::from_generic_limbs(shift_left(&self.limbs, rhs, N::BITS))
        })
    }
}

/// `floor(self / 2^rhs)`: `0` when `rhs >= N` or `rhs < 0`.
impl<N: Bits, A: ShiftAmount> Shr<A> for Uint<N> {
    type Output = Self;

    #[inline(always)]
    fn shr(self, rhs: A) -> Self::Output {
        rhs.to_usize().map_or(Self::ZERO, |rhs| {
            Uint::from_generic_limbs(shift_right(&self.limbs, rhs, N::BITS))
        })
    }
}

impl<N: Bits> Uint<N> {
    /// `(self * 2^rhs) mod 2^N`, or `None` when `rhs >= N`.
    #[inline(always)]
    #[must_use]
    pub fn checked_shl(self, rhs: u32) -> Option<Self> {
        let rhs = rhs as usize;
        (rhs < N::BITS).then(|| self << rhs)
    }

    /// `floor(self / 2^rhs)`, or `None` when `rhs >= N`.
    #[inline(always)]
    #[must_use]
    pub fn checked_shr(self, rhs: u32) -> Option<Self> {
        let rhs = rhs as usize;
        (rhs < N::BITS).then(|| self >> rhs)
    }

    /// `(self * 2^(rhs mod N)) mod 2^N`.
    #[inline(always)]
    #[must_use]
    pub fn wrapping_shl(self, rhs: u32) -> Self {
        self << (rhs as usize % N::BITS)
    }

    /// `floor(self / 2^(rhs mod N))`.
    #[inline(always)]
    #[must_use]
    pub fn wrapping_shr(self, rhs: u32) -> Self {
        self >> (rhs as usize % N::BITS)
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
