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

pub(crate) mod ct;
pub(crate) mod dc;
pub(crate) mod reciprocal;

use core::num::NonZero;
use core::ops::{Add, Div, Rem};

use generic_array::{ArrayLength, GenericArray};
use typenum::Sum;

use crate::bits::Bits;
use crate::bits::sealed::Sealed;
use crate::kernel::{
    add_assign, compare, fits, shl_bits_into, shr_bits_into, split, split_mut, sub_mul_row,
    sub_mul_row_chains, subtract_if_above,
};
use crate::mul::WideningMul;
use crate::nonzero::{NonZeroUint, significant_len};
use crate::recursion::Halvings;
use crate::stream::{LimbSource, ShlBits, ZeroExtended, funnel_left, funnel_right};
use crate::uint::Uint;
use reciprocal::{NormalizedLimb, Reciprocal, Reciprocal3By2, join};

impl<N: Bits> Uint<N> {
    /// `(self / divisor, self % divisor)`. The divisor may be of any width; the remainder takes the
    /// divisor's.
    #[inline(always)]
    #[must_use]
    pub fn div_rem<M: Bits>(self, divisor: NonZeroUint<M>) -> (Self, Uint<M>) {
        let mut quotient = GenericArray::<u64, N::Limbs>::default();
        let remainder = divide::<N::Window, M>(&self.limbs, divisor, Top::Any, &mut quotient);
        (Uint::from_generic_limbs(quotient), remainder)
    }

    /// `(self / divisor, self % divisor)`, or `None` when `divisor` is zero. The divisor may be of
    /// any width; the remainder takes the divisor's.
    #[inline(always)]
    #[must_use]
    pub fn checked_div_rem<M: Bits>(self, divisor: Uint<M>) -> Option<(Self, Uint<M>)> {
        NonZeroUint::new(divisor).map(|divisor| self.div_rem(divisor))
    }

    /// `self / divisor`, or `None` when `divisor` is zero. The divisor may be of any width.
    #[inline(always)]
    #[must_use]
    pub fn checked_div<M: Bits>(self, divisor: Uint<M>) -> Option<Self> {
        NonZeroUint::new(divisor).map(|divisor| self / divisor)
    }

    /// `self % divisor` in the divisor's width, or `None` when `divisor` is zero.
    #[inline(always)]
    #[must_use]
    pub fn checked_rem<M: Bits>(self, divisor: Uint<M>) -> Option<Uint<M>> {
        NonZeroUint::new(divisor).map(|divisor| self % divisor)
    }

    /// `(self * rhs) mod modulus`. The operands may be of any width and any value.
    #[inline(always)]
    #[must_use]
    pub fn mul_mod<RhsN, ModN>(self, rhs: Uint<RhsN>, modulus: NonZeroUint<ModN>) -> Uint<ModN>
    where
        N: Add<RhsN>,
        RhsN: Bits,
        ModN: Bits,
        Sum<N, RhsN>: Bits,
    {
        // A factor below the modulus keeps the top of the product below it when the product has
        // room above the other factor for the modulus's limbs. The divide-and-conquer division
        // lays out its blocks at compile time; a quotient length chosen at run time doubles its
        // stack, so it divides the whole window.
        type Window<N, RhsN> = <Sum<N, RhsN> as Sealed>::Window;
        let room = <Sum<N, RhsN> as Bits>::LIMBS >= ModN::LIMBS + RhsN::LIMBS
            && !Shape::<Window<N, RhsN>, ModN::Limbs>::DIVIDE_AND_CONQUER;
        let top = if room && compare(&self.limbs, &modulus.get().limbs).is_lt() {
            Top::Below
        } else {
            Top::Any
        };
        let product = self.widening_mul(rhs);
        divide::<Window<N, RhsN>, ModN>(&product.limbs, modulus, top, &mut DiscardQuotient)
    }

    /// `(self * rhs) mod modulus`, or `None` when `modulus` is zero. The operands may be of any
    /// width and any value.
    #[inline(always)]
    #[must_use]
    pub fn checked_mul_mod<RhsN, ModN>(
        self,
        rhs: Uint<RhsN>,
        modulus: Uint<ModN>,
    ) -> Option<Uint<ModN>>
    where
        N: Add<RhsN>,
        RhsN: Bits,
        ModN: Bits,
        Sum<N, RhsN>: Bits,
    {
        NonZeroUint::new(modulus).map(|modulus| self.mul_mod(rhs, modulus))
    }
}

