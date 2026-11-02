//! Division and modular multiplication of any widths: normalized schoolbook long division
//! (Knuth, TAOCP vol. 2, 4.3.1, Algorithm D) whose quotient limbs come from the reciprocal steps
//! in `reciprocal`, and divide-and-conquer division in `dc` for long divisors that fill their
//! width.
//!
//! The division mirrors the multiplication: each quotient limb scales the divisor, which is
//! subtracted from a window of the dividend that moves down one limb per quotient limb. Divisors of
//! one or two limbs keep the running remainder in registers. The product of the operands followed
//! by this reduction is the cheapest modular multiplication for a modulus used once (Handbook of
//! Applied Cryptography, Note 14.39).

pub(crate) mod dc;
pub(crate) mod reciprocal;

use core::num::NonZero;
use core::ops::{Add, Div, Rem};

use generic_array::sequence::GenericSequence;
use generic_array::{ArrayLength, GenericArray};
use typenum::Sum;

use crate::bits::Bits;
use crate::bits::sealed::Sealed;
use crate::kernel::{add_assign, fits, split, split_mut, sub_mul_row, subtract_if_above};
use crate::mul::WideningMul;
use crate::nonzero::NonZeroUint;
use crate::recursion::Halvings;
use crate::stream::{LimbStream, ShlBits, ShrBits, ZeroExtended, funnel_left, funnel_right};
use crate::uint::Uint;
use reciprocal::{NormalizedLimb, Reciprocal, Reciprocal3By2, join};

impl<N: Bits> Uint<N> {
    /// `(self / divisor, self % divisor)`. The divisor may be of any width; the remainder takes the
    /// divisor's.
    #[inline(always)]
    #[must_use]
    pub fn div_rem<M: Bits>(self, divisor: NonZeroUint<M>) -> (Self, Uint<M>) {
        let mut quotient = GenericArray::<u64, N::Limbs>::default();
        let remainder = divide::<N::Window, M>(&self.limbs, divisor, &mut quotient);
        (Uint::from_generic_limbs(quotient), remainder)
    }

    /// `(self / divisor, self % divisor)`, or `None` when `divisor` is zero. The divisor may be of
    /// any width; the remainder takes the divisor's.
    #[inline(always)]
    #[must_use]
    pub fn checked_div_rem<M: Bits>(self, divisor: Uint<M>) -> Option<(Self, Uint<M>)> {
        NonZeroUint::new(divisor).map(|divisor| self.div_rem(divisor))
    }

    /// `(self * rhs) mod modulus`, or `None` when `modulus` is zero. The operands may be of any
    /// size relative to the modulus.
    #[inline(always)]
    #[must_use]
    pub fn mul_mod<RhsN, ModN>(self, rhs: Uint<RhsN>, modulus: Uint<ModN>) -> Option<Uint<ModN>>
    where
        N: Add<RhsN>,
        RhsN: Bits,
        ModN: Bits,
        Sum<N, RhsN>: Bits,
    {
        let product = self.widening_mul(rhs);
        divide_checked::<_, <Sum<N, RhsN> as Sealed>::Window, ModN>(
            &product.limbs,
            &modulus.limbs,
            &mut DiscardQuotient,
        )
    }
}

/// `self / divisor`.
impl<N: Bits, M: Bits> Div<NonZeroUint<M>> for Uint<N> {
    type Output = Self;

    #[inline(always)]
    fn div(self, divisor: NonZeroUint<M>) -> Self::Output {
        self.div_rem(divisor).0
    }
}

/// `self mod divisor`, in the divisor's width.
impl<N: Bits, M: Bits> Rem<NonZeroUint<M>> for Uint<N> {
    type Output = Uint<M>;

    #[inline(always)]
    fn rem(self, divisor: NonZeroUint<M>) -> Self::Output {
        divide::<N::Window, M>(&self.limbs, divisor, &mut DiscardQuotient)
    }
}

