//! Modular addition, subtraction and negation, with one kernel: `a - b` is `a + TwosComplement(b)`,
//! and `-a` is `0 - a`. The addend's type selects the carry chain and the correction that brings
//! the sum below the modulus.

use crate::bits::Bits;
use crate::kernel::{
    add_unless_carry, add_unless_carry_masked, reduced, subtract_when_above_in_place,
    subtract_when_above_masked,
};
use crate::nonzero::NonZeroUint;
use generic_array::GenericArray;

use crate::stream::{
    AddOp, LimbSource, Op2CarryIter, SubOp, TwosComplement, ZeroExtended, pull_limbs,
};
use crate::uint::Uint;

impl<N: Bits> Uint<N> {
    /// `(self + rhs) mod modulus`.
    #[inline(always)]
    #[must_use]
    pub fn add_mod(self, rhs: Self, modulus: NonZeroUint<N>) -> Self {
        below_modulus(
            self,
            rhs,
            modulus,
            #[inline(always)]
            |a, b, m| sum_mod(a, b.source(), m),
        )
    }

    /// `(self - rhs) mod modulus`.
    #[inline(always)]
    #[must_use]
    pub fn sub_mod(self, rhs: Self, modulus: NonZeroUint<N>) -> Self {
        below_modulus(
            self,
            rhs,
            modulus,
            #[inline(always)]
            |a, b, m| sum_mod(a, TwosComplement(b.source()), m),
        )
    }

    /// `-self mod modulus`.
    #[inline(always)]
    #[must_use]
    pub fn neg_mod(self, modulus: NonZeroUint<N>) -> Self {
        below_modulus(
            Self::ZERO,
            self,
            modulus,
            #[inline(always)]
            |zero, a, m| sum_mod(zero, TwosComplement(a.source()), m),
        )
    }
}

/// The most limbs whose operands [`below_modulus`] selects into registers.
const SELECTED_LIMBS: usize = 4;

/// `sum(a mod modulus, b mod modulus, modulus)`. An operand whose top limb is below the modulus's
/// is below the modulus; any other reduces by a division.
///
/// Up to [`SELECTED_LIMBS`] limbs, each operand is a select between itself and its remainder,
/// which stays in registers. Longer operands would go through copies: there, `sum` inlines twice,
/// once on the operands where they lie and once on their remainders. On Zen 3 the two copies of
/// `sum` take 18% to 25% fewer cycles than the selects from 384 bits up, and the selects take as
/// many or fewer up to 256 bits.
#[inline(always)]
fn below_modulus<N: Bits>(
    a: Uint<N>,
    b: Uint<N>,
    modulus: NonZeroUint<N>,
    sum: impl Fn(&Uint<N>, &Uint<N>, &Uint<N>) -> Uint<N>,
) -> Uint<N> {
    let m = modulus.as_ref();
    if N::LIMBS <= SELECTED_LIMBS {
        let a = if below(&a, m) { a } else { a % modulus };
        let b = if below(&b, m) { b } else { b % modulus };
        sum(&a, &b, m)
    } else if below(&a, m) && below(&b, m) {
        sum(&a, &b, m)
    } else {
        sum(&(a % modulus), &(b % modulus), m)
    }
}

/// Whether the top limb of `value` is below the top limb of `modulus`, which puts `value` below
/// `modulus`.
#[inline(always)]
fn below<N: Bits>(value: &Uint<N>, modulus: &Uint<N>) -> bool {
    top(value) < top(modulus)
}

/// The top limb.
#[inline(always)]
pub(crate) fn top<N: Bits>(value: &Uint<N>) -> u64 {
    value.limbs.last().copied().unwrap_or(0)
}

/// The second operand of [`sum_mod`], with the correction that brings the sum below the modulus.
pub(crate) trait Addend: LimbSource {
    /// The limbs of `a + addend` in `N` bits, and the carry out of the top limb.
    fn sum<N: Bits>(a: &Uint<N>, addend: Self) -> (GenericArray<u64, N::Limbs>, u64);

    /// Brings `sum`, whose carry out of the top limb is `carry`, below `modulus`.
    fn correct(sum: &mut [u64], carry: u64, modulus: &[u64]);

    /// [`Self::correct`] under masks at every length: writes the corrected `sum` to `out`.
    fn correct_masked(out: &mut [u64], sum: &[u64], carry: u64, modulus: &[u64]);
}

/// The limbs of `b`: `a + b` is below twice the modulus, which a carry out or a sum at or above
/// the modulus subtracts once.
impl Addend for ZeroExtended<'_> {
    #[inline(always)]
    fn sum<N: Bits>(a: &Uint<N>, addend: Self) -> (GenericArray<u64, N::Limbs>, u64) {
        let mut sum = Op2CarryIter::<_, _, AddOp>::new(a.source().pull(), addend.pull());
        (pull_limbs::<N::Limbs>(&mut sum), sum.carry())
    }

    #[inline(always)]
    fn correct(sum: &mut [u64], carry: u64, modulus: &[u64]) {
        reduced(subtract_when_above_in_place(sum, modulus, carry));
    }

    #[inline(always)]
    fn correct_masked(out: &mut [u64], sum: &[u64], carry: u64, modulus: &[u64]) {
        subtract_when_above_masked(out, sum, modulus, carry);
    }
}

/// `2^(64 * k) - b`: the sum is `a - b + 2^(64 * k)`, which carries out exactly when `a >= b`. It
/// runs as the subtraction `a - b`, whose borrow is the opposite of that carry. Without the carry,
/// `a - b` borrowed, and the modulus is added back; the carry out of that addition cancels the
/// borrow.
impl<S: LimbSource> Addend for TwosComplement<S> {
    #[inline(always)]
    fn sum<N: Bits>(a: &Uint<N>, addend: Self) -> (GenericArray<u64, N::Limbs>, u64) {
        let mut difference = Op2CarryIter::<_, _, SubOp>::new(a.source().pull(), addend.0.pull());
        (
            pull_limbs::<N::Limbs>(&mut difference),
            difference.carry() ^ 1,
        )
    }

    #[inline(always)]
    fn correct(sum: &mut [u64], carry: u64, modulus: &[u64]) {
        add_unless_carry(sum, modulus, carry);
    }

    #[inline(always)]
    fn correct_masked(out: &mut [u64], sum: &[u64], carry: u64, modulus: &[u64]) {
        add_unless_carry_masked(out, sum, modulus, carry);
    }
}

/// `(a + addend) mod modulus` for `a` and the addend's value below `modulus`. The sum takes its
/// correction in place.
#[inline(always)]
pub(crate) fn sum_mod<N: Bits, B: Addend>(a: &Uint<N>, addend: B, modulus: &Uint<N>) -> Uint<N> {
    let (limbs, carry) = B::sum(a, addend);
    let mut out = Uint::<N> { limbs };
    B::correct(&mut out.limbs, carry, &modulus.limbs);
    // Below the modulus, the bits above `N` are zero.
    out
}

/// [`sum_mod`] with the correction under masks: no branch and no memory address depends on `a` or
/// the addend. The sum runs into registers or a copy, and the correction writes the result.
#[inline(always)]
pub(crate) fn sum_mod_masked<N: Bits, B: Addend>(
    a: &Uint<N>,
    addend: B,
    modulus: &Uint<N>,
) -> Uint<N> {
    let (limbs, carry) = B::sum(a, addend);
    let mut out = Uint::<N>::ZERO;
    B::correct_masked(&mut out.limbs, &limbs, carry, &modulus.limbs);
    out
}
