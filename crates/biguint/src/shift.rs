//! Shifts by a run-time amount, chosen by width at compile time: native `u128` shifts up to two
//! limbs; up to eight limbs a branch-free barrel shifter, which funnels every limb by `rhs % 64`
//! and then moves whole limbs by 1, 2, 4, ... places as the bits of `rhs / 64` select; above, the
//! limb stream written at its limb offset.

use core::hint::select_unpredictable;
use core::ops::{Shl, Shr};

use generic_array::sequence::GenericSequence;
use generic_array::{ArrayLength, GenericArray};

use crate::bits::Bits;
use crate::kernel::{split, split_mut};
use crate::stream::{
    LimbStream, ShlBits, ShrBits, ZeroExtended, for_each_slot, funnel_left, funnel_right,
};
use crate::uint::Uint;

/// The most limbs that shift through the barrel shifter.
const SELECT_LIMBS: usize = 8;

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
    } else if L::USIZE <= SELECT_LIMBS {
        let bit_shift = (rhs % 64) as u32;
        let limb = |index: usize| limbs.get(index).copied().unwrap_or(0);
        let funneled = GenericArray::<u64, L>::generate(|index| {
            funnel_left(limb(index), index.checked_sub(1).map_or(0, limb), bit_shift)
        });
        clear_from(
            moved(funneled, rhs / 64, |out, step| out.checked_sub(step)),
            rhs >= bits,
        )
    } else {
        let mut out = GenericArray::<u64, L>::default();
        if rhs < bits {
            let (_, high) = split_mut(&mut out, rhs / 64);
            let mut shifted = ShlBits::new(ZeroExtended::new(limbs), (rhs % 64) as u32);
            for_each_slot(high, |slot| *slot = shifted.next_limb());
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
    } else if L::USIZE <= SELECT_LIMBS {
        let bit_shift = (rhs % 64) as u32;
        let limb = |index: usize| limbs.get(index).copied().unwrap_or(0);
        let funneled = GenericArray::<u64, L>::generate(|index| {
            funnel_right(limb(index), limb(index + 1), bit_shift)
        });
        clear_from(
            moved(funneled, rhs / 64, |out, step| {
                out.checked_add(step).filter(|&from| from < L::USIZE)
            }),
            rhs >= bits,
        )
    } else {
        let mut out = GenericArray::<u64, L>::default();
        if rhs < bits {
            let (_, source) = split(limbs, rhs / 64);
            let (body, top) = split_mut(&mut out, source.len().saturating_sub(1));
            // The stream reads one limb ahead; the top limb has none above it.
            let mut shifted = ShrBits::new(ZeroExtended::new(source), (rhs % 64) as u32);
            for_each_slot(body, |slot| *slot = shifted.next_limb());
            if let (Some(slot), Some(&last)) = (top.first_mut(), source.last()) {
                *slot = last >> (rhs % 64);
            }
        }
        out
    }
}

/// The limbs moved by `limb_shift` places in stages of 1, 2, 4, ... limbs, each a conditional
/// move per limb; `from` names the limb that a stage moves into limb `out`, `None` for a zero.
#[inline(always)]
fn moved<L: ArrayLength>(
    limbs: GenericArray<u64, L>,
    limb_shift: usize,
    from: impl Fn(usize, usize) -> Option<usize>,
) -> GenericArray<u64, L> {
    let mut limbs = limbs;
    let mut step = 1;
    while step < L::USIZE {
        let take = limb_shift & step != 0;
        let previous = limbs.clone();
        let source = |index: usize| previous.get(index).copied().unwrap_or(0);
        limbs = GenericArray::generate(|out| {
            select_unpredictable(take, from(out, step).map_or(0, source), source(out))
        });
        step <<= 1;
    }
    limbs
}

/// Zero when `clear`, else `limbs`, without a branch.
#[inline(always)]
fn clear_from<L: ArrayLength>(limbs: GenericArray<u64, L>, clear: bool) -> GenericArray<u64, L> {
    let mut limbs = limbs;
    for limb in &mut limbs {
        *limb = select_unpredictable(clear, 0, *limb);
    }
    limbs
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
