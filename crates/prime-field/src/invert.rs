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
//! On one limb the variable-time inversion is the extended Euclidean algorithm on the limb, one
//! hardware division per step. The constant-time inversion there is a Fermat exponentiation
//! `x^(p - 2)` by an addition chain, cheaper than the batches; the chain depends on the exponent
//! alone, so it runs in constant time with respect to `x`.

use crate::field::{Fp, Modulus};
use core::hint::black_box;

use crate::chain::Program;
use crate::kernel::{add, equal, fits, is_zero, limb, mul_add, neg_mod, select, set, sub};
use crate::params::{ExtendedLimbs, Limbs, Shape};
use crate::width::Width;

/// The division steps of one batch.
const BATCH: usize = 62;

/// The mask of the low `BATCH` bits.
const LOW: u64 = (1 << BATCH) - 1;

impl<M: Modulus> Fp<M> {
    /// `self^-1`, or `None` for `0`.
    #[inline(always)]
    #[must_use]
    pub fn invert(self) -> Option<Self> {
        if Shape::<M>::LIMBS == 1 {
            return self.invert_euclid();
        }
        self.invert_with::<true>()
    }

    /// `self^-1`, or `None` for `0`, in time that depends on the modulus alone; the `Option` is
    /// built in [`declassify`].
    #[inline(always)]
    pub(crate) fn invert_ct(self) -> Option<Self> {
        if Shape::<M>::LIMBS == 1 {
            return declassify(
                !self.is_zero(),
                self.pow_program::<true>(&Shape::<M>::FERMAT_CHAIN),
            );
        }
        self.invert_with::<false>()
    }

    /// The inverse of a one-limb element by the extended Euclidean algorithm on its
    /// representation `v`: `v^-1`, times `R^3` by a Montgomery product for a Montgomery
    /// residue `v = x R`, which yields `x^-1 R`.
    #[inline(always)]
    fn invert_euclid(self) -> Option<Self> {
        let value = limb(self.limbs(), 0);
        if Shape::<M>::MERSENNE && value == Shape::<M>::P0 {
            return None;
        }
        let mut limbs = M::Width::ZERO;
        set(limbs.as_mut_slice(), 0, euclid(value, Shape::<M>::P0)?);
        let inverse = Self::from_limbs(limbs);
        Some(if Shape::<M>::MONTGOMERY {
            inverse.mul_with::<false>(Self::from_limbs(Shape::<M>::R3))
        } else {
            inverse
        })
    }

    #[inline(always)]
    fn invert_with<const VARTIME: bool>(self) -> Option<Self> {
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
        let masks = Masks::new();
        let batches = (45_907 * Shape::<M>::BITS + 26_313)
            .div_ceil(19_929)
            .div_ceil(BATCH);
        for _ in 0..batches {
            if VARTIME && is_zero(g.as_slice()) {
                break;
            }
            let matrix = Matrix::divsteps::<VARTIME>(
                masks,
                &mut zeta,
                limb(f.as_slice(), 0),
                limb(g.as_slice(), 0),
            );
            let next_f =
                combination_shifted::<M>(masks, matrix.u, f.as_slice(), matrix.v, g.as_slice());
            let next_g =
                combination_shifted::<M>(masks, matrix.q, f.as_slice(), matrix.r, g.as_slice());
            let next_d =
                combination_mod::<M>(masks, matrix.u, d.as_slice(), matrix.v, e.as_slice());
            let next_e =
                combination_mod::<M>(masks, matrix.q, d.as_slice(), matrix.r, e.as_slice());
            (f, g) = (next_f, next_g);
            (d, e) = (next_d, next_e);
        }
        // `f` is `±1` exactly when `y` is a unit; `-1` is all ones in two's complement.
        let mut one = M::Width::EXTENDED_ZERO;
        set(one.as_mut_slice(), 0, 1);
        let positive = equal(f.as_slice(), one.as_slice());
        let negative = f.as_slice().iter().fold(u64::MAX, |all, limb| all & limb) == u64::MAX;
        let mut negated = M::Width::ZERO;
        neg_mod(
            negated.as_mut_slice(),
            d.as_slice(),
            Shape::<M>::P.as_slice(),
        );
        let mut inverse = M::Width::ZERO;
        select(
            inverse.as_mut_slice(),
            negative,
            negated.as_slice(),
            d.as_slice(),
        );
        disclose::<VARTIME, _>(positive | negative, Self::from_limbs(inverse))
    }
}

