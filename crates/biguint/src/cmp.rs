use core::cmp::Ordering;

use crate::bits::Bits;
use crate::uint::Uint;

impl<N: Bits> Ord for Uint<N> {
    #[inline(always)]
    fn cmp(&self, rhs: &Self) -> Ordering {
        for (lhs_limb, rhs_limb) in self.limbs.iter().zip(&rhs.limbs).rev() {
            match lhs_limb.cmp(rhs_limb) {
                Ordering::Equal => {}
                Ordering::Less => return Ordering::Less,
                Ordering::Greater => return Ordering::Greater,
            }
        }
        Ordering::Equal
    }
}

impl<N: Bits> PartialOrd for Uint<N> {
    #[inline(always)]
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}
