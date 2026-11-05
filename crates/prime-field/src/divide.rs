//! The quotient `x / k` of an element by a nonzero integer `k` of fewer bits than the width.
//!
//! The divisions exist for a modulus that fills its width, so `k < 2^(n - 1) < p` for the bit
//! length `n` of `p`, and `k` has an inverse.
//!
//! In variable time, a `k` below `2^64` takes its inverse by the extended Euclidean algorithm on
//! `p` and `k`. The first step divides `p` by `k` into `p = q k + r` by the reciprocal of `k`
//! (Möller and Granlund, IEEE TC 2011); every later step divides one machine word by another.
//! The steps from `(k, r)` end in `1 = ±(s k - t r)`, whence `k^-1 = ±(s + t q) mod p`, with
//! `s + t q <= p / 2`. The element then scales by that integer. A wider `k` takes the safegcd
//! inverse of its element. In constant time, every `k` takes the constant-time inverse of its
//! element. A Montgomery field of several limbs inverts the plain residue `k` instead, with a seed
//! that makes the result the representation of `k^-1`.

use biguint::typenum::U64;
use biguint::{Bits, NonZero, NonZeroUint, Uint};

use crate::field::{Fp, Modulus};
use crate::invert::euclid;
use crate::kernel::limb;
use crate::limb::mac;
use crate::params::Shape;
use crate::width::Width;

impl<M: Modulus> Fp<M> {
    /// `self / k` for the nonzero integer `k < 2^(n - 1)` in little-endian limbs, in constant
    /// time when `CT` holds. A modulus whose bit length falls short of its width fails to
    /// compile.
    #[inline(always)]
    pub(crate) fn divide_with<const CT: bool>(self, k: &[u64]) -> Self {
        const {
            assert!(
                Shape::<M>::BITS == <M::Width as Bits>::BITS,
                "the division by an integer needs a modulus that fills its width"
            );
        };
        if CT {
            return self.mul_with::<true>(Self::inverse_of_integer::<true>(k));
        }
        let word = match k {
            [low, high @ ..] if high.iter().all(|&limb| limb == 0) => {
                NonZero::new(Uint::<U64>::from_u64(*low))
            }
            _ => None,
        };
        match word {
            Some(word) => self.divide_word(word),
            None => self.mul_with::<false>(Self::inverse_of_integer::<false>(k)),
        }
    }

    /// The inverse of the nonzero integer `k < p`, in constant time when `CT` holds. A Montgomery
    /// field of several limbs inverts `k` as its plain residue, whose seed makes the result the
    /// representation of `k^-1`; another field inverts the element of `k`.
    #[inline(always)]
    fn inverse_of_integer<const CT: bool>(k: &[u64]) -> Self {
        if Shape::<M>::LIMBS > 1 && Shape::<M>::MONTGOMERY {
            Self::inverse_of_residue::<CT>(k)
        } else {
            Self::inverse_of::<CT>(Self::element_of::<CT>(k))
        }
    }

    /// The element of the nonzero integer `k < p` in a field of plain residues or of one limb, in
    /// constant time when `CT` holds. A `k` of one limb takes the one-limb representation of an
    /// integer; a wider `k` is its own residue and takes no reduction.
    #[inline(always)]
    fn element_of<const CT: bool>(k: &[u64]) -> Self {
        if k.len() <= 1 {
            return Self::integer_with::<CT>(k);
        }
        let mut residue = M::Width::ZERO;
        residue
            .iter_mut()
            .zip(k)
            .for_each(|(slot, &limb)| *slot = limb);
        // A nonzero residue is its own plain representation: a Mersenne field needs no zero test.
        Self::from_limbs(residue)
    }

    /// `k^-1` from the plain residue `k` in a Montgomery field, in constant time when `CT` holds:
    /// a call, one per field and timing.
    #[inline(never)]
    fn inverse_of_residue<const CT: bool>(k: &[u64]) -> Self {
        let mut residue = M::Width::ZERO;
        residue
            .iter_mut()
            .zip(k)
            .for_each(|(slot, &limb)| *slot = limb);
        let seed = &Shape::<M>::RESIDUE_SEED;
        if CT {
            Self::invert_seeded::<false>(&residue, seed)
        } else {
            Self::invert_seeded::<true>(&residue, seed)
        }
    }

    /// The inverse of the nonzero `element`, in constant time when `CT` holds: a call, one per
    /// field and timing.
    #[inline(never)]
    fn inverse_of<const CT: bool>(element: Self) -> Self {
        if CT {
            biguint::Field::ct_invert_nonzero(element)
        } else {
            biguint::Field::invert_nonzero(element)
        }
    }

    /// `self / k` for `0 < k < p` of one word, in variable time.
    #[inline]
    fn divide_word(self, divisor: NonZeroUint<U64>) -> Self {
        let k = limb(divisor.get().as_limbs(), 0);
        if Shape::<M>::LIMBS == 1 {
            return self.scale_with::<false>(&[euclid(k, Shape::<M>::P0)]);
        }
        let (quotient, remainder) = M::MODULUS.div_rem(divisor);
        let remainder = limb(remainder.as_limbs(), 0);
        let (s, t, negative) = if k >> 32 == 0 {
            let (s, t, negative) = bezout32(remainder as u32, k as u32);
            (u64::from(s), u64::from(t), negative)
        } else {
            bezout64(remainder, k)
        };
        // `s + t q`, below `p / 2`.
        let mut inverse = M::Width::ZERO;
        let mut carry = s;
        for (out, &limb) in inverse.iter_mut().zip(quotient.as_limbs()) {
            (*out, carry) = mac(limb, t, carry, 0);
        }
        let product = self.scale_with::<false>(&inverse);
        if negative { -product } else { product }
    }
}

/// Generates the extended Euclidean algorithm with both coefficients on one integer type. The
/// remainders `r_i` start from `m` and `a`; `r_i = s_i m - t_i a` at even `i` and
/// `r_i = t_i a - s_i m` at odd `i`, for coefficients `s_i, t_i >= 0` that grow by
/// `c_(i + 1) = c_(i - 1) + q c_i`. Two steps run per pass, so that no value moves between
/// registers.
macro_rules! bezout {
    ($name:ident, $int:ty) => {
        /// `(s, t, negative)` with `s m - t a = 1`, or `t a - s m = 1` when `negative` holds,
        /// for `a < m` coprime to `m`. The coefficients stay below `m`.
        #[inline(always)]
        fn $name(a: $int, m: $int) -> ($int, $int, bool) {
            let (mut even, mut odd) = (m, a);
            let (mut even_s, mut odd_s): ($int, $int) = (1, 0);
            let (mut even_t, mut odd_t): ($int, $int) = (0, 1);
            loop {
                if odd == 0 {
                    return (even_s, even_t, false);
                }
                let quotient = even / odd;
                even %= odd;
                even_s += quotient * odd_s;
                even_t += quotient * odd_t;
                if even == 0 {
                    return (odd_s, odd_t, true);
                }
                let quotient = odd / even;
                odd %= even;
                odd_s += quotient * even_s;
                odd_t += quotient * even_t;
            }
        }
    };
}

bezout!(bezout32, u32);
bezout!(bezout64, u64);