/// `a^-1 mod p` by the extended Euclidean algorithm, for `a < p`; `None` when `a` and `p` share a
/// factor, `a = 0` among them. A modulus below `2^32` divides 32-bit limbs.
#[inline(always)]
fn euclid(a: u64, p: u64) -> Option<u64> {
    if p >> 32 == 0 {
        euclid32(a as u32, p as u32).map(u64::from)
    } else {
        euclid64(a, p)
    }
}

/// Generates the extended Euclidean algorithm on one integer type. The remainders `r_i` start
/// from `p` and `a`, with `r_i = (-1)^(i + 1) t_i a mod p` for coefficients `t_i >= 0` that grow
/// by `t_(i + 1) = t_(i - 1) + q t_i`; two steps run per pass, so that no value moves between
/// registers.
macro_rules! euclid {
    ($name:ident, $int:ty) => {
        /// `a^-1 mod p`; `None` when the greatest common divisor is not `1`.
        #[inline(always)]
        fn $name(a: $int, p: $int) -> Option<$int> {
            let (mut even, mut odd) = (p, a);
            let (mut even_t, mut odd_t): ($int, $int) = (0, 1);
            loop {
                if odd == 0 {
                    // The last remainder has an even index: its coefficient is negative.
                    return (even == 1).then(|| p - even_t);
                }
                let quotient = even / odd;
                even %= odd;
                even_t += quotient * odd_t;
                if even == 0 {
                    return (odd == 1).then_some(odd_t);
                }
                let quotient = odd / even;
                odd %= even;
                odd_t += quotient * even_t;
            }
        }
    };
}

euclid!(euclid32, u32);
euclid!(euclid64, u64);

/// `Some(inverse)` when `invertible`, else `None`; the constant-time inversion builds it in
/// [`declassify`].
#[inline(always)]
fn disclose<const VARTIME: bool, T>(invertible: bool, inverse: T) -> Option<T> {
    if VARTIME {
        invertible.then_some(inverse)
    } else {
        declassify(invertible, inverse)
    }
}

/// `Some(value)` when `some`, else `None`. The `Option` that [`Secret::invert`] and
/// [`Secret::sqrt`] return discloses `some`, so the branch on it is the one decision of those
/// constant-time operations that may depend on the value. The function keeps a symbol of its own,
/// by which the constant-time check of `ct-check` exempts that branch; it would exempt any other
/// code here too.
///
/// [`Secret::invert`]: crate::Secret::invert
/// [`Secret::sqrt`]: crate::Secret::sqrt
#[inline(never)]
pub(crate) fn declassify<T>(some: bool, value: T) -> Option<T> {
    if some { Some(value) } else { None }
}

/// The masks of one inversion, all ones or zero: a sign spread by a shift of 63 that passes through
/// [`black_box`] once per inversion. The compiler cannot tell that a mask takes only those two
/// values, so no masked step becomes a branch, at one shift per mask where
/// [`mask`](crate::kernel::mask) costs a barrier per mask.
#[derive(Clone, Copy)]
struct Masks {
    spread: u32,
}

impl Masks {
    #[inline(always)]
    fn new() -> Self {
        Self {
            spread: black_box(63),
        }
    }

    /// All ones when `value` is negative.
    #[inline(always)]
    fn negative(self, value: i64) -> u64 {
        value.wrapping_shr(self.spread) as u64
    }

    /// All ones when bit 0 of `value` is set.
    #[inline(always)]
    fn odd(self, value: u64) -> u64 {
        self.negative(value.wrapping_shl(self.spread) as i64)
    }

    /// All ones when `condition` holds.
    #[inline(always)]
    fn of(self, condition: bool) -> u64 {
        self.odd(u64::from(condition))
    }
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
    fn divsteps<const VARTIME: bool>(masks: Masks, zeta: &mut i64, f: u64, g: u64) -> Self {
        if VARTIME {
            Self::divsteps_vartime(zeta, f, g)
        } else {
            Self::divsteps_masked(masks, zeta, f, g)
        }
    }

