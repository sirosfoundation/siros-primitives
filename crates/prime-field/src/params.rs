//! The constants that a modulus determines, computed at compile time, and the raw arithmetic on
//! limb arrays that they parameterize.
//!
//! Every constant is an associated `const` of [`Shape<M>`], so it is evaluated once per modulus
//! and folds into the code of every operation: the modulus limbs are immediates, and each
//! strategy choice is a constant condition that the compiler resolves.

use core::marker::PhantomData;

use biguint::Bits;
use generic_array::GenericArray;

use crate::Modulus;
use crate::kernel::{
    add, add_mod, below, bit, bit_len, cios, copy, fits, fold, inv64, is_zero, jacobi, limb,
    mul_wide, redc, reduce_once, set, shr, square_wide, sub, trailing_zeros, truncate,
    truncate_mut,
};
use crate::width::Width;

/// The limbs of one element of the field of `M`.
pub(crate) type Limbs<M> = GenericArray<u64, <<M as Modulus>::Width as Bits>::Limbs>;
/// The limbs of a partly folded product.
pub(crate) type ExtendedLimbs<M> = GenericArray<u64, <<M as Modulus>::Width as Width>::Extended>;

/// The most folds a pseudo-Mersenne reduction may take before Montgomery multiplication is
/// cheaper.
const MAX_FOLDS: usize = 3;

/// The constants that share the exponentiation of the two-adic root.
const ROOT_STAGES: usize = 8;

/// The constants and raw arithmetic of the field of `M`.
pub(crate) struct Shape<M>(PhantomData<M>);

