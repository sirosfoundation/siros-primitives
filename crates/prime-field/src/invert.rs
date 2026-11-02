//! Inversion by Pornin's optimized binary GCD (*Optimized Binary GCD for Modular Inversion*,
//! 2020, Algorithm 2).
//!
//! The extended binary GCD of `y` and `p` runs on 64-bit approximations of its two values for 31
//! steps at a time: the low 31 bits and the top 33 bits of each value decide the steps exactly.
//! The steps of a batch compose into a 2×2 matrix of 32-bit factors, which then updates the full
//! values and the two Bézout coefficients modulo `p` once per batch. The constant-time variant
//! runs the fixed `2 BITS - 1` steps with masked selections; the variable-time variant skips the
//! even steps of a batch at once and stops when the first value reaches zero.

use crate::field::{Fp, Modulus};
use crate::kernel::{add, equal, fits, is_zero, limb, mul_add, neg_mod, set, sub};
use crate::params::{ExtendedLimbs, Limbs, Shape};
use crate::width::Width;

/// The steps of one batch: 31 steps fit the 64-bit approximations with 33 exact top bits.
const BATCH: usize = 31;

impl<M: Modulus> Fp<M> {
    /// `self^-1`, or `None` for `0`, in time that depends on the modulus alone.
    #[inline]
    #[must_use]
    pub fn invert(self) -> Option<Self> {
        self.invert_with::<false>()
    }

    /// `self^-1`, or `None` for `0`, in time that depends on the value: the fastest inversion
    /// for public values.
    #[inline]
    #[must_use]
    pub fn invert_vartime(self) -> Option<Self> {
        self.invert_with::<true>()
    }

    #[inline(always)]
    fn invert_with<const VARTIME: bool>(self) -> Option<Self> {
        // `a = u y 2^-k` and `b = v y 2^-k` modulo `p` after `k` steps, with `u` and `v` scaled
        // by the representation's seed so that `v` ends as the representation of `y^-1`.
        let mut a = M::Width::EXTENDED_ZERO;
        a.as_mut_slice()
            .iter_mut()
            .zip(self.limbs())
            .for_each(|(slot, &value)| *slot = value);
        let mut b = M::Width::EXTENDED_ZERO;
        b.as_mut_slice()
            .iter_mut()
            .zip(Shape::<M>::P.as_slice())
            .for_each(|(slot, &value)| *slot = value);
        let mut u = Shape::<M>::INVERSE_SEED;
        let mut v = M::Width::ZERO;
        let batches = (2 * Shape::<M>::BITS - 1).div_ceil(BATCH);
        for _ in 0..batches {
            if VARTIME && is_zero(a.as_slice()) {
                break;
            }
            let (approximate_a, approximate_b) = approximate(a.as_slice(), b.as_slice());
            let matrix = Matrix::divsteps::<VARTIME>(approximate_a, approximate_b);
            let (next_a, negated_a) =
                combination_shifted::<M>(matrix.f0, a.as_slice(), matrix.g0, b.as_slice());
            let (next_b, negated_b) =
                combination_shifted::<M>(matrix.f1, a.as_slice(), matrix.g1, b.as_slice());
            // A negated `a` or `b` keeps `a = u y 2^-k` through the same negation of `u` or `v`.
            let next_u =
                combination_mod::<M>(matrix.f0, u.as_slice(), matrix.g0, v.as_slice(), negated_a);
            let next_v =
                combination_mod::<M>(matrix.f1, u.as_slice(), matrix.g1, v.as_slice(), negated_b);
            (a, b) = (next_a, next_b);
            (u, v) = (next_u, next_v);
        }
        let mut one = M::Width::EXTENDED_ZERO;
        set(one.as_mut_slice(), 0, 1);
        if equal(b.as_slice(), one.as_slice()) {
            Some(Self::from_limbs(v))
        } else {
            None
        }
    }
}

