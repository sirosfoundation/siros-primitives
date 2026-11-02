//! Inversion by the Bernstein–Yang safegcd (*Fast constant-time gcd computation and modular
//! inversion*, 2019), in batches of 62 division steps as in libsecp256k1 and crypto-bigint.
//!
//! A division step reads the parity of `g` and the sign of `delta`, so 62 steps run on the low
//! 64 bits of `f` and `g` alone. The steps of a batch compose into a 2×2 matrix of 62-bit signed
//! factors, which then updates the full `f` and `g` and the Bézout coefficients `d` and `e`
//! modulo `p` once per batch. The steps track `zeta = -(delta + 1/2)`, the half-delta variant
//! whose proven bound is `(45907 BITS + 26313) / 19929` steps. The constant-time variant runs
//! that bound with masked selections, in the formulation of libsecp256k1's `divsteps_59`; the
//! variable-time variant skips the even steps of a batch at once by the trailing zeros of `g`
//! and stops when `g` reaches zero.
//!
//! For moduli of at most 64 bits a Fermat exponentiation `x^(p - 2)` is cheaper than the batches,
//! and the exponent is public, so it runs in constant time with respect to `x`.

use crate::field::{Fp, Modulus};
use crate::kernel::{add, equal, fits, is_zero, limb, mul_add, neg_mod, set, sub};
use crate::params::{ExtendedLimbs, Limbs, Shape};
use crate::width::Width;

/// The division steps of one batch.
const BATCH: usize = 62;

/// The mask of the low `BATCH` bits.
const LOW: u64 = (1 << BATCH) - 1;

/// The widest modulus whose inverse is a Fermat exponentiation.
const FERMAT_BITS: usize = 64;

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
        if Shape::<M>::BITS <= FERMAT_BITS {
            return if self.is_zero() {
                None
            } else {
                Some(self.pow_limbs(Shape::<M>::P_MINUS_TWO.as_slice()))
            };
        }
        // `f = d y` and `g = e y` modulo `p` throughout, with `f` odd; `d` and `e` carry the
        // representation's seed, so that `d` ends as the representation of `±y^-1`.
        let mut f = M::Width::EXTENDED_ZERO;
        f.as_mut_slice()
            .iter_mut()
            .zip(Shape::<M>::P.as_slice())
            .for_each(|(slot, &value)| *slot = value);
        let mut g = M::Width::EXTENDED_ZERO;
        g.as_mut_slice()
            .iter_mut()
            .zip(self.limbs())
            .for_each(|(slot, &value)| *slot = value);
        let mut d = M::Width::ZERO;
        let mut e = Shape::<M>::INVERSE_SEED;
        let mut zeta: i64 = -1;
        let batches = (45_907 * Shape::<M>::BITS + 26_313)
            .div_ceil(19_929)
            .div_ceil(BATCH);
        for _ in 0..batches {
            if VARTIME && is_zero(g.as_slice()) {
                break;
            }
            let matrix = Matrix::divsteps::<VARTIME>(
                &mut zeta,
                limb(f.as_slice(), 0),
                limb(g.as_slice(), 0),
            );
            let next_f = combination_shifted::<M>(matrix.u, f.as_slice(), matrix.v, g.as_slice());
            let next_g = combination_shifted::<M>(matrix.q, f.as_slice(), matrix.r, g.as_slice());
            let next_d = combination_mod::<M>(matrix.u, d.as_slice(), matrix.v, e.as_slice());
            let next_e = combination_mod::<M>(matrix.q, d.as_slice(), matrix.r, e.as_slice());
            (f, g) = (next_f, next_g);
            (d, e) = (next_d, next_e);
        }
        // `f` is `±1` exactly when `y` is a unit; `-1` is all ones in two's complement.
        let mut one = M::Width::EXTENDED_ZERO;
        set(one.as_mut_slice(), 0, 1);
        let positive = equal(f.as_slice(), one.as_slice());
        let negative = f.as_slice().iter().all(|limb| *limb == u64::MAX);
        let mut negated = M::Width::ZERO;
        neg_mod(
            negated.as_mut_slice(),
            d.as_slice(),
            Shape::<M>::P.as_slice(),
        );
        let inverse = Self::select(negative, Self::from_limbs(negated), Self::from_limbs(d));
        (positive | negative).then_some(inverse)
    }
}

/// All ones when `condition` holds, else zero.
#[inline(always)]
const fn mask(condition: bool) -> u64 {
    0u64.wrapping_sub(condition as u64)
}

/// The factors of one batch: the new `f` is `(u f + v g) / 2^62` and the new `g` is
/// `(q f + r g) / 2^62`, both exact.
struct Matrix {
    u: i64,
    v: i64,
    q: i64,
    r: i64,
}