impl<M: Modulus> Shape<M> {
    /// The limbs of the modulus.
    pub(crate) const P: Limbs<M> = {
        let mut out = M::Width::ZERO;
        copy(out.as_mut_slice(), M::MODULUS.as_limbs());
        out
    };
    /// The limbs of an element.
    pub(crate) const LIMBS: usize = <M::Width as Bits>::LIMBS;
    /// The bit length of the modulus.
    pub(crate) const BITS: usize = bit_len(Self::P.as_slice());
    /// The modulus leaves the top bit of the top limb clear, so a sum of two elements fits the
    /// limbs and the Montgomery accumulator needs one carry limb.
    pub(crate) const SPARE: bool = Self::BITS < 64 * Self::LIMBS;
    /// The modulus is odd and at least 3. A use of any other constant evaluates this one.
    const VALID: () = assert!(
        limb(Self::P.as_slice(), 0) & 1 == 1 && Self::BITS >= 2,
        "the modulus of a prime field is an odd number of at least two bits"
    );
    /// `-p^-1 mod 2^64`.
    pub(crate) const INV: u64 = inv64(limb(Self::P.as_slice(), 0));
    /// `c` of a pseudo-Mersenne modulus `2^BITS - c` with `c < 2^64`.
    const DEFICIT: Option<u64> = Self::deficit();
    /// The folds that reduce a product below `2 p` for the deficit.
    const FOLDS: usize = match Self::DEFICIT {
        Some(deficit) => folds(Self::BITS, deficit),
        None => usize::MAX,
    };
    /// Products reduce by pseudo-Mersenne folding; elements are plain residues.
    pub(crate) const PSEUDO_MERSENNE: bool = Self::FOLDS <= MAX_FOLDS;
    /// Products reduce by Montgomery multiplication; elements are residues times `R`.
    pub(crate) const MONTGOMERY: bool = !Self::PSEUDO_MERSENNE;
    /// `R = 2^(64 L) mod p`.
    const R: Limbs<M> = Self::pow2_mod(64 * Self::LIMBS);
    /// `R^2 mod p`.
    pub(crate) const R2: Limbs<M> = Self::pow2_mod(128 * Self::LIMBS);
    /// The representation of `1`.
    pub(crate) const ONE: Limbs<M> = {
        let () = Self::VALID;
        if Self::MONTGOMERY {
            Self::R
        } else {
            let mut one = M::Width::ZERO;
            set(one.as_mut_slice(), 0, 1);
            one
        }
    };
    /// `p = 3 mod 4`: a square root is `x^((p + 1) / 4)`.
    pub(crate) const THREE_MOD_FOUR: bool = limb(Self::P.as_slice(), 0) & 3 == 3;
    /// `s` with `p - 1 = 2^s t`, `t` odd.
    pub(crate) const TWO_ADICITY: usize = trailing_zeros(Self::P_MINUS_ONE.as_slice());
    /// `p - 1`.
    const P_MINUS_ONE: Limbs<M> = {
        let mut out = M::Width::ZERO;
        copy(out.as_mut_slice(), Self::P.as_slice());
        set(out.as_mut_slice(), 0, limb(Self::P.as_slice(), 0) - 1);
        out
    };
    /// `t`, the odd part of `p - 1`.
    const ODD_PART: Limbs<M> = {
        let mut out = M::Width::ZERO;
        shr(
            out.as_mut_slice(),
            Self::P_MINUS_ONE.as_slice(),
            Self::TWO_ADICITY,
        );
        out
    };
    /// The exponent of the square root: `(p + 1) / 4` when `p = 3 mod 4`, else `(t - 1) / 2`,
    /// the first step of Tonelli–Shanks.
    pub(crate) const SQRT_EXPONENT: Limbs<M> = {
        let mut out = M::Width::ZERO;
        if Self::THREE_MOD_FOUR {
            let mut quarter = M::Width::ZERO;
            shr(quarter.as_mut_slice(), Self::P.as_slice(), 2);
            fits(add(out.as_mut_slice(), quarter.as_slice(), &[1]));
        } else {
            shr(out.as_mut_slice(), Self::ODD_PART.as_slice(), 1);
        }
        out
    };
    /// The representation of the least quadratic non-residue, by the Jacobi symbol. Zero when
    /// `p = 3 mod 4`, which needs none.
    const NON_RESIDUE: Limbs<M> = if Self::THREE_MOD_FOUR {
        M::Width::ZERO
    } else {
        let mut candidate = 2;
        while jacobi(candidate, Self::P.as_slice()) != -1 {
            candidate += 1;
        }
        let mut value = M::Width::ZERO;
        set(value.as_mut_slice(), 0, candidate);
        Self::encode(value.as_slice())
    };
    /// `z^t` for the non-residue `z`: a generator of the 2-Sylow subgroup, the seed of
    /// Tonelli–Shanks. The exponentiation runs in [`ROOT_STAGES`] constants, each over a slice
    /// of the exponent bits, so that no single constant evaluation exceeds the compiler's budget.
    pub(crate) const TWO_ADIC_ROOT: Limbs<M> = Self::root_stage(Self::ROOT_7, 8);
    const ROOT_7: Limbs<M> = Self::root_stage(Self::ROOT_6, 7);
    const ROOT_6: Limbs<M> = Self::root_stage(Self::ROOT_5, 6);
    const ROOT_5: Limbs<M> = Self::root_stage(Self::ROOT_4, 5);
    const ROOT_4: Limbs<M> = Self::root_stage(Self::ROOT_3, 4);
    const ROOT_3: Limbs<M> = Self::root_stage(Self::ROOT_2, 3);
    const ROOT_2: Limbs<M> = Self::root_stage(Self::ROOT_1, 2);
    const ROOT_1: Limbs<M> = Self::root_stage(Self::ONE, 1);

    /// Raises `previous`, the root exponentiation over the exponent bits above the stage's
    /// slice, through the slice of bits of `stage`.
    const fn root_stage(previous: Limbs<M>, stage: usize) -> Limbs<M> {
        if Self::THREE_MOD_FOUR {
            return M::Width::ZERO;
        }
        let width = 64 * Self::LIMBS;
        Self::pow_segment(
            previous,
            Self::NON_RESIDUE.as_slice(),
            Self::ODD_PART.as_slice(),
            width - (stage - 1) * width / ROOT_STAGES,
            width - stage * width / ROOT_STAGES,
        )
    }

    /// `c` when `p = 2^BITS - c` with `c < 2^64`.
    const fn deficit() -> Option<u64> {
        // `2^BITS - p` over the limbs: the top limb of `2^BITS` is `1 << (BITS % 64)`, or the
        // carry into the limb above the element when `BITS` fills the limbs.
        let mut power = M::Width::EXTENDED_ZERO;
        set(
            power.as_mut_slice(),
            Self::BITS / 64,
            1 << (Self::BITS % 64),
        );
        let mut difference = M::Width::EXTENDED_ZERO;
        fits(sub(
            difference.as_mut_slice(),
            power.as_slice(),
            Self::P.as_slice(),
        ));
        let (low, high) = difference.as_slice().split_at(1);
        if is_zero(high) {
            Some(limb(low, 0))
        } else {
            None
        }
    }