    /// The steps with branches; the even steps of a run of trailing zeros of `g` run at once.
    #[inline(always)]
    fn divsteps_vartime(zeta: &mut i64, mut f: u64, mut g: u64) -> Self {
        let (mut u, mut v, mut q, mut r) = (1i64, 0i64, 0i64, 1i64);
        let mut remaining = BATCH as u32;
        while remaining > 0 {
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
            g >>= 1;
            u <<= 1;
            v <<= 1;
            remaining -= 1;
        }
        Self { u, v, q, r }
    }

    /// The steps with masked selections. The factors take the same steps in two lanes, `(u, v)`
    /// beside `(q, r)`, which the compiler may pair in vector registers.
    #[inline(always)]
    fn divsteps_masked(masks: Masks, zeta: &mut i64, mut f: u64, mut g: u64) -> Self {
        let mut top = [1i64, 0];
        let mut bottom = [0i64, 1];
        let mut remaining = BATCH;
        while remaining > 0 {
            let negate = masks.negative(*zeta);
            let odd = masks.odd(g);
            let x = (f ^ negate).wrapping_sub(negate);
            g = g.wrapping_add(x & odd);
            let swap = negate & odd;
            *zeta = (*zeta ^ swap as i64) - 1;
            f = f.wrapping_add(g & swap);
            for (high, low) in top.iter_mut().zip(bottom.iter_mut()) {
                let y = (*high ^ negate as i64).wrapping_sub(negate as i64);
                *low = low.wrapping_add(y & odd as i64);
                *high = high.wrapping_add(*low & swap as i64) << 1;
            }
            g >>= 1;
            remaining -= 1;
        }
        let [u, v] = top;
        let [q, r] = bottom;
        Self { u, v, q, r }
    }
}

/// `f a + g b` in two's complement over the length of `out`, for `a` and `b` in two's
/// complement; returns the mask of its sign. The result fits the length.
#[inline(always)]
fn combination(masks: Masks, out: &mut [u64], f: i64, a: &[u64], g: i64, b: &[u64]) -> u64 {
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
    masks.negative(out.last().map_or(0, |top| *top as i64))
}

/// `(f a + g b) / 2^62` in two's complement, the division exact.
#[inline(always)]
fn combination_shifted<M: Modulus>(
    masks: Masks,
    f: i64,
    a: &[u64],
    g: i64,
    b: &[u64],
) -> ExtendedLimbs<M> {
    let mut out = M::Width::EXTENDED_ZERO;
    let negative = combination(masks, out.as_mut_slice(), f, a, g, b);
    shr_in_place(out.as_mut_slice(), BATCH, negative);
    out
}

/// `(f u + g v) / 2^62 mod p` for `u, v < p`.
#[inline(always)]
fn combination_mod<M: Modulus>(masks: Masks, f: i64, u: &[u64], g: i64, v: &[u64]) -> Limbs<M> {
    let mut sum = M::Width::EXTENDED_ZERO;
    let negative = combination(masks, sum.as_mut_slice(), f, u, g, v);
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
    select_in_place(
        sum.as_mut_slice(),
        masks.of(borrow == 0),
        reduced.as_slice(),
    );
    let borrow = sub(
        reduced.as_mut_slice(),
        sum.as_slice(),
        Shape::<M>::P.as_slice(),
    );
    select_in_place(
        sum.as_mut_slice(),
        masks.of(borrow == 0),
        reduced.as_slice(),
    );
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

/// Two's complement negation under the mask `flip`.
#[inline(always)]
fn negate_masked(value: &mut [u64], flip: u64) {
    let mut carry = flip & 1;
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

/// `value = other` under the mask `take`.
#[inline(always)]
fn select_in_place(value: &mut [u64], take: u64, other: &[u64]) {
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
    const P_MINUS_TWO: Limbs<M> = {
        let mut out = M::Width::ZERO;
        fits(sub(out.as_mut_slice(), Self::P.as_slice(), &[2]));
        out
    };

    /// The addition chain of `x^(p - 2)`, for one-limb moduli.
    pub(crate) const FERMAT_CHAIN: Program<M> = if Self::LIMBS == 1 {
        Program::for_exponent(Self::P_MINUS_TWO.as_slice())
    } else {
        Program::EMPTY
    };
}
