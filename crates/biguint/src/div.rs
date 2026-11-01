//! Division and modular multiplication of any widths: normalized schoolbook long division
//! (Knuth, TAOCP vol. 2, 4.3.1, Algorithm D) whose quotient limbs come from the reciprocal steps
//! in `reciprocal`.
//!
//! The division mirrors the multiplication: each quotient limb scales the divisor's `MulIter`
//! stream, which is subtracted from a window of the dividend that moves down one limb per quotient
//! limb. The product of the operands followed by this reduction is the cheapest modular
//! multiplication for a modulus used once (Handbook of Applied Cryptography, Note 14.39).

mod reciprocal;

use core::ops::Add;

use generic_array::sequence::GenericSequence;
use generic_array::{ArrayLength, GenericArray};
use typenum::Sum;

use crate::bits::Bits;
use crate::bits::sealed::Sealed;
use crate::limb::BorrowingSub;
use crate::mul::WideningMul;
use crate::shift::{ShiftLeftStream, ShiftRightStream};
use crate::stream::{LimbStream, MulIter, ZeroExtended, add_into};
use crate::uint::Uint;
use reciprocal::{NormalizedLimb, Reciprocal, Reciprocal3By2};

impl<N: Bits> Uint<N> {
    /// `(self / divisor, self % divisor)`, or `None` when `divisor` is zero.
    #[inline]
    #[must_use]
    pub fn div_rem(self, divisor: Self) -> Option<(Self, Self)> {
        let divisor = NormalizedDivisor::new(&divisor.limbs)?;
        let mut quotient = GenericArray::<u64, N::Limbs>::default();
        let remainder = divisor.reduce::<N::Window, N>(&self.limbs, &mut quotient);
        Some((Uint::from_generic_limbs(quotient), remainder))
    }

    /// `(self * rhs) mod modulus`, or `None` when `modulus` is zero. The operands may be of any
    /// size relative to the modulus.
    #[inline]
    #[must_use]
    pub fn mul_mod<RhsN, ModN>(self, rhs: Uint<RhsN>, modulus: Uint<ModN>) -> Option<Uint<ModN>>
    where
        N: Add<RhsN>,
        RhsN: Bits,
        ModN: Bits,
        Sum<N, RhsN>: Bits,
    {
        let modulus = NormalizedDivisor::new(&modulus.limbs)?;
        let product = self.widening_mul(rhs);
        Some(
            modulus.reduce::<<Sum<N, RhsN> as Sealed>::Window, ModN>(
                &product.limbs,
                &mut DiscardQuotient,
            ),
        )
    }
}

/// A nonzero divisor, shifted left by `shift` bits until its top bit is set.
struct NormalizedDivisor<L: ArrayLength> {
    shift: usize,
    limbs: DivisorLimbs<L>,
}

/// The shifted divisor with the reciprocal of its leading limbs.
enum DivisorLimbs<L: ArrayLength> {
    /// One significant limb.
    One(Reciprocal),
    /// The `len >= 2` significant limbs `limbs[..len]`, led by the two that `leading` divides by.
    Many {
        limbs: GenericArray<u64, L>,
        len: usize,
        leading: Reciprocal3By2,
    },
}

impl<L: ArrayLength> NormalizedDivisor<L> {
    /// `None` when `divisor` is zero.
    #[inline(always)]
    fn new(divisor: &GenericArray<u64, L>) -> Option<Self> {
        let len = divisor.iter().rposition(|limb| *limb != 0)? + 1;
        let significant = divisor.get(..len)?;
        let shift = significant.last()?.leading_zeros() as usize;
        let mut shifted = ShiftLeftStream::new(significant, shift);
        let limbs =
            GenericArray::<u64, L>::generate(
                |index| {
                    if index < len { shifted.next_limb() } else { 0 }
                },
            );

        let high = NormalizedLimb::new(*limbs.get(len - 1)?)?;
        let limbs = match len.checked_sub(2).and_then(|index| limbs.get(index)) {
            Some(&low) => DivisorLimbs::Many {
                leading: Reciprocal3By2::new(high, low),
                limbs,
                len,
            },
            None => DivisorLimbs::One(Reciprocal::new(high)),
        };
        Some(Self { shift, limbs })
    }

    /// `dividend mod divisor` in `M` bits. The quotient limbs go to `quotient`. `W` holds the
    /// dividend's limbs and one more for the bits that the normalization shifts out.
    #[inline(always)]
    fn reduce<W: ArrayLength, M: Bits>(
        &self,
        dividend: &[u64],
        quotient: &mut impl QuotientSink,
    ) -> Uint<M> {
        let mut shifted = ShiftLeftStream::new(dividend, self.shift);
        let mut window = GenericArray::<u64, W>::generate(|_| shifted.next_limb());
        match &self.limbs {
            DivisorLimbs::One(divisor) => divide_by_limb(&mut window, *divisor, quotient),
            DivisorLimbs::Many {
                limbs,
                len,
                leading,
            } => {
                if let Some(divisor) = limbs.get(..*len) {
                    divide_by_limbs(&mut window, divisor, *leading, quotient);
                }
            }
        }
        Uint::from_stream(ShiftRightStream::new(&window, self.shift))
    }
}

