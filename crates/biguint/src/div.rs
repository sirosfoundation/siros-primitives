//! Division and modular multiplication of any widths: normalized schoolbook long division
//! (Knuth, TAOCP vol. 2, 4.3.1, Algorithm D) whose quotient limbs come from the reciprocal steps
//! in `reciprocal`.
//!
//! The division mirrors the multiplication: each quotient limb scales the divisor, which is
//! subtracted from a window of the dividend that moves down one limb per quotient limb. Divisors of
//! one or two limbs keep the running remainder in registers. The product of the operands followed
//! by this reduction is the cheapest modular multiplication for a modulus used once (Handbook of
//! Applied Cryptography, Note 14.39).

mod reciprocal;

use core::ops::Add;

use generic_array::sequence::GenericSequence;
use generic_array::{ArrayLength, GenericArray};
use typenum::Sum;

use crate::bits::Bits;
use crate::bits::sealed::Sealed;
use crate::limb::BorrowingSub;
use crate::mul::WideningMul;
use crate::stream::{LimbStream, MulCarry, ShlBits, ShrBits, ZeroExtended, add_into, funnel_left};
use crate::uint::Uint;
use reciprocal::{NormalizedLimb, Reciprocal, Reciprocal3By2, join};

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
    shift: u32,
    limbs: DivisorLimbs<L>,
}

/// The shifted divisor with the reciprocal of its leading limbs.
enum DivisorLimbs<L: ArrayLength> {
    /// One significant limb.
    One(Reciprocal),
    /// Two significant limbs.
    Two(Reciprocal3By2),
    /// The `len >= 3` significant limbs `limbs[..len]`, led by the two that `leading` divides by.
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
        let shift = divisor.get(len - 1)?.leading_zeros();
        let limbs = match (len, divisor.as_slice()) {
            (1, [low, ..]) => {
                DivisorLimbs::One(Reciprocal::new(NormalizedLimb::new(low << shift)?))
            }
            (2, [low, high, ..]) => DivisorLimbs::Two(Reciprocal3By2::new(
                NormalizedLimb::new(funnel_left(*high, *low, shift))?,
                low << shift,
            )),
            _ => {
                // The top significant limb sheds no bits, so the limbs above it stay zero.
                let mut shifted = ShlBits::new(ZeroExtended::new(divisor.iter().copied()), shift);
                let limbs = GenericArray::<u64, L>::generate(|_| shifted.next_limb());
                let high = NormalizedLimb::new(*limbs.get(len - 1)?)?;
                let low = *len.checked_sub(2).and_then(|index| limbs.get(index))?;
                DivisorLimbs::Many {
                    leading: Reciprocal3By2::new(high, low),
                    limbs,
                    len,
                }
            }
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
        let shift = self.shift;
        match &self.limbs {
            DivisorLimbs::One(divisor) => {
                Uint::from_u64(divide_by_limb(dividend, shift, *divisor, quotient) >> shift)
            }
            DivisorLimbs::Two(divisor) => {
                Uint::from_u128(divide_by_two_limbs(dividend, shift, *divisor, quotient) >> shift)
            }
            DivisorLimbs::Many {
                limbs,
                len,
                leading,
            } => {
                let mut shifted = ShlBits::new(ZeroExtended::new(dividend.iter().copied()), shift);
                let mut window = GenericArray::<u64, W>::generate(|_| shifted.next_limb());
                if *len == L::USIZE {
                    // A divisor that fills its limbs passes its length as a constant, so the
                    // compiler unrolls the division as it does the multiplication.
                    divide_by_limbs(&mut window, limbs.as_slice(), *leading, quotient);
                } else if let Some(divisor) = limbs.get(..*len) {
                    divide_by_limbs(&mut window, divisor, *leading, quotient);
                }
                Uint::from_stream(ShrBits::new(
                    ZeroExtended::new(window.iter().copied()),
                    shift,
                ))
            }
        }
    }
}

/// The limbs of `dividend * 2^shift` from the most significant: the bits shifted out of the top
/// limb, then one limb per dividend limb.
#[inline(always)]
fn normalized_from_top(dividend: &[u64], shift: u32) -> (u64, impl Iterator<Item = u64> + '_) {
    let spill = dividend.last().map_or(0, |&top| funnel_left(0, top, shift));
    let below = dividend
        .iter()
        .rev()
        .skip(1)
        .copied()
        .chain(core::iter::once(0));
    let limbs = dividend
        .iter()
        .rev()
        .copied()
        .zip(below)
        .map(move |(limb, below)| funnel_left(limb, below, shift));
    (spill, limbs)
}

/// `dividend * 2^shift mod d` for a one-limb `d`. The quotient limbs go to `quotient`.
#[inline(always)]
fn divide_by_limb(
    dividend: &[u64],
    shift: u32,
    divisor: Reciprocal,
    quotient: &mut impl QuotientSink,
) -> u64 {
    let (spill, limbs) = normalized_from_top(dividend, shift);
    // The spill holds only the bits that the normalization shifted out, so it stays below `d`.
    (0..dividend.len())
        .rev()
        .zip(limbs)
        .fold(spill, |remainder, (index, limb)| {
            let (digit, remainder) = divisor.div_rem(remainder, limb);
            quotient.put(index, digit);
            remainder
        })
}

/// `dividend * 2^shift mod d` for a two-limb `d`. The quotient limbs go to `quotient`.
#[inline(always)]
fn divide_by_two_limbs(
    dividend: &[u64],
    shift: u32,
    divisor: Reciprocal3By2,
    quotient: &mut impl QuotientSink,
) -> u128 {
    let (spill, mut limbs) = normalized_from_top(dividend, shift);
    // The spill holds only the bits that the normalization shifted out, so the two leading limbs
    // stay below `d`.
    let leading = join(spill, limbs.next().unwrap_or(0));
    (0..dividend.len().saturating_sub(1)).rev().zip(limbs).fold(
        leading,
        |remainder, (index, limb)| {
            let (digit, remainder) = divisor.div_rem(remainder, limb);
            quotient.put(index, digit);
            remainder
        },
    )
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
    let leading_part = join(*top, *high);
    let next = *next;

    if leading_part == leading.divisor() {
        return full_quotient_limb(top, body, divisor);
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

/// The quotient limb when the leading limbs of `top` and `body` equal the divisor's: exactly
/// `2^64 - 1`, whose subtraction empties the top limb. It is rare, so it stays out of the hot path.
#[cold]
#[inline(never)]
fn full_quotient_limb(top: &mut u64, body: &mut [u64], divisor: &[u64]) -> u64 {
    *top = top.wrapping_sub(submul_into(body, divisor, u64::MAX));
    u64::MAX
}

/// Subtracts `scale * limbs` from the equally long `window` and returns the limb that the
/// subtraction owes above the window.
#[inline(always)]
fn submul_into(window: &mut [u64], limbs: &[u64], scale: u64) -> u64 {
    let mut product = MulCarry::new(scale);
    let mut borrow = 0;
    for (slot, &limb) in window.iter_mut().zip(limbs) {
        (*slot, borrow) = (*slot).sub_borrow(product.step(limb), borrow);
    }
    product.carry() + borrow
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