    /// `2^exponent mod p`, by doubling.
    const fn pow2_mod(exponent: usize) -> Limbs<M> {
        let mut value = M::Width::ZERO;
        set(value.as_mut_slice(), 0, 1);
        let mut step = 0;
        while step < exponent {
            let mut doubled = M::Width::ZERO;
            let mut scratch = M::Width::ZERO;
            add_mod(
                doubled.as_mut_slice(),
                scratch.as_mut_slice(),
                value.as_slice(),
                value.as_slice(),
                Self::P.as_slice(),
                Self::SPARE,
            );
            value = doubled;
            step += 1;
        }
        value
    }

    /// The representation of the integer `value < p`.
    #[inline(always)]
    pub(crate) const fn encode(value: &[u64]) -> Limbs<M> {
        let mut out = M::Width::ZERO;
        if Self::MONTGOMERY {
            let mut scratch = M::Width::ZERO;
            cios(
                out.as_mut_slice(),
                scratch.as_mut_slice(),
                value,
                Self::R2.as_slice(),
                Self::P.as_slice(),
                Self::INV,
                Self::SPARE,
            );
        } else {
            copy(out.as_mut_slice(), value);
        }
        out
    }

    /// The integer `< p` that `value` represents.
    #[inline(always)]
    pub(crate) const fn decode(value: &[u64]) -> Limbs<M> {
        let mut out = M::Width::ZERO;
        if Self::MONTGOMERY {
            let mut wide = M::Width::WIDE_ZERO;
            copy(wide.as_mut_slice(), value);
            let mut scratch = M::Width::ZERO;
            redc(
                out.as_mut_slice(),
                scratch.as_mut_slice(),
                wide.as_mut_slice(),
                Self::P.as_slice(),
                Self::INV,
                Self::SPARE,
            );
        } else {
            copy(out.as_mut_slice(), value);
        }
        out
    }

    /// The product of two representations.
    #[inline(always)]
    pub(crate) const fn mul_raw(a: &[u64], b: &[u64]) -> Limbs<M> {
        let mut out = M::Width::ZERO;
        let mut scratch = M::Width::ZERO;
        if Self::MONTGOMERY {
            cios(
                out.as_mut_slice(),
                scratch.as_mut_slice(),
                a,
                b,
                Self::P.as_slice(),
                Self::INV,
                Self::SPARE,
            );
        } else {
            let mut wide = M::Width::WIDE_ZERO;
            mul_wide(wide.as_mut_slice(), a, b);
            Self::fold_raw(&mut out, &mut scratch, wide.as_slice());
        }
        out
    }

    /// The square of a representation.
    #[inline(always)]
    pub(crate) const fn square_raw(a: &[u64]) -> Limbs<M> {
        let mut out = M::Width::ZERO;
        let mut scratch = M::Width::ZERO;
        let mut wide = M::Width::WIDE_ZERO;
        square_wide(wide.as_mut_slice(), a);
        if Self::MONTGOMERY {
            redc(
                out.as_mut_slice(),
                scratch.as_mut_slice(),
                wide.as_mut_slice(),
                Self::P.as_slice(),
                Self::INV,
                Self::SPARE,
            );
        } else {
            Self::fold_raw(&mut out, &mut scratch, wide.as_slice());
        }
        out
    }

    /// The limbs that the value needs after each fold, from the bound on its bit length.
    const FOLD_LIMBS: [usize; MAX_FOLDS] = fold_limbs(Self::BITS, Self::DEFICIT);

