use core::ops::{BitAnd, BitOr, BitXor, Not};

use crate::bits::Bits;
use crate::stream::{AndOp, Op2, OrOp, TwosComplement, XorOp};
use crate::uint::Uint;

/// Limb-wise AND; `rhs` is zero-extended or truncated to `N` bits.
impl<N: Bits, RhsN: Bits> BitAnd<Uint<RhsN>> for Uint<N> {
    type Output = Self;

    #[inline(always)]
    fn bitand(self, rhs: Uint<RhsN>) -> Self::Output {
        Uint::from_source(
            Op2::<_, _, AndOp>::new(self.source(), rhs.source()),
            N::BITS.min(RhsN::BITS),
        )
    }
}

/// Limb-wise OR; `rhs` is zero-extended or truncated to `N` bits.
impl<N: Bits, RhsN: Bits> BitOr<Uint<RhsN>> for Uint<N> {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: Uint<RhsN>) -> Self::Output {
        Uint::from_source(
            Op2::<_, _, OrOp>::new(self.source(), rhs.source()),
            N::BITS.max(RhsN::BITS),
        )
    }
}

/// Limb-wise XOR; `rhs` is zero-extended or truncated to `N` bits.
impl<N: Bits, RhsN: Bits> BitXor<Uint<RhsN>> for Uint<N> {
    type Output = Self;

    #[inline(always)]
    fn bitxor(self, rhs: Uint<RhsN>) -> Self::Output {
        Uint::from_source(
            Op2::<_, _, XorOp>::new(self.source(), rhs.source()),
            N::BITS.max(RhsN::BITS),
        )
    }
}

/// Complements the `N` bits.
impl<N: Bits> Not for Uint<N> {
    type Output = Self;

    #[inline(always)]
    fn not(self) -> Self::Output {
        Uint::from_source(TwosComplement(self.source()), usize::MAX)
    }
}
