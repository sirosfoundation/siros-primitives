//! The product of an element by an integer of any width, and the representation of such an
//! integer.
//!
//! An element holds a residue `r`: the integer, or its Montgomery form `x R mod p`. Both forms
//! scale alike, `(x R) k = (x k) R`, so `r k mod p` is the representation of `x k`, and
//! `1 k mod p` (or `R k mod p`) the representation of `k`.
//!
//! The product runs by Horner's rule over the limbs of `k` from the top: each step multiplies `r`
//! by one limb into `L + 1` limbs and reduces that value, below `p 2^64`, by one quotient limb.
//! The reduction depends on the form of `p`:
//!
//! - a pseudo-Mersenne `2^n - c` folds the part above `2^n` back times `c`;
//! - Goldilocks `2^64 - 2^32 + 1` folds by `2^64 = 2^32 - 1` and `2^96 = -1` (Plonky3's
//!   `reduce128`);
//! - a one-limb Mersenne `2^n - 1` sums the `n`-bit chunks;
//! - every other modulus takes the quotient limb from the top 128 bits of the value times a
//!   precomputed 129-bit reciprocal of `p` (Barrett, CRYPTO 1986; the one-limb quotient steps of
//!   division by invariant integers, Granlund and Montgomery, PLDI 1994, and Möller and
//!   Granlund, IEEE TC 2011). The estimate falls at most one below the quotient, so the
//!   remainder lies below `2 p` and one conditional subtraction ends the step. That subtraction
//!   is rare: the variable-time form branches on it.

use crate::field::Modulus;
use crate::kernel::{add_mod, below, fits, limb, set, shifted_limb, sub};
use crate::params::{ExtendedLimbs, Limbs, Shape, folds_from};
use crate::unrolled::{mac, sbb, select, unroll};
use crate::width::Width;

impl<M: Modulus> Shape<M> {
    /// `a k mod p` for the representation `a` and the integer `k` in little-endian limbs: the
    /// representation of the element times `k`. The variable-time form skips the zero limbs at
    /// the top of `k`.
    #[inline(always)]
    pub(crate) fn scale<const CT: bool>(a: &Limbs<M>, k: &[u64]) -> Limbs<M> {
        let residue = if k.len() <= 1 {
            Self::reduce_wide::<CT>(Self::row(a, limb(k, 0)))
        } else {
            Self::horner::<CT>(a, k)
        };
        Self::represent_residue(residue)
    }

    /// The representation of the integer `k`: `R k mod p`, or `k mod p` for a field of plain
    /// residues.
    #[inline(always)]
    pub(crate) fn integer<const CT: bool>(k: &[u64]) -> Limbs<M> {
        Self::scale::<CT>(&Self::ONE, k)
    }

    /// [`Self::scale`] for `k` of several limbs, as a residue below `p`.
    #[inline]
    fn horner<const CT: bool>(a: &Limbs<M>, k: &[u64]) -> Limbs<M> {
        let mut top = k.len() - 1;
        if !CT {
            while top > 0 && limb(k, top) == 0 {
                top -= 1;
            }
        }
        let mut acc = Self::reduce_wide::<CT>(Self::row(a, limb(k, top)));
        while top > 0 {
            top -= 1;
            let (low, high) = Self::row(a, limb(k, top));
            acc = Self::reduce_wide::<CT>((low, Self::carry_in(&acc, &high)));
        }
        acc
    }

    /// `a v` as `(low, high)`: its low limb and the `L` limbs above it. `high < p` for `a <= p`.
    #[inline(always)]
    fn row(a: &Limbs<M>, v: u64) -> (u64, Limbs<M>) {
        let mut high = M::Width::ZERO;
        let mut low = 0;
        let mut carry = 0;
        unroll!(j in 0, Self::LIMBS => {
            let (product, next) = mac(limb(a, j), v, carry, 0);
            if j == 0 {
                low = product;
            } else {
                set(high.as_mut_slice(), j - 1, product);
            }
            carry = next;
        });
        set(high.as_mut_slice(), Self::LIMBS - 1, carry);
        (low, high)
    }

    /// The high part of `acc 2^64 + h 2^64 + low` for residues `acc` and `h`, as the step's
    /// reduction takes it: `acc + h mod p`, which keeps the value below `p 2^64`, or the plain sum
    /// where the Mersenne chunks absorb it.
    #[inline(always)]
    fn carry_in(acc: &Limbs<M>, high: &Limbs<M>) -> Limbs<M> {
        if Self::LIMBS == 1 {
            let (a, h) = (limb(acc, 0), limb(high, 0));
            let sum = if Self::MERSENNE_CHUNKED {
                a + h
            } else {
                Self::add_residues1(a, h)
            };
            Self::one_limb(sum)
        } else {
            Self::add(acc, high)
        }
    }