impl Matrix {
    /// The 62 division steps of a batch on the low limbs of `f` and `g`. A step with `g` odd
    /// adds `f` to `g`, or `-f` when `zeta` is negative, in which case `f` then takes the old
    /// `g`; every step halves `g`, which doubles the factors of `f`, and lowers `zeta`.
    #[inline(always)]
    fn divsteps<const VARTIME: bool>(zeta: &mut i64, mut f: u64, mut g: u64) -> Self {
        let (mut u, mut v, mut q, mut r) = (1i64, 0i64, 0i64, 1i64);
        let mut remaining = BATCH as u32;
        while remaining > 0 {
            if VARTIME {
                // Every trailing zero of `g` is a step that only halves `g`.
                let zeros = g.trailing_zeros().min(remaining);
                g >>= zeros;
                u <<= zeros;
                v <<= zeros;
                *zeta -= i64::from(zeros);
                remaining -= zeros;
                if remaining == 0 {
                    break;
                }
                if *zeta < 0 {
                    *zeta = -*zeta - 2;
                    (f, g) = (g, g.wrapping_sub(f));
                    (u, v, q, r) = (q, r, q - u, r - v);
                } else {
                    *zeta -= 1;
                    g = g.wrapping_add(f);
                    q += u;
                    r += v;
                }
            } else {
                let negate = (*zeta >> 63) as u64;
                let odd = mask(g & 1 != 0);
                let x = (f ^ negate).wrapping_sub(negate);
                let y = (u ^ negate as i64).wrapping_sub(negate as i64);
                let z = (v ^ negate as i64).wrapping_sub(negate as i64);
                g = g.wrapping_add(x & odd);
                q = q.wrapping_add(y & odd as i64);
                r = r.wrapping_add(z & odd as i64);
                let swap = negate & odd;
                *zeta = (*zeta ^ swap as i64) - 1;
                f = f.wrapping_add(g & swap);
                u = u.wrapping_add(q & swap as i64);
                v = v.wrapping_add(r & swap as i64);
            }
            g >>= 1;
            u <<= 1;
            v <<= 1;
            remaining -= 1;
        }
        Self { u, v, q, r }
    }
}

/// `f a + g b` in two's complement over the length of `out`, for `a` and `b` in two's
/// complement; returns whether the result is negative. The result fits the length.
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
    // The limbs of `a` and `b` were read as unsigned; their sign terms vanish modulo `2^(64
    // len)`, so the low limbs are right, and the sign is the top bit.
    out.last().is_some_and(|top| (*top as i64) < 0)
}

/// `(f a + g b) / 2^62` in two's complement, the division exact.
#[inline(always)]
fn combination_shifted<M: Modulus>(f: i64, a: &[u64], g: i64, b: &[u64]) -> ExtendedLimbs<M> {
    let mut out = M::Width::EXTENDED_ZERO;
    let negative = combination(out.as_mut_slice(), f, a, g, b);
    shr_in_place(out.as_mut_slice(), BATCH, mask(negative));
    out
}

/// `(f u + g v) / 2^62 mod p` for `u, v < p`.
#[inline(always)]
fn combination_mod<M: Modulus>(f: i64, u: &[u64], g: i64, v: &[u64]) -> Limbs<M> {
    let mut sum = M::Width::EXTENDED_ZERO;
    let negative = combination(sum.as_mut_slice(), f, u, g, v);
    negate_masked(sum.as_mut_slice(), negative);
    // `|sum| < 2^63 p`. Adding `z p` with `z = -sum p^-1 mod 2^62` makes the sum divisible by
    // `2^62`; the quotient is below `3 p`.
    let z = limb(sum.as_slice(), 0).wrapping_mul(Shape::<M>::INV) & LOW;
    let mut carry = 0;
    for (index, slot) in sum.as_mut_slice().iter_mut().enumerate() {
        let (low, high) = mul_add(z, limb(Shape::<M>::P.as_slice(), index), *slot, carry);
        *slot = low;
        carry = high;
    }
    shr_in_place(sum.as_mut_slice(), BATCH, 0);
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

/// `value >>= shift` for `0 < shift < 64`, with `fill` (all ones or zero) entering at the top.
#[inline(always)]
fn shr_in_place(value: &mut [u64], shift: usize, fill: u64) {
    let len = value.len();
    for index in 0..len {
        let low = limb(value, index) >> shift;
        let above = if index + 1 < len {
            limb(value, index + 1)
        } else {
            fill
        };
        set(value, index, low | (above << (64 - shift)));
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
    /// The `e` that the inversion starts from: `R^2` in Montgomery form, so that the Bézout
    /// coefficient of `±1` comes out as the representation of the inverse; `1` for plain
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

    /// `p - 2`, the Fermat exponent.
    pub(crate) const P_MINUS_TWO: Limbs<M> = {
        let mut out = M::Width::ZERO;
        fits(sub(out.as_mut_slice(), Self::P.as_slice(), &[2]));
        out
    };
}