    /// Reduces the full product `wide` into `out` by [`Self::FOLDS`] folds and one conditional
    /// subtraction. Each fold writes the limbs that its bound needs.
    #[inline(always)]
    const fn fold_raw(out: &mut Limbs<M>, scratch: &mut Limbs<M>, wide: &[u64]) {
        let deficit = match Self::DEFICIT {
            Some(deficit) => deficit,
            None => 0,
        };
        let mut first = M::Width::EXTENDED_ZERO;
        let mut second = M::Width::EXTENDED_ZERO;
        fold(
            truncate_mut(first.as_mut_slice(), Self::FOLD_LIMBS[0]),
            wide,
            Self::BITS,
            deficit,
        );
        let mut step = 1;
        while step < Self::FOLDS {
            let limbs = Self::FOLD_LIMBS[step];
            if step % 2 == 1 {
                fold(
                    truncate_mut(second.as_mut_slice(), limbs),
                    first.as_slice(),
                    Self::BITS,
                    deficit,
                );
            } else {
                fold(
                    truncate_mut(first.as_mut_slice(), limbs),
                    second.as_slice(),
                    Self::BITS,
                    deficit,
                );
            }
            step += 1;
        }
        let folded = if Self::FOLDS % 2 == 1 {
            first.as_slice()
        } else {
            second.as_slice()
        };
        let folded = truncate(folded, Self::FOLD_LIMBS[Self::FOLDS - 1]);
        copy(out.as_mut_slice(), folded);
        reduce_once(
            out.as_mut_slice(),
            scratch.as_mut_slice(),
            limb(folded, Self::LIMBS),
            Self::P.as_slice(),
        );
    }

    /// Continues the square-and-multiply of `base^exponent` from `acc`, the result over the
    /// exponent bits at and above `high`, through the bits in `low..high`.
    const fn pow_segment(
        acc: Limbs<M>,
        base: &[u64],
        exponent: &[u64],
        high: usize,
        low: usize,
    ) -> Limbs<M> {
        let mut result = acc;
        let mut index = high;
        while index > low {
            index -= 1;
            result = Self::square_raw(result.as_slice());
            if bit(exponent, index) {
                result = Self::mul_raw(result.as_slice(), base);
            }
        }
        result
    }

    /// Whether `value` is below the modulus.
    #[inline(always)]
    pub(crate) const fn is_canonical(value: &[u64]) -> bool {
        below(value, Self::P.as_slice())
    }
}

/// The limbs of the value after each fold of the pseudo-Mersenne reduction, from the bound on
/// its bit length: a product of two residues has `2 bits` bits; a fold of a value of `b` bits
/// leaves at most `max(bits, b - bits + bits(deficit)) + 1` bits.
const fn fold_limbs(bits: usize, deficit: Option<u64>) -> [usize; MAX_FOLDS] {
    let deficit_bits = match deficit {
        Some(deficit) => 64 - deficit.leading_zeros() as usize,
        None => 64,
    };
    let mut limbs = [0; MAX_FOLDS];
    let mut bound = 2 * bits;
    let mut step = 0;
    while step < MAX_FOLDS {
        let high = bound - bits + deficit_bits;
        bound = if high > bits { high } else { bits } + 1;
        limbs[step] = bound.div_ceil(64);
        step += 1;
    }
    limbs
}

/// The folds that bring a product of two residues modulo `2^bits - deficit` below twice the
/// modulus, with `deficit < 2^64`. Before the first fold the part above `2^bits` is below
/// `2^bits`; each fold multiplies that part by about `deficit / 2^bits`. More than [`MAX_FOLDS`]
/// folds report `MAX_FOLDS + 1`.
const fn folds(bits: usize, deficit: u64) -> usize {
    let deficit = deficit as u128;
    // `2^bits`; `None` when it exceeds `u128`, which every bound below then satisfies.
    let power = if bits < 128 {
        Some(1u128 << bits)
    } else {
        None
    };
    // An exclusive bound on the part above `2^bits` before the next fold; `None` is `2^bits`
    // beyond `u128`.
    let mut high = power;
    let mut count = 0;
    while count < MAX_FOLDS {
        count += 1;
        // After the fold the value is at most `2^bits - 1 + (high - 1) deficit`, which is below
        // `2 (2^bits - deficit)` when `(high + 1) deficit <= 2^bits`.
        let done = match (high, power) {
            (Some(high), Some(power)) => match (high + 1).checked_mul(deficit) {
                Some(bound) => bound <= power,
                None => false,
            },
            (Some(_), None) => true,
            (None, _) => false,
        };
        if done {
            return count;
        }
        high = Some(match (high, power) {
            (Some(high), Some(power)) => ((high - 1) * deficit).div_ceil(power) + 1,
            (Some(_), None) => 1,
            (None, _) => deficit + 1,
        });
    }
    MAX_FOLDS + 1
}
