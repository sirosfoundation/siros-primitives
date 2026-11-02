use core::ops::{Add, Sub};

use generic_array::GenericArray;

use crate::bits::Bits;
use crate::limb::{BorrowingSub, CarryingAdd};
use crate::stream::{AddOp, LimbSource, Op2CarryIter, SubOp};
use crate::uint::Uint;

impl<N: Bits> Uint<N> {
    /// `self + rhs + carry` as `(sum mod 2^N, carry_out)`; `carry_out` is bit `N` of the sum.
    #[inline(always)]
    #[must_use]
    pub fn carrying_add(self, rhs: Self, carry: bool) -> (Self, bool) {
        let mut out = GenericArray::<u64, N::Limbs>::default();
        let mut limb_carry = u64::from(carry);
        for ((slot, &lhs_limb), &rhs_limb) in out.iter_mut().zip(&self.limbs).zip(&rhs.limbs) {
            (*slot, limb_carry) = lhs_limb.add_carry(rhs_limb, limb_carry);
        }

        // A full top limb carries out through the limb carry. A partial top limb keeps bit `N`
        // inside itself, above the bits that `from_generic_limbs` keeps.
        let top_bits = N::BITS % 64;
        let carry_out = if top_bits == 0 {
            limb_carry
        } else {
            out.last().copied().unwrap_or(0) >> top_bits
        };

        (Uint::from_generic_limbs(out), carry_out != 0)
    }

    /// `self - rhs - borrow` as `(difference mod 2^N, borrow_out)`; `borrow_out` is whether
    /// `self < rhs + borrow`.
    #[inline(always)]
    #[must_use]
    pub fn borrowing_sub(self, rhs: Self, borrow: bool) -> (Self, bool) {
        let mut out = GenericArray::<u64, N::Limbs>::default();
        let mut limb_borrow = u64::from(borrow);
        for ((slot, &lhs_limb), &rhs_limb) in out.iter_mut().zip(&self.limbs).zip(&rhs.limbs) {
            (*slot, limb_borrow) = lhs_limb.sub_borrow(rhs_limb, limb_borrow);
        }

        // Both operands are below `2^N`, so the top limb borrows exactly when the `N`-bit
        // difference does.
        (Uint::from_generic_limbs(out), limb_borrow != 0)
    }

    /// `(self + rhs) mod 2^N`.
    #[inline(always)]
    #[must_use]
    pub fn wrapping_add(self, rhs: Self) -> Self {
        self + rhs
    }

    /// `(self + rhs) mod 2^N`, and whether the sum reaches `2^N`.
    #[inline(always)]
    #[must_use]
    pub fn overflowing_add(self, rhs: Self) -> (Self, bool) {
        self.carrying_add(rhs, false)
    }

    /// `self + rhs`, or `None` when the sum reaches `2^N`.
    #[inline(always)]
    #[must_use]
    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        let (sum, overflow) = self.overflowing_add(rhs);
        (!overflow).then_some(sum)
    }

    /// `self + rhs`, or [`Self::MAX`] when the sum reaches `2^N`.
    #[inline(always)]
    #[must_use]
    pub fn saturating_add(self, rhs: Self) -> Self {
        self.checked_add(rhs).unwrap_or(Self::MAX)
    }

    /// `(self - rhs) mod 2^N`.
    #[inline(always)]
    #[must_use]
    pub fn wrapping_sub(self, rhs: Self) -> Self {
        self - rhs
    }

    /// `(self - rhs) mod 2^N`, and whether `self < rhs`.
    #[inline(always)]
    #[must_use]
    pub fn overflowing_sub(self, rhs: Self) -> (Self, bool) {
        self.borrowing_sub(rhs, false)
    }

    /// `self - rhs`, or `None` when `self < rhs`.
    #[inline(always)]
    #[must_use]
    pub fn checked_sub(self, rhs: Self) -> Option<Self> {
        let (difference, overflow) = self.overflowing_sub(rhs);
        (!overflow).then_some(difference)
    }

    /// `self - rhs`, or `0` when `self < rhs`.
    #[inline(always)]
    #[must_use]
    pub fn saturating_sub(self, rhs: Self) -> Self {
        self.checked_sub(rhs).unwrap_or(Self::ZERO)
    }

    /// `(-self) mod 2^N`.
    #[inline(always)]
    #[must_use]
    pub fn wrapping_neg(self) -> Self {
        Self::ZERO - self
    }

    /// `-self`, which is `Some` only for zero.
    #[inline(always)]
    #[must_use]
    pub fn checked_neg(self) -> Option<Self> {
        self.is_zero().then_some(self)
    }
}

/// `(self + rhs) mod 2^N`.
impl<N: Bits, RhsN: Bits> Add<Uint<RhsN>> for Uint<N> {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Uint<RhsN>) -> Self::Output {
        Uint::from_stream(
            Op2CarryIter::<_, _, AddOp>::new(self.source().pull(), rhs.source().pull()),
            N::BITS.max(RhsN::BITS) + 1,
        )
    }
}

/// `(self - rhs) mod 2^N`.
impl<N: Bits, RhsN: Bits> Sub<Uint<RhsN>> for Uint<N> {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Uint<RhsN>) -> Self::Output {
        Uint::from_stream(
            Op2CarryIter::<_, _, SubOp>::new(self.source().pull(), rhs.source().pull()),
            usize::MAX,
        )
    }
}
