//! Division by Knuth's Algorithm D (TAOCP vol. 2, 4.3.1) on base-2^32 digits.
//!
//! Each digit sits in a `u64`, so every intermediate product, remainder and quotient estimate
//! fits in one machine word.

use generic_array::sequence::GenericSequence;
use generic_array::{ArrayLength, GenericArray};

use crate::bits::Bits;
use crate::uint::Uint;

const DIGIT_BITS: usize = 32;
const DIGIT_BASE: u64 = 1 << DIGIT_BITS;
const DIGIT_MASK: u64 = DIGIT_BASE - 1;

type Digits<N> = GenericArray<u64, <N as crate::bits::sealed::Sealed>::Digits>;
type NormalizedDigits<N> = GenericArray<u64, <N as crate::bits::sealed::Sealed>::NormalizedDigits>;

impl<N: Bits> Uint<N> {
    /// `(self / divisor, self % divisor)`, or `None` when `divisor` is zero.
    #[inline(always)]
    #[must_use]
    pub fn div_rem(self, divisor: Self) -> Option<(Self, Self)> {
        if divisor.is_zero() {
            return None;
        }
        if self < divisor {
            return Some((Self::ZERO, self));
        }

        let dividend_digits = to_digits(self);
        let divisor_digits = to_digits(divisor);
        let divisor_len = significant_len(&divisor_digits);

        if divisor_len == 1 {
            let (quotient, remainder) =
                div_rem_single_digit::<N>(dividend_digits, digit(&divisor_digits, 0))?;
            return Some((from_digits(quotient), from_digits(remainder)));
        }

        let dividend_len = significant_len(&dividend_digits);
        let quotient_len = dividend_len.saturating_sub(divisor_len) + 1;
        let shift = digit(&divisor_digits, divisor_len.saturating_sub(1)).leading_zeros() as usize
            - (u64::BITS as usize - DIGIT_BITS);

        let divisor_norm: Digits<N> = normalize_digits(&divisor_digits, shift);
        let mut dividend_norm: NormalizedDigits<N> = normalize_digits(&dividend_digits, shift);
        let mut quotient = Digits::<N>::default();

        for offset in 0..quotient_len {
            let j = quotient_len - 1 - offset;
            let mut qhat = estimate_digit::<N>(&dividend_norm, &divisor_norm, j, divisor_len)?;
            let borrowed =
                subtract_scaled::<N>(&mut dividend_norm, &divisor_norm, j, divisor_len, qhat);
            if borrowed {
                qhat = qhat.saturating_sub(1);
                add_back::<N>(&mut dividend_norm, &divisor_norm, j, divisor_len);
            }
            set_digit(&mut quotient, j, qhat);
        }

        let remainder = unnormalize_remainder::<N>(&dividend_norm, shift);
        Some((from_digits(quotient), from_digits(remainder)))
    }
}

#[inline(always)]
fn to_digits<N: Bits>(value: Uint<N>) -> Digits<N> {
    let mut out = Digits::<N>::default();
    for (limb_index, limb) in value.limbs.iter().copied().enumerate() {
        set_digit(&mut out, limb_index * 2, limb & DIGIT_MASK);
        set_digit(&mut out, limb_index * 2 + 1, limb >> DIGIT_BITS);
    }
    out
}

#[inline(always)]
fn from_digits<N: Bits>(digits: Digits<N>) -> Uint<N> {
    Uint::from_generic_limbs(GenericArray::generate(|limb_index| {
        digit(&digits, limb_index * 2) | (digit(&digits, limb_index * 2 + 1) << DIGIT_BITS)
    }))
}

#[inline(always)]
fn digit<L: ArrayLength>(digits: &GenericArray<u64, L>, index: usize) -> u64 {
    digits.get(index).copied().unwrap_or(0)
}

#[inline(always)]
fn set_digit<L: ArrayLength>(digits: &mut GenericArray<u64, L>, index: usize, value: u64) {
    if let Some(slot) = digits.get_mut(index) {
        *slot = value & DIGIT_MASK;
    }
}

#[inline(always)]
fn significant_len<L: ArrayLength>(digits: &GenericArray<u64, L>) -> usize {
    digits
        .iter()
        .rposition(|digit| *digit != 0)
        .map_or(0, |index| index + 1)
}

/// `digits * 2^shift` in `Out` digits, for `shift < DIGIT_BITS`.
#[inline(always)]
fn normalize_digits<In, Out>(digits: &GenericArray<u64, In>, shift: usize) -> GenericArray<u64, Out>
where
    In: ArrayLength,
    Out: ArrayLength,
{
    let mut out = GenericArray::<u64, Out>::default();
    let mut carry = 0u64;
    for index in 0..out.len() {
        let current = digit(digits, index);
        let value = if shift == 0 {
            current
        } else {
            ((current << shift) & DIGIT_MASK) | carry
        };
        set_digit(&mut out, index, value);
        carry = if shift == 0 {
            0
        } else {
            current >> (DIGIT_BITS - shift)
        };
    }
    out
}