/// The 64-bit approximations of `a` and `b`: their low 31 bits and the top 33 bits at the bit
/// length of the larger one; the values themselves when both fit 64 bits. Every selection is
/// masked.
#[inline(always)]
fn approximate(a: &[u64], b: &[u64]) -> (u64, u64) {
    let len = bit_len_masked(a).max(bit_len_masked(b));
    let shift = len.saturating_sub(33);
    let small = mask(len <= 64);
    let top = |value: &[u64]| {
        let low = limb_masked(value, shift / 64);
        let high = limb_masked(value, shift / 64 + 1);
        (((u128::from(high) << 64) | u128::from(low)) >> (shift % 64)) as u64
    };
    let approximation = |value: &[u64]| {
        let exact = limb(value, 0);
        let folded = (exact & 0x7fff_ffff) | (top(value) << 31);
        (exact & small) | (folded & !small)
    };
    (approximation(a), approximation(b))
}

/// The bit length, by one pass over every limb.
#[inline(always)]
fn bit_len_masked(value: &[u64]) -> usize {
    value.iter().enumerate().fold(0, |len, (index, &limb)| {
        let here = index * 64 + 64 - limb.leading_zeros() as usize;
        let nonzero = mask(limb != 0) as usize;
        (here & nonzero) | (len & !nonzero)
    })
}

/// Limb `index`, by one pass over every limb; `0` beyond the length.
#[inline(always)]
fn limb_masked(value: &[u64], index: usize) -> u64 {
    value
        .iter()
        .enumerate()
        .fold(0, |found, (position, &limb)| {
            found | (limb & mask(position == index))
        })
}

/// All ones when `condition` holds, else zero.
#[inline(always)]
const fn mask(condition: bool) -> u64 {
    0u64.wrapping_sub(condition as u64)
}

/// The factors of one batch: the new `a` is `(f0 a + g0 b) / 2^31` and the new `b` is
/// `(f1 a + g1 b) / 2^31`, both exact.
struct Matrix {
    f0: i64,
    g0: i64,
    f1: i64,
    g1: i64,
}

impl Matrix {
    /// The 31 steps of a batch on the approximations: an odd `a` below `b` swaps the two, an odd
    /// `a` subtracts `b`, and `a` halves while the factors of `b` double.
    #[inline(always)]
    fn divsteps<const VARTIME: bool>(mut a: u64, mut b: u64) -> Self {
        let (mut f0, mut g0, mut f1, mut g1) = (1i64, 0i64, 0i64, 1i64);
        let mut remaining = BATCH as u32;
        while remaining > 0 {
            if VARTIME {
                // Every trailing zero of `a` is a step that only halves `a`.
                let zeros = a.trailing_zeros().min(remaining);
                a >>= zeros;
                f1 <<= zeros;
                g1 <<= zeros;
                remaining -= zeros;
                if remaining == 0 {
                    break;
                }
                if a < b {
                    (a, b) = (b, a);
                    (f0, g0, f1, g1) = (f1, g1, f0, g0);
                }
                a -= b;
                f0 -= f1;
                g0 -= g1;
            } else {
                let odd = mask(a & 1 != 0);
                let swap = odd & mask(a < b);
                let delta = (a ^ b) & swap;
                a ^= delta;
                b ^= delta;
                let delta = (f0 ^ f1) & swap as i64;
                f0 ^= delta;
                f1 ^= delta;
                let delta = (g0 ^ g1) & swap as i64;
                g0 ^= delta;
                g1 ^= delta;
                a = a.wrapping_sub(b & odd);
                f0 -= f1 & odd as i64;
                g0 -= g1 & odd as i64;
            }
            a >>= 1;
            f1 <<= 1;
            g1 <<= 1;
            remaining -= 1;
        }
        Self { f0, g0, f1, g1 }
    }
}

/// `f a + g b` in two's complement over the length of `out`; returns whether it is negative.
/// The magnitude fits the length.
#[inline(always)]
fn combination(out: &mut [u64], f: i64, a: &[u64], g: i64, b: &[u64]) -> bool {
    let mut carry: i128 = 0;
    for (index, slot) in out.iter_mut().enumerate() {
        let sum = carry
            + i128::from(f) * i128::from(limb(a, index))
            + i128::from(g) * i128::from(limb(b, index));
        *slot = sum as u64;
        carry = sum >> 64;
    }
    carry < 0
}