    /// `high 2^64 + low mod p` for `high < 2 p` on a Mersenne modulus whose chunks sum it, and
    /// `high < p` on every other modulus.
    #[inline(always)]
    fn reduce_wide<const CT: bool>((low, high): (u64, Limbs<M>)) -> Limbs<M> {
        if Self::LIMBS == 1 {
            let value = (u128::from(limb(&high, 0)) << 64) | u128::from(low);
            let residue = if Self::MERSENNE_CHUNKED {
                Self::mersenne_chunks(value)
            } else if Self::SHIFT_REDC {
                Self::goldilocks_reduce(value)
            } else {
                Self::barrett1::<CT>(value)
            };
            Self::one_limb(residue)
        } else if Self::PSEUDO_MERSENNE {
            Self::fold_wide::<CT>(low, &high)
        } else {
            Self::barrett::<CT>(low, &high)
        }
    }

    /// The representation of a residue below `p`: `p` for `0` on a Mersenne modulus.
    #[inline(always)]
    fn represent_residue(residue: Limbs<M>) -> Limbs<M> {
        if Self::MERSENNE {
            let value = limb(&residue, 0);
            Self::one_limb(select(value == 0, Self::P0, value))
        } else {
            residue
        }
    }

    /// The limbs whose low limb is `value`.
    #[inline(always)]
    fn one_limb(value: u64) -> Limbs<M> {
        let mut out = M::Width::ZERO;
        set(out.as_mut_slice(), 0, value);
        out
    }

    // The quotient-limb estimate.

    /// The low 128 bits of `floor(2^(n + 128) / p)`, whose bit 128 is set, as two limbs: the
    /// long division of `2^n mod p = 2^n - p` through 128 more bits.
    const RECIPROCAL: [u64; 2] = {
        let mut power = M::Width::EXTENDED_ZERO;
        set(power.as_mut_slice(), Self::BITS / 64, 1 << (Self::BITS % 64));
        let mut remainder = M::Width::ZERO;
        // The borrow out of the limbs cancels the bit of `2^n` above them when `n = 64 L`.
        fits(sub(remainder.as_mut_slice(), power.as_slice(), Self::P.as_slice()));
        let mut quotient = [0u64; 2];
        let mut index = 128;
        while index > 0 {
            index -= 1;
            // `2 rem >= p` exactly when `rem >= p - rem`.
            let mut complement = M::Width::ZERO;
            fits(sub(
                complement.as_mut_slice(),
                Self::P.as_slice(),
                remainder.as_slice(),
            ));
            if !below(remainder.as_slice(), complement.as_slice()) {
                let bits = limb(&quotient, index / 64) | 1 << (index % 64);
                set(&mut quotient, index / 64, bits);
            }
            let mut doubled = M::Width::ZERO;
            add_mod(
                doubled.as_mut_slice(),
                remainder.as_slice(),
                remainder.as_slice(),
                Self::P.as_slice(),
                Self::SPARE,
            );
            remainder = doubled;
        }
        quotient
    };

    /// The quotient limb of the value whose bits from `n - 64` up are `y = y1 2^64 + y0`, at
    /// most one below `floor(value / p)`: `floor(y floor(2^(n + 128) / p) / 2^192)` without the
    /// low partial products. The value lies below `p 2^64`, so the quotient fits a limb.
    #[inline(always)]
    fn quotient(y0: u64, y1: u64) -> u64 {
        let [m0, m1] = Self::RECIPROCAL;
        let high = |a: u64, b: u64| (u128::from(a) * u128::from(b)) >> 64;
        let sum = u128::from(y0) + u128::from(y1) * u128::from(m1) + high(y1, m0) + high(y0, m1);
        y1 + (sum >> 64) as u64
    }

    /// `high 2^64 + low mod p` for `high < p` by one quotient limb, on several limbs.
    #[inline(always)]
    fn barrett<const CT: bool>(low: u64, high: &Limbs<M>) -> Limbs<M> {
        let value = |index: usize| {
            if index == 0 {
                low
            } else {
                limb(high, index - 1)
            }
        };
        // The bits from `n - 64` up: limbs `s / 64` and the one above, shifted by `s % 64`, of
        // the `L + 1` limbs of the value.
        let (y0, y1) = if Self::BITS >= 64 {
            let mut limbs = M::Width::EXTENDED_ZERO;
            unroll!(j in 0, Self::LIMBS + 1 => {
                set(limbs.as_mut_slice(), j, value(j));
            });
            let shift = Self::BITS - 64;
            (
                shifted_limb(limbs.as_slice(), shift, 0),
                shifted_limb(limbs.as_slice(), shift, 1),
            )
        } else {
            let y = ((u128::from(value(1)) << 64) | u128::from(low)) << (64 - Self::BITS);
            (y as u64, (y >> 64) as u64)
        };
        let q = Self::quotient(y0, y1);
        // `value - q p`, below `2 p`, on `L` limbs and the top limb.
        let mut out = M::Width::ZERO;
        let mut carry = 0;
        let mut borrow = 0;
        unroll!(j in 0, Self::LIMBS => {
            let (product, next) = mac(q, limb(Self::P.as_slice(), j), carry, 0);
            let (difference, borrow_out) = sbb(borrow, value(j), product);
            set(out.as_mut_slice(), j, difference);
            carry = next;
            borrow = borrow_out;
        });
        let top = if Self::SPARE {
            0
        } else {
            sbb(borrow, value(Self::LIMBS), carry).0 as u8
        };
        if CT {
            Self::reduce_select(out, top)
        } else {
            Self::reduce_branch(out, top)
        }
    }