#[inline(always)]
fn unnormalize_remainder<N: Bits>(dividend_norm: &NormalizedDigits<N>, shift: usize) -> Digits<N> {
    let mut out = Digits::<N>::default();
    for index in 0..out.len() {
        let low = digit(dividend_norm, index);
        let high = digit(dividend_norm, index + 1);
        let value = if shift == 0 {
            low
        } else {
            (low >> shift) | ((high << (DIGIT_BITS - shift)) & DIGIT_MASK)
        };
        set_digit(&mut out, index, value);
    }
    out
}

/// Short division by one nonzero digit.
#[inline(always)]
fn div_rem_single_digit<N: Bits>(
    dividend: Digits<N>,
    divisor: u64,
) -> Option<(Digits<N>, Digits<N>)> {
    let mut quotient = Digits::<N>::default();
    let mut remainder = 0u64;
    for offset in 0..dividend.len() {
        let index = dividend.len() - 1 - offset;
        let current = (remainder << DIGIT_BITS) | digit(&dividend, index);
        let (digit_quotient, digit_remainder) = scalar_div_rem(current, divisor)?;
        set_digit(&mut quotient, index, digit_quotient);
        remainder = digit_remainder;
    }

    let mut remainder_digits = Digits::<N>::default();
    set_digit(&mut remainder_digits, 0, remainder);
    Some((quotient, remainder_digits))
}

/// Algorithm D step D3: the quotient digit estimate, at most one above the true digit.
#[inline(always)]
fn estimate_digit<N: Bits>(
    dividend_norm: &NormalizedDigits<N>,
    divisor_norm: &Digits<N>,
    j: usize,
    divisor_len: usize,
) -> Option<u64> {
    let top = digit(dividend_norm, j + divisor_len);
    let next = digit(dividend_norm, j + divisor_len - 1);
    let second = digit(dividend_norm, j + divisor_len - 2);
    let divisor_top = digit(divisor_norm, divisor_len - 1);
    let divisor_second = digit(divisor_norm, divisor_len - 2);
    let numerator = (top << DIGIT_BITS) | next;
    let (mut qhat, mut rhat) = scalar_div_rem(numerator, divisor_top)?;

    if qhat == DIGIT_BASE {
        qhat = qhat.saturating_sub(1);
        rhat += divisor_top;
    }

    while rhat < DIGIT_BASE && qhat * divisor_second > (rhat << DIGIT_BITS) + second {
        qhat = qhat.saturating_sub(1);
        rhat += divisor_top;
    }

    Some(qhat)
}

#[inline(always)]
fn scalar_div_rem(numerator: u64, divisor: u64) -> Option<(u64, u64)> {
    let quotient = numerator.checked_div(divisor)?;
    let remainder = numerator.wrapping_sub(quotient.wrapping_mul(divisor));
    Some((quotient, remainder))
}

/// Algorithm D step D4: subtracts `qhat * divisor` at digit `j`; `true` when it borrowed.
#[inline(always)]
fn subtract_scaled<N: Bits>(
    dividend_norm: &mut NormalizedDigits<N>,
    divisor_norm: &Digits<N>,
    j: usize,
    divisor_len: usize,
    qhat: u64,
) -> bool {
    let mut carry = 0u64;
    let mut borrow = 0u64;

    for i in 0..divisor_len {
        let product = qhat * digit(divisor_norm, i) + carry;
        carry = product >> DIGIT_BITS;
        let subtrahend = (product & DIGIT_MASK) + borrow;
        let current = digit(dividend_norm, j + i);
        if current >= subtrahend {
            set_digit(dividend_norm, j + i, current - subtrahend);
            borrow = 0;
        } else {
            set_digit(dividend_norm, j + i, current + DIGIT_BASE - subtrahend);
            borrow = 1;
        }
    }

    let top = digit(dividend_norm, j + divisor_len);
    let subtrahend = carry + borrow;
    if top >= subtrahend {
        set_digit(dividend_norm, j + divisor_len, top - subtrahend);
        false
    } else {
        set_digit(
            dividend_norm,
            j + divisor_len,
            top + DIGIT_BASE - subtrahend,
        );
        true
    }
}

/// Algorithm D step D6: adds the divisor back at digit `j` after an overestimated digit.
#[inline(always)]
fn add_back<N: Bits>(
    dividend_norm: &mut NormalizedDigits<N>,
    divisor_norm: &Digits<N>,
    j: usize,
    divisor_len: usize,
) {
    let mut carry = 0u64;
    for i in 0..divisor_len {
        let sum = digit(dividend_norm, j + i) + digit(divisor_norm, i) + carry;
        set_digit(dividend_norm, j + i, sum);
        carry = sum >> DIGIT_BITS;
    }

    let top = digit(dividend_norm, j + divisor_len) + carry;
    set_digit(dividend_norm, j + divisor_len, top);
}