/// `dividend mod divisor` in `M` bits. The quotient limbs go to `quotient`. `W` holds the
/// dividend's limbs and one more for the bits that the normalization shifts out.
///
/// A dividend and a divisor that fit in `u128` divide natively: the compiler emits the hardware
/// division, which beats computing a reciprocal for one or two quotient limbs.
#[inline(always)]
pub(crate) fn divide<W: ArrayLength, M: Bits>(
    dividend: &[u64],
    divisor: NonZeroUint<M>,
    quotient: &mut (impl QuotientSink + ?Sized),
) -> Uint<M> {
    if Native::<W, M::Limbs>::FITS
        && let Some(divisor) = divisor.native()
    {
        return native(dividend, divisor, quotient);
    }
    let divisor = divisor.get().limbs;
    // A nonzero value whose upper limbs are zero has its nonzero limb at the bottom.
    let len = divisor
        .iter()
        .skip(1)
        .rposition(|limb| *limb != 0)
        .map_or(1, |index| index + 2);
    long::<_, W, M>(dividend, &divisor, len, quotient)
}

/// [`divide`] by a divisor that may be zero: `None` for zero.
#[inline(always)]
fn divide_checked<L: ArrayLength, W: ArrayLength, M: Bits>(
    dividend: &[u64],
    divisor: &GenericArray<u64, L>,
    quotient: &mut (impl QuotientSink + ?Sized),
) -> Option<Uint<M>> {
    if Native::<W, L>::FITS {
        return Some(native(dividend, NonZero::new(wide(divisor))?, quotient));
    }
    let len = divisor.iter().rposition(|limb| *limb != 0)? + 1;
    Some(long::<_, W, M>(dividend, divisor, len, quotient))
}

/// Whether a dividend in a `W`-limb window and an `L`-limb divisor both fit in `u128`.
struct Native<W, L>(core::marker::PhantomData<(W, L)>);

impl<W: ArrayLength, L: ArrayLength> Native<W, L> {
    const FITS: bool = W::USIZE <= 3 && L::USIZE <= 2;
}

/// `dividend mod divisor` by `u128` division; the dividend has at most two limbs. One-limb
/// operands narrow to a single `u64` division.
#[inline(always)]
fn native<M: Bits>(
    dividend: &[u64],
    divisor: NonZero<u128>,
    quotient: &mut (impl QuotientSink + ?Sized),
) -> Uint<M> {
    let dividend = wide(dividend);
    let whole = dividend / divisor;
    quotient.put(0, whole as u64);
    quotient.put(1, (whole >> 64) as u64);
    Uint::from_u128(dividend % divisor)
}

/// The value of at most two limbs.
#[inline(always)]
fn wide(limbs: &[u64]) -> u128 {
    limbs
        .iter()
        .rev()
        .fold(0, |value, &limb| (value << 64) | u128::from(limb))
}

/// `dividend mod divisor` in `M` bits for a divisor whose low `len` limbs hold its value, the top
/// one nonzero. The divisor shifts left until its top bit is set, the dividend with it, and the
/// remainder shifts back; a divisor whose top bit is set runs a copy of each path without shifts.
/// Divisors of one or two significant limbs divide in registers.
#[inline(always)]
fn long<L: ArrayLength, W: ArrayLength, M: Bits>(
    dividend: &[u64],
    divisor: &GenericArray<u64, L>,
    len: usize,
    quotient: &mut (impl QuotientSink + ?Sized),
) -> Uint<M> {
    match split(divisor, len).0 {
        [low] => {
            if low.leading_zeros() == 0 {
                Uint::from_u64(by_limb(dividend, *low, 0, quotient))
            } else {
                Uint::from_u64(by_limb(dividend, *low, low.leading_zeros(), quotient))
            }
        }
        [low, high] => {
            if high.leading_zeros() == 0 {
                Uint::from_u128(by_two_limbs(dividend, *low, *high, 0, quotient))
            } else {
                let shift = high.leading_zeros();
                Uint::from_u128(by_two_limbs(dividend, *low, *high, shift, quotient))
            }
        }
        [.., top] => by_limbs::<L, W, M>(dividend, divisor, len, top.leading_zeros(), quotient),
        // The divisor has a significant limb.
        [] => Uint::ZERO,
    }
}

/// `dividend mod divisor` for the one-limb divisor `low`, `shift` its leading zeros.
#[inline(always)]
fn by_limb(
    dividend: &[u64],
    low: u64,
    shift: u32,
    quotient: &mut (impl QuotientSink + ?Sized),
) -> u64 {
    let divisor = Reciprocal::new(NormalizedLimb::top_set(low << shift));
    divide_by_limb(dividend, shift, divisor, quotient) >> shift
}

