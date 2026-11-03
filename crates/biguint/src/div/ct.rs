//! Division in constant time over the dividend, by a public divisor: schoolbook long division
//! (Knuth, TAOCP vol. 2, 4.3.1, Algorithm D) whose every step runs the same instructions for every
//! dividend.
//!
//! Each quotient limb comes from the 2-by-1 reciprocal step on the window's top two limbs, with
//! its corrections under masks; a top limb equal to the divisor's takes `2^64 - 1` under a mask.
//! That estimate exceeds the quotient limb by at most two (Theorem B), so the scaled divisor is
//! subtracted and added back twice, each time under the mask of the borrow still owed.

use generic_array::{ArrayLength, GenericArray};

use super::QuotientSink;
use super::reciprocal::{NormalizedLimb, Reciprocal};
use crate::bits::Bits;
use crate::kernel::{
    add_masked, fits, mask_equal, mask_of, shl_bits_into, shr_bits_into, split, split_mut,
    sub_mul_row,
};
use crate::nonzero::significant_len;
use crate::uint::Uint;

/// A public divisor prepared for divisions in constant time over the dividend: shifted left until
/// the top bit of its top significant limb is set, with the reciprocal of that limb.
pub(crate) struct MaskedDivisor<L: ArrayLength> {
    /// The divisor shifted left by `shift`.
    normalized: GenericArray<u64, L>,
    /// The limbs up to its top significant one, at least one.
    len: usize,
    /// The leading zeros of its top significant limb.
    shift: u32,
    /// The reciprocal of the normalized top limb.
    top: Reciprocal,
}

impl<L: ArrayLength> MaskedDivisor<L> {
    /// The nonzero divisor `divisor`.
    #[inline(always)]
    pub(crate) fn new(divisor: &GenericArray<u64, L>) -> Self {
        let len = significant_len(divisor);
        let shift = divisor
            .get(len.wrapping_sub(1))
            .map_or(0, |top| top.leading_zeros());
        let mut normalized = GenericArray::<u64, L>::default();
        shl_bits_into(&mut normalized, divisor, shift);
        let top = normalized.get(len.wrapping_sub(1)).copied().unwrap_or(0);
        Self {
            normalized,
            len,
            shift,
            top: Reciprocal::new(NormalizedLimb::top_set(top)),
        }
    }

    /// `dividend mod divisor` in `M` bits; the quotient limbs go to `quotient`. `W` holds the
    /// dividend's limbs and one more.
    #[inline(always)]
    pub(crate) fn remainder<W: ArrayLength, M: Bits>(
        &self,
        dividend: &[u64],
        quotient: &mut (impl QuotientSink + ?Sized),
    ) -> Uint<M> {
        let mut window = GenericArray::<u64, W>::default();
        shl_bits_into(&mut window, dividend, self.shift);
        let (window, _) = split_mut(&mut window, dividend.len() + 1);
        let (divisor, _) = split(&self.normalized, self.len);
        let mut index = window.len().saturating_sub(self.len + 1);
        while let Some((_, part)) = window.split_at_mut_checked(index) {
            let (part, _) = split_mut(part, self.len + 1);
            if part.len() == self.len + 1 {
                quotient.put(index, self.quotient_limb(part, divisor));
            }
            match index.checked_sub(1) {
                Some(below) => index = below,
                None => break,
            }
        }
        let (low, _) = split(window, self.len);
        let mut remainder = GenericArray::<u64, M::Limbs>::default();
        shr_bits_into(&mut remainder, low, self.shift);
        Uint::from_generic_limbs(remainder)
    }

    /// Reduces the `len + 1` limbs of `part`, whose top `len` limbs are below the divisor, below
    /// the divisor, and returns the quotient limb.
    #[inline(always)]
    fn quotient_limb(&self, part: &mut [u64], divisor: &[u64]) -> u64 {
        let (Some(&top), Some(&divisor_top)) = (part.last(), divisor.last()) else {
            return 0;
        };
        let next = part
            .len()
            .checked_sub(2)
            .and_then(|index| part.get(index))
            .copied()
            .unwrap_or(0);
        // A top limb equal to the divisor's estimates `2^64 - 1`; the step then divides a zero
        // top limb, which keeps its precondition.
        let full = mask_equal(top, divisor_top);
        let (estimate, _) = self.top.div_rem_masked(top & !full, next);
        let estimate = estimate | full;
        let Some((top, low)) = part.split_last_mut() else {
            return 0;
        };
        let (difference, borrow) = top.overflowing_sub(sub_mul_row(low, divisor, estimate));
        *top = difference;
        // The part is negative by `2^(64 * (len + 1))` per owed borrow; each add-back of the
        // divisor that carries out of the top limb repays one.
        let first = u64::from(borrow);
        let second = first ^ add_masked(part, divisor, mask_of(first));
        fits(add_masked(part, divisor, mask_of(second)));
        estimate.wrapping_sub(first).wrapping_sub(second)
    }
}
