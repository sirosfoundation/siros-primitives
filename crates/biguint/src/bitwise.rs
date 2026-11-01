use core::ops::{BitAnd, BitOr, BitXor, Not};

use crate::bits::Bits;
use crate::stream::{AndOp, NotOp, Op1Iter, Op2Iter, OrOp, XorOp};
use crate::uint::Uint;

/// Limb-wise AND; `rhs` is zero-extended or truncated to `N` bits.
impl<N: Bits, RhsN: Bits> BitAnd<Uint<RhsN>> for Uint<N> {
    type Output = Self;

    #[inline(always)]
    fn bitand(self, rhs: Uint<RhsN>) -> Self::Output {
        Uint::from_stream(Op2Iter::<_, _, AndOp>::new(
            self.into_stream(),
            rhs.into_stream(),
        ))
    }
}

/// Limb-wise OR; `rhs` is zero-extended or truncated to `N` bits.
impl<N: Bits, RhsN: Bits> BitOr<Uint<RhsN>> for Uint<N> {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: Uint<RhsN>) -> Self::Output {
        Uint::from_stream(Op2Iter::<_, _, OrOp>::new(
            self.into_stream(),
            rhs.into_stream(),
        ))
    }
}

/// Limb-wise XOR; `rhs` is zero-extended or truncated to `N` bits.
impl<N: Bits, RhsN: Bits> BitXor<Uint<RhsN>> for Uint<N> {
    type Output = Self;

    #[inline(always)]
    fn bitxor(self, rhs: Uint<RhsN>) -> Self::Output {
        Uint::from_stream(Op2Iter::<_, _, XorOp>::new(
            self.into_stream(),
            rhs.into_stream(),
        ))
    }
}

/// Complements the `N` bits.
impl<N: Bits> Not for Uint<N> {
    type Output = Self;

    #[inline(always)]
    fn not(self) -> Self::Output {
        Uint::from_stream(Op1Iter::<_, NotOp>::new(self.into_stream()))
    }
}
