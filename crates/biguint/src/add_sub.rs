use core::ops::{Add, Sub};

use generic_array::GenericArray;

use crate::bits::Bits;
use crate::limb::{BorrowingSub, CarryingAdd};
#[cfg(all(target_arch = "wasm32", not(wide_arithmetic)))]
use crate::stream::AddAheadOp;
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

/// `(self + rhs) mod 2^N`. The chain starts from `rhs`, whose limbs LLVM loads into registers
/// while each step adds a limb of `self` from memory: the 512-bit sum took 9.9 cycles from `self`
/// and 9.4 from `rhs`.
impl<N: Bits, RhsN: Bits> Add<Uint<RhsN>> for Uint<N> {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Uint<RhsN>) -> Self::Output {
        #[cfg(target_arch = "wasm32")]
        if const {
            N::LIMBS == RhsN::LIMBS && (N::LIMBS == 4 || (N::LIMBS == 2 && cfg!(wide_arithmetic)))
        } {
            let mut sum = Self::ZERO;
            pair_sum(sum.limbs.as_mut_slice(), &self.limbs, &rhs.limbs);
            if let Some(top) = sum.limbs.as_mut_slice().last_mut() {
                *top &= N::TOP_MASK;
            }
            return sum;
        }
        #[cfg(all(target_arch = "wasm32", not(wide_arithmetic)))]
        if const { N::LIMBS >= AHEAD_SUM_LIMBS || RhsN::LIMBS >= AHEAD_SUM_LIMBS } {
            return Uint::from_stream(
                Op2CarryIter::<_, _, AddAheadOp>::new(rhs.source().pull(), self.source().pull()),
                N::BITS.max(RhsN::BITS) + 1,
            );
        }
        Uint::from_stream(
            Op2CarryIter::<_, _, AddOp>::new(rhs.source().pull(), self.source().pull()),
            N::BITS.max(RhsN::BITS) + 1,
        )
    }
}

/// `a + b` into `out` by pairs of limbs, each pair one `i64.add128` with wide arithmetic. The 256-bit
/// sum took Firefox 11.0 cycles against 12.2 limb by limb, and Chrome 9.1 against 8.7; without wide
/// arithmetic Firefox 10.0 against 10.3, and Chrome 9.6 against 10.3. The other sums run limb by
/// limb: with wide arithmetic the 512-bit one took Firefox 31.1 by pairs against 32.1, and Chrome
/// 26.1 against 20.8; without, the 128-bit one Chrome 4.1 against 4.0, the 512-bit one Chrome 27.5
/// against 26.3.
#[cfg(target_arch = "wasm32")]
#[inline(always)]
fn pair_sum(out: &mut [u64], a: &[u64], b: &[u64]) {
    let limb = |limbs: &[u64], index: usize| limbs.get(index).copied().unwrap_or(0);
    let mut carry = false;
    let mut index = 0;
    while index < out.len() {
        let x = u128::from(limb(a, index)) | (u128::from(limb(a, index + 1)) << 64);
        let y = u128::from(limb(b, index)) | (u128::from(limb(b, index + 1)) << 64);
        let (sum, first) = x.overflowing_add(y);
        let (sum, second) = sum.overflowing_add(u128::from(carry));
        carry = first | second;
        if let Some(slot) = out.get_mut(index) {
            *slot = sum as u64;
        }
        if let Some(slot) = out.get_mut(index + 1) {
            *slot = (sum >> 64) as u64;
        }
        index += 2;
    }
}

/// The shortest sums that carry by [`AddAheadOp`] on WebAssembly without wide arithmetic. The
/// 384-bit sum took Chrome 16.3 cycles in throughput and 14.8 in latency against 16.3 and 15.4 with
/// the carry of a second overflowing addition, Firefox 15.3 and 15.5 against 15.9 and 15.5.
#[cfg(all(target_arch = "wasm32", not(wide_arithmetic)))]
const AHEAD_SUM_LIMBS: usize = 6;

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
