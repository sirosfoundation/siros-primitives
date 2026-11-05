//! Inversion by the Bernstein–Yang safegcd (*Fast constant-time gcd computation and modular
//! inversion*, 2019), in batches of division steps as in libsecp256k1 and crypto-bigint.
//!
//! A division step reads the parity of `g` and the sign of `delta`, so a batch of up to 62 steps
//! runs on the low 64 bits of `f` and `g` alone. The steps of a batch compose into a 2×2 matrix
//! of signed factors scaled by `2^62`, which then updates the full `f` and `g` and the Bézout
//! coefficients `d` and `e` modulo `p` once per batch.
//!
//! The variable-time variant is libsecp256k1's `divsteps_62_var`: the original steps from
//! `delta = 1`, a run of trailing zeros of `g` at once, and the steps that add `f` to `g` at once
//! as one multiple of `f` that clears up to 6 low bits; it stops when `g` reaches zero. The
//! constant-time variant tracks `zeta = -(delta + 1/2)`, the half-delta variant whose proven
//! bound is `(45907 BITS + 26313) / 19929` steps, and runs that bound with masked selections in
//! the formulation of libsecp256k1's `divsteps_59`. Its batches are two halves of 30 steps, short
//! enough that a row of factors `(a, b)` packs into the one word `a + 2^32 b`.
//!
//! On one limb the variable-time inversion is the extended Euclidean algorithm on the limb, one
//! hardware division per step. The constant-time inversion there is a Fermat exponentiation
//! `x^(p - 2)` by an addition chain, cheaper than the batches; the chain depends on the exponent
//! alone, so it runs in constant time with respect to `x`.

use crate::field::{Fp, Modulus};
use core::hint::black_box;

use crate::chain::Program;
use crate::kernel::{add, fits, is_zero, limb, mul_add, neg_mod, select, set, sub};
use crate::params::{ExtendedLimbs, Limbs, Shape};
use crate::width::Width;

/// The division steps of one batch: its matrix scales by `2^BATCH`.
const BATCH: usize = 62;

/// The division steps of one half of a constant-time batch. Within `HALF` steps from the
/// identity, every factor stays in `[-2^HALF, 2^HALF]`, so a row of two factors fits one word.
const HALF: usize = 30;

/// The division steps of a constant-time batch: two halves, whose product takes the scale
/// `2^BATCH` by a shift.
const MASKED_BATCH: usize = 2 * HALF;

/// The mask of the low `BATCH` bits.
const LOW: u64 = (1 << BATCH) - 1;

impl<M: Modulus> biguint::Field for Fp<M> {
    // As a call, the P-256 base inverse measured 2.5% more cycles.
    #[inline(always)]
    fn invert_nonzero(self) -> Self {
        if Shape::<M>::LIMBS == 1 {
            return self.invert_euclid();
        }
        self.invert_with::<true>()
    }

    /// The time depends on the modulus alone.
    #[inline(always)]
    fn ct_invert_nonzero(self) -> Self {
        if Shape::<M>::LIMBS == 1 {
            return self.pow_program::<true>(&Shape::<M>::FERMAT_CHAIN);
        }
        self.invert_with::<false>()
    }
}

impl<M: Modulus> Fp<M> {
    /// The inverse of a nonzero one-limb element by the extended Euclidean algorithm on its
    /// representation `v`: `v^-1`, times `R^3` by a Montgomery product for a Montgomery
    /// residue `v = x R`, which yields `x^-1 R`.
    #[inline(always)]
    fn invert_euclid(self) -> Self {
        let mut limbs = M::Width::ZERO;
        set(
            limbs.as_mut_slice(),
            0,
            euclid(limb(self.limbs(), 0), Shape::<M>::P0),
        );
        let inverse = Self::from_limbs(limbs);
        if Shape::<M>::MONTGOMERY {
            inverse.mul_with::<false>(Self::from_limbs(Shape::<M>::R3))
        } else {
            inverse
        }
    }

