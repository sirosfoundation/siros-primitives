use core::cmp::Ordering;
use core::hash::{Hash, Hasher};

use crate::bits::Bits;
use crate::scan;
use crate::uint::Uint;

impl<N: Bits> PartialEq for Uint<N> {
    #[inline(always)]
    fn eq(&self, rhs: &Self) -> bool {
        scan::equal(&self.limbs, &rhs.limbs)
    }
}

/// Hashes the limbs, which equal values share.
impl<N: Bits> Hash for Uint<N> {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.limbs.hash(state);
    }
}

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
