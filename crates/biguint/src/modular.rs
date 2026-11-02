//! Modular addition, subtraction and negation, with one kernel: `a - b` is `a + TwosComplement(b)`
//! with a carry of one into the bottom limb, and `-a` is `0 - a`. The addend's type selects the
//! correction that brings the sum below the modulus.

use crate::bits::Bits;
use crate::kernel::{add_assign, fits, reduced, sub_assign, subtract_if_above};
use crate::nonzero::NonZeroUint;
use crate::stream::{AddOp, LimbSource, Op2CarryIter, TwosComplement, ZeroExtended, pull_limbs};
use crate::uint::Uint;

impl<N: Bits> Uint<N> {
    /// `(self + rhs) mod modulus`.
    #[inline(always)]
    #[must_use]
    pub fn add_mod(self, rhs: Self, modulus: NonZeroUint<N>) -> Self {
        let value = modulus.get();
        let (a, b) = (
            reduce_operand(self, modulus, &value),
            reduce_operand(rhs, modulus, &value),
        );
        sum_mod(&a, b.source(), &value)
    }

    /// `(self - rhs) mod modulus`.
    #[inline(always)]
    #[must_use]
    pub fn sub_mod(self, rhs: Self, modulus: NonZeroUint<N>) -> Self {
        let value = modulus.get();
        let (a, b) = (
            reduce_operand(self, modulus, &value),
            reduce_operand(rhs, modulus, &value),
        );
        sum_mod(&a, TwosComplement(b.source()), &value)
    }

    /// `-self mod modulus`.
    #[inline(always)]
    #[must_use]
    pub fn neg_mod(self, modulus: NonZeroUint<N>) -> Self {
        let value = modulus.get();
        let a = reduce_operand(self, modulus, &value);
        sum_mod(&Self::ZERO, TwosComplement(a.source()), &value)
    }
}

/// `value mod modulus`, whose value is `modulus_value`: `value` itself when its top limb is below
/// the modulus's, the remainder of the division otherwise.
#[inline(always)]
fn reduce_operand<N: Bits>(
    value: Uint<N>,
    modulus: NonZeroUint<N>,
    modulus_value: &Uint<N>,
) -> Uint<N> {
    if top(&value) < top(modulus_value) {
        value
    } else {
        value % modulus
    }
}

/// The top limb.
#[inline(always)]
pub(crate) fn top<N: Bits>(value: &Uint<N>) -> u64 {
    value.limbs.last().copied().unwrap_or(0)
}

/// The second operand of [`sum_mod`], with the correction that brings the sum below the modulus.
pub(crate) trait Addend: LimbSource {
    /// The carry into the bottom limb of the sum.
    const CARRY_IN: u64;

    /// Brings `sum`, whose carry out of the top limb is `carry`, below `modulus`.
    fn correct(sum: &mut [u64], carry: u64, modulus: &[u64]);
}

/// The limbs of `b`: `a + b` is below twice the modulus, which a carry out or a sum at or above
/// the modulus subtracts once. The top limbs decide whether the sum reaches the modulus.
impl Addend for ZeroExtended<'_> {
    const CARRY_IN: u64 = 0;

    #[inline(always)]
    fn correct(sum: &mut [u64], carry: u64, modulus: &[u64]) {
        if carry != 0 {
            // The sum is at least `2^(64 * k)`, above the modulus; the borrow cancels the carry.
            fits(sub_assign(sum, modulus));
        } else {
            reduced(subtract_if_above(sum, modulus));
        }
    }
}

/// `2^(64 * k) - b`: the sum is `a - b + 2^(64 * k)`, which carries out exactly when `a >= b`.
/// Without the carry, `a - b` borrowed, and the modulus is added back.
impl<S: LimbSource> Addend for TwosComplement<S> {
    const CARRY_IN: u64 = 1;

    #[inline(always)]
    fn correct(sum: &mut [u64], carry: u64, modulus: &[u64]) {
        if carry == 0 {
            // The carry out of the addition cancels the borrow.
            fits(add_assign(sum, modulus));
        }
    }
}

/// `(a + addend) mod modulus` for `a` and the addend's value below `modulus`.
#[inline(always)]
pub(crate) fn sum_mod<N: Bits, B: Addend>(a: &Uint<N>, addend: B, modulus: &Uint<N>) -> Uint<N> {
    let mut sum =
        Op2CarryIter::<_, _, AddOp>::with_carry(a.source().pull(), addend.pull(), B::CARRY_IN);
    let mut limbs = pull_limbs::<N::Limbs>(&mut sum);
    B::correct(&mut limbs, sum.carry(), &modulus.limbs);
    // Below the modulus, the bits above `N` are zero.
    Uint { limbs }
}