/// What a division knows of the top of its dividend.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum Top {
    /// Nothing.
    Any,
    /// The top `len` limbs of the dividend, for a divisor of `len` significant limbs, are below
    /// the divisor, as in a product of a factor below the divisor by a factor of at most
    /// `dividend.len() - len` limbs: the top quotient limb is zero, and the division skips it.
    Below,
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
        divide::<N::Window, M>(&self.limbs, divisor, Top::Any, &mut DiscardQuotient)
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
    top: Top,
    quotient: &mut (impl QuotientSink + ?Sized),
) -> Uint<M> {
    if Native::<W, M::Limbs>::FITS
        && let Some(divisor) = divisor.native()
    {
        return native(dividend, divisor, quotient);
    }
    let limbs = divisor.limbs();
    long::<_, W, M>(dividend, limbs, significant_len(limbs), top, quotient)
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
/// one nonzero.
#[inline(always)]
fn long<L: ArrayLength, W: ArrayLength, M: Bits>(
    dividend: &[u64],
    divisor: &GenericArray<u64, L>,
    len: usize,
    top: Top,
    quotient: &mut (impl QuotientSink + ?Sized),
) -> Uint<M> {
    Invariant::new(divisor, len).remainder::<W, M>(dividend, top, quotient)
}

/// A divisor prepared for division: shifted left until the top bit of its top significant limb is
/// set, with the reciprocal of its leading limbs. A division prepares its divisor; a modulus keeps
/// it. The dividend shifts with the divisor and the remainder shifts back; a divisor whose top bit
/// is set runs a copy of each path without shifts. Divisors of one or two significant limbs divide
/// in registers.
pub(crate) enum Invariant<L: ArrayLength> {
    /// One significant limb, with its leading zeros.
    Limb { divisor: Reciprocal, shift: u32 },
    /// Two significant limbs, with the leading zeros of the top one.
    TwoLimbs { divisor: Reciprocal3By2, shift: u32 },
    /// Three significant limbs and more.
    Limbs {
        /// The divisor shifted left by `shift`.
        normalized: GenericArray<u64, L>,
        /// The limbs up to its top significant one.
        len: usize,
        /// The leading zeros of its top significant limb.
        shift: u32,
        /// The reciprocal of its top two limbs.
        leading: Reciprocal3By2,
    },
}

impl<L: ArrayLength> Invariant<L> {
    /// The divisor whose low `len >= 1` limbs hold its value, the top one nonzero. An unoptimized
    /// build calls it, as it does the product kernels.
    #[cfg_attr(debug_assertions, inline(never))]
    #[cfg_attr(not(debug_assertions), inline(always))]
    pub(crate) fn new(divisor: &GenericArray<u64, L>, len: usize) -> Self {
        let top = divisor.get(len.wrapping_sub(1)).copied().unwrap_or(0);
        if top.leading_zeros() == 0 {
            Self::prepare(divisor, len, top, 0)
        } else {
            Self::prepare(divisor, len, top, top.leading_zeros())
        }
    }

    /// [`Self::new`] for the top significant limb `top` and its leading zeros `shift`.
    #[inline(always)]
    fn prepare(divisor: &GenericArray<u64, L>, len: usize, top: u64, shift: u32) -> Self {
        let limb = |index: usize| divisor.get(index).copied().unwrap_or(0);
        let next = limb(len.wrapping_sub(2));
        // The bits shifted out of the limb below the top one fill the top one.
        let high = NormalizedLimb::top_set(funnel_left(top, next, shift));
        match len {
            1 => Self::Limb {
                divisor: Reciprocal::new(high),
                shift,
            },
            2 => Self::TwoLimbs {
                divisor: Reciprocal3By2::new(high, next << shift),
                shift,
            },
            _ => {
                let mut invariant = Self::Limbs {
                    normalized: GenericArray::default(),
                    len,
                    shift,
                    leading: Reciprocal3By2::new(
                        high,
                        funnel_left(next, limb(len.wrapping_sub(3)), shift),
                    ),
                };
                // The divisor shifts into the invariant's own limbs.
                if let Self::Limbs { normalized, .. } = &mut invariant {
                    shl_bits_into(normalized, divisor, shift);
                }
                invariant
            }
        }
    }

    /// `dividend mod divisor` in `M` bits. The quotient limbs go to `quotient`. `W` holds the
    /// dividend's limbs and one more. An unoptimized build calls it, as it does the product
    /// kernels.
    #[cfg_attr(debug_assertions, inline(never))]
    #[cfg_attr(not(debug_assertions), inline(always))]
    pub(crate) fn remainder<W: ArrayLength, M: Bits>(
        &self,
        dividend: &[u64],
        top: Top,
        quotient: &mut (impl QuotientSink + ?Sized),
    ) -> Uint<M> {
        match self {
            Self::Limb { divisor, shift } => Uint::from_u64(if *shift == 0 {
                divide_by_limb(dividend, 0, *divisor, top, quotient)
            } else {
                divide_by_limb(dividend, *shift, *divisor, top, quotient) >> *shift
            }),
            Self::TwoLimbs { divisor, shift } => {
                if *shift == 0 {
                    Uint::from_u128(divide_by_two_limbs(dividend, 0, *divisor, top, quotient))
                } else {
                    let remainder = divide_by_two_limbs(dividend, *shift, *divisor, top, quotient);
                    let (low, high) = (remainder as u64, (remainder >> 64) as u64);
                    Uint::from_u128(join(high >> *shift, funnel_right(low, high, *shift)))
                }
            }
            Self::Limbs {
                normalized,
                len,
                shift,
                leading,
            } => by_limbs::<L, W, M>(dividend, normalized, *len, *shift, *leading, top, quotient),
        }
    }
}

/// [`Invariant::remainder`] by the divisor `normalized`, of three significant limbs and more.
#[inline(always)]
fn by_limbs<L: ArrayLength, W: ArrayLength, M: Bits>(
    dividend: &[u64],
    normalized: &GenericArray<u64, L>,
    len: usize,
    shift: u32,
    leading: Reciprocal3By2,
    top: Top,
    quotient: &mut (impl QuotientSink + ?Sized),
) -> Uint<M> {
    let mut window = if shift == 0 {
        shifted_window::<W>(dividend, 0)
    } else {
        shifted_window::<W>(dividend, shift)
    };
    let span = match top {
        Top::Any => window.len(),
        // The limbs above the dividend's are zero and the top `len` limbs are below the divisor:
        // the division starts one quotient limb lower.
        Top::Below => {
            quotient.put(dividend.len().wrapping_sub(len), 0);
            dividend.len()
        }
    };
    let (part, _) = split_mut(&mut window, span);
    if len == L::USIZE {
        // A divisor that fills its limbs passes its length as a constant, so the compiler unrolls
        // the division as it does the multiplication.
        if Shape::<W, L>::DIVIDE_AND_CONQUER {
            divide_and_conquer::<M>(part, normalized.as_slice(), leading, quotient);
        } else {
            divide_by_limbs::<true>(part, normalized.as_slice(), leading, quotient);
        }
    } else {
        let (significant, _) = split(normalized, len);
        divide_by_limbs::<false>(part, significant, leading, quotient);
    }
    let mut remainder = GenericArray::<u64, M::Limbs>::default();
    if shift == 0 {
        shr_bits_into(&mut remainder, &window, 0);
    } else {
        shr_bits_into(&mut remainder, &window, shift);
    }
    Uint::from_generic_limbs(remainder)
}

/// [`dc::divide_window`] with its scratch for a remainder of `M` bits. An unoptimized build calls
/// it, as it does the product kernels, so the scratch takes stack only when the division runs.
#[cfg_attr(debug_assertions, inline(never))]
#[cfg_attr(not(debug_assertions), inline(always))]
fn divide_and_conquer<M: Bits>(
    window: &mut [u64],
    divisor: &[u64],
    leading: Reciprocal3By2,
    quotient: &mut (impl QuotientSink + ?Sized),
) {
    let mut scratch = GenericArray::<u64, M::DivisionScratch>::default();
    dc::divide_window::<Halvings>(window, divisor, leading, quotient, &mut scratch);
}

/// The dividend shifted left by `shift` in a window of `W` limbs: the bits shifted out of its top
/// limb fill the window's limb above it. An unoptimized build calls it, so its copies take stack
/// only while it runs.
#[cfg_attr(debug_assertions, inline(never))]
#[cfg_attr(not(debug_assertions), inline(always))]
fn shifted_window<W: ArrayLength>(dividend: &[u64], shift: u32) -> GenericArray<u64, W> {
    let mut window = GenericArray::<u64, W>::default();
    shl_bits_into(&mut window, dividend, shift);
    window
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
    let shifted = ShlBits::new(ZeroExtended::new(dividend), shift);
    let spill = shifted.limb(dividend.len());
    // A range of indices gives the loops a trip count that the compiler computes.
    let limbs = (0..dividend.len())
        .rev()
        .map(move |index| shifted.limb(index));
    (spill, limbs)
}

/// `dividend * 2^shift mod d` for a one-limb `d`. The quotient limbs go to `quotient`.
#[inline(always)]
fn divide_by_limb(
    dividend: &[u64],
    shift: u32,
    divisor: Reciprocal,
    top: Top,
    quotient: &mut (impl QuotientSink + ?Sized),
) -> u64 {
    let (spill, mut limbs) = normalized_from_top(dividend, shift);
    // The spill holds only the bits that the normalization shifted out, so it stays below `d`.
    // Below, the spill is zero and the top limb is below `d`: it starts the remainder.
    let (first, count) = match top {
        Top::Any => (spill, dividend.len()),
        Top::Below => {
            quotient.put(dividend.len().wrapping_sub(1), 0);
            (limbs.next().unwrap_or(0), dividend.len().saturating_sub(1))
        }
    };
    (0..count)
        .rev()
        .zip(limbs)
        .fold(first, |remainder, (index, limb)| {
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
    top: Top,
    quotient: &mut (impl QuotientSink + ?Sized),
) -> u128 {
    let (spill, mut limbs) = normalized_from_top(dividend, shift);
    // The spill holds only the bits that the normalization shifted out, so the two leading limbs
    // stay below `d`. Below, the spill is zero and the top two limbs are below `d`: they start
    // the remainder.
    let high = limbs.next().unwrap_or(0);
    let (leading, count) = match top {
        Top::Any => (join(spill, high), dividend.len().saturating_sub(1)),
        Top::Below => {
            quotient.put(dividend.len().wrapping_sub(2), 0);
            (
                join(high, limbs.next().unwrap_or(0)),
                dividend.len().saturating_sub(2),
            )
        }
    };
    (0..count)
        .rev()
        .zip(limbs)
        .fold(leading, |remainder, (index, limb)| {
            let (digit, remainder) = divisor.div_rem(remainder, limb);
            quotient.put(index, digit);
            remainder
        })
}

/// Divides `numerator` by the normalized `divisor`, whose two leading limbs `leading` divides by:
/// writes the low `numerator.len() - divisor.len()` quotient limbs to `quotient`, leaves the
/// remainder in the low `divisor.len()` limbs of `numerator` and zeros above them, and returns
/// the top quotient limb, `0` or `1`.
#[cfg_attr(debug_assertions, inline(never))]
#[cfg_attr(not(debug_assertions), inline(always))]
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
    divide_by_limbs::<false>(numerator, divisor, leading, quotient);
    u64::from(above)
}

/// Leaves `window mod divisor` in the low `divisor.len()` limbs of `window` and zeros above them.
/// The window shrinks from the top by one limb per quotient limb. The top `divisor.len()` limbs of
/// `window` are below `divisor`. `CONSTANT` tells that the compiler knows the divisor's length,
/// which unrolls the subtraction of each scaled divisor into two carry chains
/// ([`sub_mul_row_chains`]). An unoptimized build calls it, as it does the product kernels.
#[cfg_attr(debug_assertions, inline(never))]
#[cfg_attr(not(debug_assertions), inline(always))]
pub(crate) fn divide_by_limbs<const CONSTANT: bool>(
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
        quotient.put(index, quotient_limb::<CONSTANT>(part, divisor, leading));
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
fn quotient_limb<const CONSTANT: bool>(
    part: &mut [u64],
    divisor: &[u64],
    leading: Reciprocal3By2,
) -> u64 {
    let Some((top, body)) = part.split_last_mut() else {
        return 0;
    };
    let [.., next, high] = body else {
        return 0;
    };
    let leading_part = join(*top, *high);
    let next = *next;

    if leading_part == leading.divisor() {
        return full_quotient_limb::<CONSTANT>(top, body, divisor);
    }

    let (digit, remainder) = leading.div_rem(leading_part, next);
    let ([low @ .., remainder_low, remainder_high], [divisor_low @ .., _, _]) =
        (&mut *body, divisor)
    else {
        return digit;
    };
    let (remainder, borrowed) = remainder.overflowing_sub(u128::from(
        scaled_subtraction::<CONSTANT>(low, divisor_low, digit),
    ));
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
fn full_quotient_limb<const CONSTANT: bool>(
    top: &mut u64,
    body: &mut [u64],
    divisor: &[u64],
) -> u64 {
    *top = top.wrapping_sub(scaled_subtraction::<CONSTANT>(body, divisor, u64::MAX));
    u64::MAX
}

/// Subtracts `divisor * digit` from `out` and returns the limb owed above it: in two carry chains
/// when the compiler knows the divisor's length, in one otherwise.
#[inline(always)]
fn scaled_subtraction<const CONSTANT: bool>(out: &mut [u64], divisor: &[u64], digit: u64) -> u64 {
    if CONSTANT {
        sub_mul_row_chains(out, divisor, digit)
    } else {
        sub_mul_row(out, divisor, digit)
    }
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