    /// `value mod p` for `value < p 2^64`, by one quotient limb, on one limb.
    #[inline(always)]
    fn barrett1<const CT: bool>(value: u128) -> u64 {
        let y = value << (64 - Self::BITS);
        let q = Self::quotient(y as u64, (y >> 64) as u64);
        let remainder = value - u128::from(q) * u128::from(Self::P0);
        Self::reduce1::<CT>(remainder as u64, (remainder >> 64) as u8)
    }

    // The special forms.

    /// The folds that bring a value below `p 2^64` under `2 p`, for a pseudo-Mersenne modulus.
    const SCALE_FOLDS: usize = folds_from(Self::BITS, Self::DEFICIT_VALUE, Some(1 << 64));

    /// `high 2^64 + low mod p` for a pseudo-Mersenne `p = 2^n - c`, by [`Self::SCALE_FOLDS`] folds
    /// `low + high c` over `L + 1` limbs.
    #[inline(always)]
    fn fold_wide<const CT: bool>(low: u64, high: &Limbs<M>) -> Limbs<M> {
        let mut value = M::Width::EXTENDED_ZERO;
        set(value.as_mut_slice(), 0, low);
        unroll!(j in 0, Self::LIMBS => {
            set(value.as_mut_slice(), j + 1, limb(high, j));
        });
        let mut folded = M::Width::EXTENDED_ZERO;
        unroll!(step in 0, Self::SCALE_FOLDS => {
            if step.is_multiple_of(2) {
                Self::fold_once(&mut folded, Self::LIMBS + 1, value.as_slice());
            } else {
                Self::fold_once(&mut value, Self::LIMBS + 1, folded.as_slice());
            }
        });
        let result: &ExtendedLimbs<M> = if Self::SCALE_FOLDS % 2 == 1 {
            &folded
        } else {
            &value
        };
        let mut out = M::Width::ZERO;
        unroll!(j in 0, Self::LIMBS => {
            set(out.as_mut_slice(), j, limb(result, j));
        });
        let top = limb(result, Self::LIMBS) as u8;
        if CT {
            Self::reduce_select(out, top)
        } else {
            Self::reduce_branch(out, top)
        }
    }

    /// `value mod p` for `p = 2^64 - 2^32 + 1` and `value < p 2^64`: with `value = h 2^96 + m
    /// 2^64 + l`, `l - h + m (2^32 - 1)`, each wrap corrected by `2^64 = 2^32 - 1`.
    #[inline(always)]
    fn goldilocks_reduce(value: u128) -> u64 {
        const EPSILON: u64 = 0xffff_ffff;
        let (low, high) = (value as u64, (value >> 64) as u64);
        let (difference, borrow) = low.overflowing_sub(high >> 32);
        let difference = select(borrow, difference.wrapping_sub(EPSILON), difference);
        let (sum, carry) = difference.overflowing_add((high & EPSILON) * EPSILON);
        let sum = select(carry, sum.wrapping_add(EPSILON), sum);
        let (reduced, below_p) = sum.overflowing_sub(Self::P0);
        select(below_p, sum, reduced)
    }

    /// A one-limb Mersenne modulus `2^n - 1` whose [`Self::CHUNKS`] chunks of `n` bits sum
    /// within a limb.
    const MERSENNE_CHUNKED: bool = Self::MERSENNE
        && (Self::CHUNKS as u128) * (Self::P0 as u128) < 1 << 64;

    /// The chunks of `n` bits of a value below `2^(n + 65)` for a Mersenne modulus; `0` for any
    /// other.
    const CHUNKS: usize = if Self::MERSENNE {
        (Self::BITS + 65).div_ceil(Self::BITS)
    } else {
        0
    };

    /// The folds `(s mod 2^n) + (s >> n)` after which the chunk sum lies below `2^(n + 1)`.
    const CHUNK_FOLDS: usize = {
        let mut bound = Self::CHUNKS as u128 * Self::P0 as u128;
        let mut folds = 0;
        while Self::MERSENNE && bound >> Self::BITS > 1 {
            bound = Self::P0 as u128 + (bound >> Self::BITS);
            folds += 1;
        }
        folds
    };

    /// `value mod p` for a Mersenne `p = 2^n - 1` and `value < 2^(n + 65)`: the sum of its
    /// chunks of `n` bits, folded below `2^(n + 1)`, then once more below `p + 2`, and one
    /// conditional subtraction.
    #[inline(always)]
    fn mersenne_chunks(value: u128) -> u64 {
        let mut sum = 0u64;
        unroll!(chunk in 0, Self::CHUNKS => {
            sum += (value >> (chunk * Self::BITS)) as u64 & Self::P0;
        });
        unroll!(fold in 0, Self::CHUNK_FOLDS + 1 => {
            sum = (sum & Self::P0) + (sum >> Self::BITS);
        });
        let (reduced, below_p) = sum.overflowing_sub(Self::P0);
        select(below_p, sum, reduced)
    }
}