/// `|f a + g b| / 2^31`, the division exact, and whether the combination was negative.
#[inline(always)]
fn combination_shifted<M: Modulus>(
    f: i64,
    a: &[u64],
    g: i64,
    b: &[u64],
) -> (ExtendedLimbs<M>, bool) {
    let mut out = M::Width::EXTENDED_ZERO;
    let negative = combination(out.as_mut_slice(), f, a, g, b);
    negate_masked(out.as_mut_slice(), negative);
    shr_in_place(out.as_mut_slice(), BATCH);
    (out, negative)
}

/// `(f u + g v) / 2^31 mod p` for `u, v < p`, negated when `negate` holds.
#[inline(always)]
fn combination_mod<M: Modulus>(f: i64, u: &[u64], g: i64, v: &[u64], negate: bool) -> Limbs<M> {
    let mut sum = M::Width::EXTENDED_ZERO;
    let negative = combination(sum.as_mut_slice(), f, u, g, v) ^ negate;
    negate_masked(sum.as_mut_slice(), negative ^ negate);
    // `|sum| < 2^32 p`. Adding `z p` with `z = -sum p^-1 mod 2^31` makes the sum divisible by
    // `2^31`; the quotient is below `3 p`.
    let z = limb(sum.as_slice(), 0).wrapping_mul(Shape::<M>::INV) & ((1 << BATCH) - 1);
    let mut carry = 0;
    for (index, slot) in sum.as_mut_slice().iter_mut().enumerate() {
        let (low, high) = mul_add(z, limb(Shape::<M>::P.as_slice(), index), *slot, carry);
        *slot = low;
        carry = high;
    }
    shr_in_place(sum.as_mut_slice(), BATCH);
    let mut reduced = M::Width::EXTENDED_ZERO;
    let borrow = sub(
        reduced.as_mut_slice(),
        sum.as_slice(),
        Shape::<M>::DOUBLE_P.as_slice(),
    );
    select_in_place(sum.as_mut_slice(), borrow == 0, reduced.as_slice());
    let borrow = sub(
        reduced.as_mut_slice(),
        sum.as_slice(),
        Shape::<M>::P.as_slice(),
    );
    select_in_place(sum.as_mut_slice(), borrow == 0, reduced.as_slice());
    let mut out = M::Width::ZERO;
    let mut negated = M::Width::ZERO;
    out.as_mut_slice()
        .iter_mut()
        .zip(sum.as_slice())
        .for_each(|(slot, &value)| *slot = value);
    neg_mod(
        negated.as_mut_slice(),
        out.as_slice(),
        Shape::<M>::P.as_slice(),
    );
    select_in_place(out.as_mut_slice(), negative, negated.as_slice());
    out
}

/// Two's complement negation when `condition` holds.
#[inline(always)]
fn negate_masked(value: &mut [u64], condition: bool) {
    let flip = mask(condition);
    let mut carry = u64::from(condition);
    for slot in value {
        let (sum, carry_out) = (*slot ^ flip).overflowing_add(carry);
        *slot = sum;
        carry = u64::from(carry_out);
    }
}

/// `value >>= shift` for `shift < 64`.
#[inline(always)]
fn shr_in_place(value: &mut [u64], shift: usize) {
    for index in 0..value.len() {
        let low = limb(value, index) >> shift;
        let high = limb(value, index + 1) << (64 - shift);
        set(value, index, low | high);
    }
}

/// `value = other` when `condition` holds.
#[inline(always)]
fn select_in_place(value: &mut [u64], condition: bool, other: &[u64]) {
    let take = mask(condition);
    for (slot, &candidate) in value.iter_mut().zip(other) {
        *slot = (candidate & take) | (*slot & !take);
    }
}

impl<M: Modulus> Shape<M> {
    /// The `u` that the inversion starts from: `R^2` in Montgomery form, so that the Bézout
    /// coefficient of `1` comes out as the representation of the inverse; `1` for plain
    /// residues.
    pub(crate) const INVERSE_SEED: Limbs<M> = if Self::MONTGOMERY {
        Self::R2
    } else {
        Self::ONE
    };

    /// `2 p`, in `L + 1` limbs.
    pub(crate) const DOUBLE_P: ExtendedLimbs<M> = {
        let mut out = M::Width::EXTENDED_ZERO;
        fits(add(
            out.as_mut_slice(),
            Self::P.as_slice(),
            Self::P.as_slice(),
        ));
        out
    };
}