/// Leaves `window mod d` in the low limb of `window` and zeros above it.
#[inline(always)]
fn divide_by_limb(window: &mut [u64], divisor: Reciprocal, quotient: &mut impl QuotientSink) {
    let Some((top, rest)) = window.split_last_mut() else {
        return;
    };
    // The top limb holds the bits the normalization shifted out, which stay below `d`.
    let mut remainder = core::mem::take(top);
    for (index, limb) in rest.iter_mut().enumerate().rev() {
        let (digit, next) = divisor.div_rem(remainder, core::mem::take(limb));
        quotient.put(index, digit);
        remainder = next;
    }
    if let Some(low) = rest.first_mut() {
        *low = remainder;
    }
}

/// Leaves `window mod divisor` in the low `divisor.len()` limbs of `window` and zeros above them.
/// The window shrinks from the top by one limb per quotient limb.
#[inline(always)]
fn divide_by_limbs(
    window: &mut [u64],
    divisor: &[u64],
    leading: Reciprocal3By2,
    quotient: &mut impl QuotientSink,
) {
    let mut prefix = window;
    while let Some(index) = prefix.len().checked_sub(divisor.len() + 1) {
        let Some((_, part)) = prefix.split_at_mut_checked(index) else {
            return;
        };
        quotient.put(index, quotient_limb(part, divisor, leading));
        prefix = match prefix.split_last_mut() {
            Some((_, rest)) => rest,
            None => return,
        };
    }
}

/// Reduces the `divisor.len() + 1` limbs of `part` below the divisor and returns the quotient
/// limb: the 3-by-2 step on the leading limbs, the scaled divisor subtracted from the others, and
/// an add-back when the step overestimated by one.
#[inline(always)]
fn quotient_limb(part: &mut [u64], divisor: &[u64], leading: Reciprocal3By2) -> u64 {
    let Some((top, body)) = part.split_last_mut() else {
        return 0;
    };
    let [.., next, high] = body else {
        return 0;
    };
    let leading_part = (u128::from(*top) << 64) | u128::from(*high);
    let next = *next;

    if leading_part == leading.divisor() {
        // The quotient limb is exactly `2^64 - 1`; the subtraction empties the top limb.
        *top = top.wrapping_sub(submul_into(body, divisor, u64::MAX));
        return u64::MAX;
    }

    let (digit, remainder) = leading.div_rem(leading_part, next);
    let ([low @ .., remainder_low, remainder_high], [divisor_low @ .., _, _]) =
        (&mut *body, divisor)
    else {
        return digit;
    };
    let (remainder, borrowed) =
        remainder.overflowing_sub(u128::from(submul_into(low, divisor_low, digit)));
    *remainder_low = remainder as u64;
    *remainder_high = (remainder >> 64) as u64;
    *top = 0;

    if borrowed {
        cancel_borrow(add_into(ZeroExtended::new(divisor.iter().copied()), body));
        digit - 1
    } else {
        digit
    }
}

/// Subtracts `scale * limbs` from the equally long `window` and returns the limb that the
/// subtraction owes above the window.
#[inline(always)]
fn submul_into(window: &mut [u64], limbs: &[u64], scale: u64) -> u64 {
    let mut product = MulIter::new(ZeroExtended::new(limbs.iter().copied()), scale);
    let mut borrow = 0;
    for slot in window {
        (*slot, borrow) = (*slot).sub_borrow(product.next_limb(), borrow);
    }
    product.next_limb() + borrow
}

/// Drops the carry out of an add-back, which cancels the borrow that the add-back repairs.
#[inline(always)]
fn cancel_borrow(_carry: u64) {}

/// Receives the quotient limbs of a division.
trait QuotientSink {
    /// Records quotient limb `index`.
    fn put(&mut self, index: usize, limb: u64);
}

impl<L: ArrayLength> QuotientSink for GenericArray<u64, L> {
    #[inline(always)]
    fn put(&mut self, index: usize, limb: u64) {
        if let Some(slot) = self.get_mut(index) {
            *slot = limb;
        }
    }
}

/// Drops the quotient of a reduction.
struct DiscardQuotient;

impl QuotientSink for DiscardQuotient {
    #[inline(always)]
    fn put(&mut self, _index: usize, _limb: u64) {}
}