    /// The inverse of a nonzero element.
    #[inline(always)]
    fn invert_with<const VARTIME: bool>(self) -> Self {
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
        // `eta = -delta` of the variable-time steps from `delta = 1`, or `zeta = -(delta + 1/2)`
        // of the masked steps from `delta = 1/2`.
        let mut eta: i64 = -1;
        let masks = Masks::new();
        let batches = if VARTIME {
            Shape::<M>::DIVSTEPS.div_ceil(BATCH)
        } else {
            Shape::<M>::HALF_DELTA_DIVSTEPS.div_ceil(MASKED_BATCH)
        };
        for _ in 0..batches {
            if VARTIME && is_zero(g.as_slice()) {
                break;
            }
            let matrix = Matrix::divsteps::<VARTIME>(
                masks,
                &mut eta,
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
        // `f` ends as `±1`: its top limb carries the sign.
        let negative = f.as_slice().last().is_some_and(|top| (*top as i64) < 0);
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
        Self::from_limbs(inverse)
    }
}

/// `a^-1 mod p` by the extended Euclidean algorithm, for `0 < a < p`. A modulus below `2^32`
/// divides 32-bit limbs.
#[inline(always)]
pub(crate) fn euclid(a: u64, p: u64) -> u64 {
    if p >> 32 == 0 {
        u64::from(euclid32(a as u32, p as u32))
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
        /// `a^-1 mod p`, for `a` coprime to `p`: the last nonzero remainder is `1`.
        #[inline(always)]
        fn $name(a: $int, p: $int) -> $int {
            let (mut even, mut odd) = (p, a);
            let (mut even_t, mut odd_t): ($int, $int) = (0, 1);
            loop {
                if odd == 0 {
                    // The last remainder has an even index: its coefficient is negative.
                    return p - even_t;
                }
                let quotient = even / odd;
                even %= odd;
                even_t += quotient * odd_t;
                if even == 0 {
                    return odd_t;
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
    /// The division steps of a batch on the low limbs of `f` and `g`: [`BATCH`] steps, or
    /// [`MASKED_BATCH`] masked steps. A step with `g` odd adds `f` to `g`, or `-f` when `eta`
    /// is negative, in which case `f` then takes the old `g`; every step halves `g`, which
    /// doubles the factors of `f`, and lowers `eta`.
    #[inline(always)]
    fn divsteps<const VARTIME: bool>(masks: Masks, eta: &mut i64, f: u64, g: u64) -> Self {
        if VARTIME {
            Self::divsteps_vartime(eta, f, g)
        } else {
            Self::divsteps_masked(masks, eta, f, g)
        }
    }

    /// The steps with branches, in the original formulation with `eta = -delta`, as in
    /// libsecp256k1's `divsteps_62_var`: a run of trailing zeros of `g` halves it at once, and
    /// the steps that add `f` to `g` before each halving run at once as `g + w f`, with `w` the
    /// multiple of `f` that clears up to 6 low bits of `g` after a swap and up to 4 otherwise,
    /// within the steps before `eta` changes sign.
    #[inline(always)]
    fn divsteps_vartime(eta: &mut i64, mut f: u64, mut g: u64) -> Self {
        let (mut u, mut v, mut q, mut r) = (1u64, 0u64, 0u64, 1u64);
        let mut remaining = BATCH as u32;
        loop {
            // The sentinel bit stops the count at the steps that remain.
            let zeros = (g | (u64::MAX << remaining)).trailing_zeros();
            g >>= zeros;
            u <<= zeros;
            v <<= zeros;
            *eta -= i64::from(zeros);
            remaining -= zeros;
            if remaining == 0 {
                break;
            }
            // `limit` lies in `[1, remaining]`: `eta >= 0` after a swap or in the other branch.
            let (w, limit) = if *eta < 0 {
                *eta = -*eta;
                (f, g) = (g, f.wrapping_neg());
                (u, q) = (q, u.wrapping_neg());
                (v, r) = (r, v.wrapping_neg());
                // `f (f^2 - 2) = -f^-1 mod 2^6` for odd `f`.
                let w = f
                    .wrapping_mul(g)
                    .wrapping_mul(f.wrapping_mul(f).wrapping_sub(2));
                (w, Self::limit(*eta, remaining).min(6))
            } else {
                // `f + ((f + 1) & 4) << 1 = f^-1 mod 2^4` for odd `f`.
                let inverse = f.wrapping_add((f.wrapping_add(1) & 4) << 1);
                let w = inverse.wrapping_neg().wrapping_mul(g);
                (w, Self::limit(*eta, remaining).min(4))
            };
            let w = w & (u64::MAX >> (64 - limit));
            g = g.wrapping_add(f.wrapping_mul(w));
            q = q.wrapping_add(u.wrapping_mul(w));
            r = r.wrapping_add(v.wrapping_mul(w));
        }
        Self {
            u: u as i64,
            v: v as i64,
            q: q as i64,
            r: r as i64,
        }
    }

    /// The low bits of `g` that one combination may clear, for `eta >= 0`: no more than the
    /// steps that remain, and no more than `eta + 1`, past which `eta` changes sign.
    #[inline(always)]
    fn limit(eta: i64, remaining: u32) -> u32 {
        (eta + 1).min(i64::from(remaining)) as u32
    }

    /// The steps with masked selections: two halves of [`HALF`] steps, the second from the `f`
    /// and `g` that the first leaves.
    #[inline(always)]
    fn divsteps_masked(masks: Masks, zeta: &mut i64, f: u64, g: u64) -> Self {
        let (first, f, g) = Self::half_masked(masks, zeta, f, g);
        let (second, _, _) = Self::half_masked(masks, zeta, f, g);
        second.after(&first)
    }

    /// [`HALF`] steps with masked selections; returns their matrix and the new `f` and `g`. The
    /// row `(a, b)` of the matrix is held as the word `a + 2^32 b`: negating, masking, adding
    /// and doubling the word does so to both factors, which stay within 31 bits.
    #[inline(always)]
    fn half_masked(masks: Masks, zeta: &mut i64, mut f: u64, mut g: u64) -> (Self, u64, u64) {
        // `u + 2^32 v` and `q + 2^32 r`, from the identity.
        let mut top = 1u64;
        let mut bottom = 1u64 << 32;
        for _ in 0..HALF {
            let negate = masks.negative(*zeta);
            let odd = masks.odd(g);
            g = g.wrapping_add((f ^ negate).wrapping_sub(negate) & odd);
            bottom = bottom.wrapping_add((top ^ negate).wrapping_sub(negate) & odd);
            let swap = negate & odd;
            *zeta = (*zeta ^ swap as i64) - 1;
            f = f.wrapping_add(g & swap);
            top = top.wrapping_add(bottom & swap) << 1;
            g >>= 1;
        }
        let (u, v) = unpack(top);
        let (q, r) = unpack(bottom);
        (Self { u, v, q, r }, f, g)
    }

    /// The matrix of `first`'s steps followed by `self`'s, `self · first`, scaled by
    /// `2^(BATCH - MASKED_BATCH)`. Its factors lie in `[-2^62, 2^62]`.
    #[inline(always)]
    fn after(&self, first: &Self) -> Self {
        let entry = |a: i64, b: i64, c: i64, d: i64| {
            a.wrapping_mul(b).wrapping_add(c.wrapping_mul(d)) << (BATCH - MASKED_BATCH)
        };
        Self {
            u: entry(self.u, first.u, self.v, first.q),
            v: entry(self.u, first.v, self.v, first.r),
            q: entry(self.q, first.u, self.r, first.q),
            r: entry(self.q, first.v, self.r, first.r),
        }
    }
}

/// The factors `(a, b)` of the word `a + 2^32 b`, for `a` and `b` in `[-2^31, 2^31)`.
#[inline(always)]
fn unpack(word: u64) -> (i64, i64) {
    let low = i64::from(word as u32 as i32);
    let high = (word.wrapping_sub(low as u64) as i64) >> 32;
    (low, high)
}

/// `f a + g b` in two's complement over the length of `out`, for `a` and `b` in two's
/// complement; returns the mask of its sign. The result fits the length.
#[inline(always)]
fn combination(masks: Masks, out: &mut [u64], f: i64, a: &[u64], g: i64, b: &[u64]) -> u64 {
    let mut carry: i128 = 0;
    for (index, slot) in out.iter_mut().enumerate() {
        let sum = carry
            + signed_product(f, limb(a, index))
            + signed_product(g, limb(b, index));
        *slot = sum as u64;
        carry = sum >> 64;
    }
    // The limbs of `a` and `b` were read as unsigned; their sign terms vanish modulo `2^(64
    // len)`, so the low limbs are right, and the sign is the top bit.
    masks.negative(out.last().map_or(0, |top| *top as i64))
}

/// `f a` for a signed `f` and an unsigned `a`.
#[cfg(not(all(target_arch = "wasm32", not(wide_arithmetic))))]
#[inline(always)]
fn signed_product(f: i64, a: u64) -> i128 {
    i128::from(f) * i128::from(a)
}

/// `f a` for a signed `f` and an unsigned `a`, from the unsigned product of the split
/// [`widening_mul`](crate::kernel::widening_mul): a negative `f` reads as `f + 2^64`, which adds
/// `a 2^64`.
#[cfg(all(target_arch = "wasm32", not(wide_arithmetic)))]
#[inline(always)]
fn signed_product(f: i64, a: u64) -> i128 {
    let product = crate::kernel::widening_mul(f as u64, a) as i128;
    product.wrapping_sub((i128::from(a) << 64) & i128::from(f >> 63))
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

    /// The division steps that bring `g` to zero from `delta = 1` (Bernstein and Yang, theorem
    /// 11.2), for `f = p` and `0 <= g < p`.
    const DIVSTEPS: usize = if Self::BITS < 46 {
        (49 * Self::BITS + 80) / 17
    } else {
        (49 * Self::BITS + 57) / 17
    };

    /// The division steps that bring `g` to zero from `delta = 1/2`, the proven bound of the
    /// half-delta variant.
    const HALF_DELTA_DIVSTEPS: usize = (45_907 * Self::BITS + 26_313).div_ceil(19_929);

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