/// `dividend mod divisor` for the two-limb divisor `high:low`, `shift` the leading zeros of `high`.
#[inline(always)]
fn by_two_limbs(
    dividend: &[u64],
    low: u64,
    high: u64,
    shift: u32,
    quotient: &mut (impl QuotientSink + ?Sized),
) -> u128 {
    let divisor = Reciprocal3By2::new(
        NormalizedLimb::top_set(funnel_left(high, low, shift)),
        low << shift,
    );
    let remainder = divide_by_two_limbs(dividend, shift, divisor, quotient);
    let (low, high) = (remainder as u64, (remainder >> 64) as u64);
    join(high >> shift, funnel_right(low, high, shift))
}

/// `dividend mod divisor` for a divisor whose low `len` limbs hold its value, `len >= 3`, and
/// `shift` the leading zeros of its top significant limb.
#[inline(always)]
fn by_limbs<L: ArrayLength, W: ArrayLength, M: Bits>(
    dividend: &[u64],
    divisor: &GenericArray<u64, L>,
    len: usize,
    shift: u32,
    quotient: &mut (impl QuotientSink + ?Sized),
) -> Uint<M> {
    let (normalized, mut window) = if shift == 0 {
        normalize::<L, W>(dividend, divisor, 0)
    } else {
        normalize::<L, W>(dividend, divisor, shift)
    };
    let (significant, _) = split(&normalized, len);
    let [.., next, top] = significant else {
        // The divisor has three significant limbs and more.
        return Uint::ZERO;
    };
    let leading = Reciprocal3By2::new(NormalizedLimb::top_set(*top), *next);
    if len == L::USIZE {
        // A divisor that fills its limbs passes its length as a constant, so the compiler
        // unrolls the division as it does the multiplication.
        if Shape::<W, L>::DIVIDE_AND_CONQUER {
            let mut scratch = GenericArray::<u64, M::DivisionScratch>::default();
            dc::divide_window::<Halvings>(
                &mut window,
                normalized.as_slice(),
                leading,
                quotient,
                &mut scratch,
            );
        } else {
            divide_by_limbs(&mut window, normalized.as_slice(), leading, quotient);
        }
    } else {
        divide_by_limbs(&mut window, significant, leading, quotient);
    }
    if shift == 0 {
        Uint::from_stream(ZeroExtended::new(&window), usize::MAX)
    } else {
        Uint::from_stream(ShrBits::new(ZeroExtended::new(&window), shift), usize::MAX)
    }
}

/// The divisor and the dividend shifted left by `shift`, the dividend in a window of `W` limbs.
/// The divisor's top significant limb sheds no bits, so its limbs above it stay zero; the bits
/// shifted out of the dividend's top limb fill the window's limb above it.
#[inline(always)]
fn normalize<L: ArrayLength, W: ArrayLength>(
    dividend: &[u64],
    divisor: &GenericArray<u64, L>,
    shift: u32,
) -> (GenericArray<u64, L>, GenericArray<u64, W>) {
    let mut shifted = ShlBits::new(ZeroExtended::new(divisor), shift);
    let normalized = GenericArray::<u64, L>::generate(|_| shifted.next_limb());
    let mut window = GenericArray::<u64, W>::default();
    {
        let (body, above) = split_mut(&mut window, dividend.len());
        let mut shifted = ShlBits::new(ZeroExtended::new(dividend), shift);
        for slot in body {
            *slot = shifted.next_limb();
        }
        if let Some(slot) = above.first_mut() {
            *slot = dividend.last().map_or(0, |&top| funnel_left(0, top, shift));
        }
    }
    (normalized, window)
}

/// The division of a `W`-limb window by a divisor that fills its `L` limbs, decided at compile
/// time.
struct Shape<W, L>(core::marker::PhantomData<(W, L)>);

impl<W: ArrayLength, L: ArrayLength> Shape<W, L> {
    /// Both the divisor and the quotient reach the divide-and-conquer threshold.
    const DIVIDE_AND_CONQUER: bool =
        L::USIZE >= dc::THRESHOLD && W::USIZE >= L::USIZE + dc::THRESHOLD;
}

/// The limbs of `dividend * 2^shift` from the most significant: the bits shifted out of the top
/// limb, then one limb per dividend limb.
#[inline(always)]
fn normalized_from_top(dividend: &[u64], shift: u32) -> (u64, impl Iterator<Item = u64> + '_) {
    let spill = dividend.last().map_or(0, |&top| funnel_left(0, top, shift));
    // A range of indices gives the loops a trip count that the compiler computes.
    let limb_at = |index: usize| dividend.get(index).copied().unwrap_or(0);
    let limbs = (0..dividend.len()).rev().map(move |index| {
        let below = index.checked_sub(1).map_or(0, limb_at);
        funnel_left(limb_at(index), below, shift)
    });
    (spill, limbs)
}

/// `dividend * 2^shift mod d` for a one-limb `d`. The quotient limbs go to `quotient`.
#[inline(always)]
fn divide_by_limb(
    dividend: &[u64],
    shift: u32,
    divisor: Reciprocal,
    quotient: &mut (impl QuotientSink + ?Sized),
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
    quotient: &mut (impl QuotientSink + ?Sized),
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

/// Divides `numerator` by the normalized `divisor`, whose two leading limbs `leading` divides by:
/// writes the low `numerator.len() - divisor.len()` quotient limbs to `quotient`, leaves the
/// remainder in the low `divisor.len()` limbs of `numerator` and zeros above them, and returns
/// the top quotient limb, `0` or `1`.
#[inline(always)]
pub(crate) fn schoolbook(
    numerator: &mut [u64],
    divisor: &[u64],
    quotient: &mut [u64],
    leading: Reciprocal3By2,
) -> u64 {
    let above = {
        let (_, top) = split_mut(numerator, numerator.len().saturating_sub(divisor.len()));
        subtract_if_above(top, divisor)
    };
    divide_by_limbs(numerator, divisor, leading, quotient);
    u64::from(above)
}

/// Leaves `window mod divisor` in the low `divisor.len()` limbs of `window` and zeros above them.
/// The window shrinks from the top by one limb per quotient limb. The top `divisor.len()` limbs of
/// `window` are below `divisor`.
#[inline(always)]
fn divide_by_limbs(
    window: &mut [u64],
    divisor: &[u64],
    leading: Reciprocal3By2,
    quotient: &mut (impl QuotientSink + ?Sized),
) {
    let mut prefix = window;
    // Below a zero top limb the top quotient limb is `0` or `1`: the limbs under it are below
    // twice the normalized divisor.
    if prefix.last() == Some(&0)
        && let Some(index) = prefix.len().checked_sub(divisor.len() + 1)
    {
        prefix = match prefix.split_last_mut() {
            Some((_, rest)) => {
                let (_, part) = split_mut(rest, index);
                quotient.put(index, u64::from(subtract_if_above(part, divisor)));
                rest
            }
            None => return,
        };
    }
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
        remainder.overflowing_sub(u128::from(sub_mul_row(low, divisor_low, digit)));
    *remainder_low = remainder as u64;
    *remainder_high = (remainder >> 64) as u64;
    *top = 0;

    if borrowed {
        add_back(body, divisor);
        digit - 1
    } else {
        digit
    }
}

/// The quotient limb when the leading limbs of `top` and `body` equal the divisor's: exactly
/// `2^64 - 1`, whose subtraction empties the top limb.
#[inline(always)]
fn full_quotient_limb(top: &mut u64, body: &mut [u64], divisor: &[u64]) -> u64 {
    *top = top.wrapping_sub(sub_mul_row(body, divisor, u64::MAX));
    u64::MAX
}

/// Adds the divisor back after a step that overestimated by one; its carry cancels the borrow.
#[inline(always)]
fn add_back(body: &mut [u64], divisor: &[u64]) {
    fits(add_assign(body, divisor));
}

/// Receives the quotient limbs of a division.
pub(crate) trait QuotientSink {
    /// Records quotient limb `index`.
    fn put(&mut self, index: usize, limb: u64);
}

impl<L: ArrayLength> QuotientSink for GenericArray<u64, L> {
    #[inline(always)]
    fn put(&mut self, index: usize, limb: u64) {
        self.as_mut_slice().put(index, limb);
    }
}

impl QuotientSink for [u64] {
    #[inline(always)]
    fn put(&mut self, index: usize, limb: u64) {
        if let Some(slot) = self.get_mut(index) {
            *slot = limb;
        }
    }
}

/// Drops the quotient of a reduction.
pub(crate) struct DiscardQuotient;

impl QuotientSink for DiscardQuotient {
    #[inline(always)]
    fn put(&mut self, _index: usize, _limb: u64) {}
}
