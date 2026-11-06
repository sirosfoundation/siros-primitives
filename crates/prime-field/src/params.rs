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
use crate::chain::Program;
use crate::kernel::{
    add, add_mod, below, bit, bit_len, cios, copy, equal, fits, fold, inv64, is_zero, jacobi, limb,
    montgomery_word, mul_wide, neg_mod, redc, reduce_once, set, shr, square_wide, sub, sub_mod,
    trailing_zeros, truncate, truncate_mut, widening_mul,
};
use crate::width::Width;

/// The limbs of one element of the field of `M`.
pub(crate) type Limbs<M> = GenericArray<u64, <<M as Modulus>::Width as Bits>::Limbs>;
/// The limbs of a partly folded product.
pub(crate) type ExtendedLimbs<M> = GenericArray<u64, <<M as Modulus>::Width as Width>::Extended>;
/// The limbs of a full product.
pub(crate) type WideLimbs<M> = GenericArray<u64, <<M as Modulus>::Width as Width>::Wide>;

/// The most folds a pseudo-Mersenne reduction may take before Montgomery multiplication is
/// cheaper.
const MAX_FOLDS: usize = 3;

/// The constants that share the exponentiation of the two-adic root.
const ROOT_STAGES: usize = 8;

/// The powers `c^(2^j)` of the two-adic root held in a table; Tonelli–Shanks squares the last
/// entry for a higher power.
pub(crate) const ROOT_POWERS: usize = 64;

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
    /// The low limb of the modulus.
    pub(crate) const P0: u64 = limb(Self::P.as_slice(), 0);
    /// The top limb of the modulus.
    pub(crate) const P_TOP: u64 = limb(Self::P.as_slice(), Self::LIMBS - 1);
    /// `c` of a pseudo-Mersenne modulus `2^BITS - c` with `c < 2^64`.
    const DEFICIT: Option<u64> = Self::deficit();
    /// `c` of a pseudo-Mersenne modulus; `0` for any other.
    pub(crate) const DEFICIT_VALUE: u64 = match Self::DEFICIT {
        Some(deficit) => deficit,
        None => 0,
    };
    /// The folds that reduce a product below `2 p` for the deficit.
    pub(crate) const FOLDS: usize = match Self::DEFICIT {
        Some(deficit) => folds(Self::BITS, deficit),
        None => usize::MAX,
    };
    /// One limb and `p = 2^BITS - 1`: elements are plain residues in `[1, p]`, with `0` held as
    /// `p`, so that a sum or a folded product above `p` is one at or above `2^BITS`.
    pub(crate) const MERSENNE: bool =
        Self::LIMBS == 1 && Self::BITS < 64 && matches!(Self::DEFICIT, Some(1));
    /// Several limbs and `p = 2^BITS - c` that folds below `2 p` in few steps: elements are
    /// plain residues and products fold.
    pub(crate) const PSEUDO_MERSENNE: bool = Self::LIMBS > 1 && Self::FOLDS <= MAX_FOLDS;
    /// Every other modulus, on one limb an unfolded `c` costing more than a product: elements
    /// are residues times `R` and products reduce by Montgomery multiplication.
    pub(crate) const MONTGOMERY: bool = !Self::MERSENNE && !Self::PSEUDO_MERSENNE;
    /// One limb, and a product of two elements fits the limb.
    pub(crate) const PRODUCT_FITS_LIMB: bool = Self::LIMBS == 1 && 2 * Self::BITS <= 64;
    /// One limb, and the modulus is prime, by the Miller–Rabin test with the first twelve
    /// prime bases, which decides every 64-bit number. A Montgomery product of a prime modulus
    /// whose product fits the limb lands below `p`.
    pub(crate) const PRIME_LIMB: bool = Self::LIMBS == 1 && is_prime(Self::P0);
    /// Montgomery form on WebAssembly without wide arithmetic, with a product of two elements
    /// inside the limb: `R = 2^32`, and a product reduces by [`half_redc`]. The BabyBear product
    /// took Chrome 4.1 cycles and Firefox 5.5 against 5.2 and 5.1 with `R = 2^64`, the square 5.4 and
    /// 6.0 against 4.7 and 5.7.
    pub(crate) const HALF_RADIX: bool = cfg!(all(target_arch = "wasm32", not(wide_arithmetic)))
        && Self::MONTGOMERY
        && Self::PRODUCT_FITS_LIMB;
    /// The bits of `R`: `64 L` unless [`Self::HALF_RADIX`] holds.
    const RADIX_BITS: usize = if Self::HALF_RADIX {
        32
    } else {
        64 * Self::LIMBS
    };
    /// `R mod p`.
    const R: Limbs<M> = Self::pow2_mod(Self::RADIX_BITS);
    /// `R^2 mod p`.
    pub(crate) const R2: Limbs<M> = Self::pow2_mod(2 * Self::RADIX_BITS);
    /// `R^3 mod p`: a Montgomery product by it turns the inverse of a representation into the
    /// representation of the inverse.
    pub(crate) const R3: Limbs<M> = Self::pow2_mod(3 * Self::RADIX_BITS);
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
    /// The representation of `0`: `p` for [`Self::MERSENNE`], else zero limbs.
    pub(crate) const ZERO: Limbs<M> = if Self::MERSENNE {
        Self::P
    } else {
        M::Width::ZERO
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
    /// The addition chain of `x^SQRT_EXPONENT`.
    pub(crate) const SQRT_CHAIN: Program<M> = Program::for_exponent(Self::SQRT_EXPONENT.as_slice());
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
    /// `c^(2^j)` for the two-adic root `c` at index `j < min(TWO_ADICITY, ROOT_POWERS)`; zero
    /// beyond.
    #[expect(
        clippy::indexing_slicing,
        reason = "the index is checked against the length"
    )]
    pub(crate) const TWO_ADIC_ROOT_POWERS: [Limbs<M>; ROOT_POWERS] = {
        let mut out = [const { M::Width::ZERO }; ROOT_POWERS];
        if !Self::THREE_MOD_FOUR {
            let mut power = Self::TWO_ADIC_ROOT;
            let mut index = 0;
            while index < ROOT_POWERS && index < Self::TWO_ADICITY {
                out[index] = power;
                power = Self::square_raw(power.as_slice());
                index += 1;
            }
        }
        out
    };

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
            add_mod(
                doubled.as_mut_slice(),
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

    /// `value mod p` in `[0, p]` for [`Self::MERSENNE`]: folds of the bits at and above `BITS`,
    /// which keep a nonzero value nonzero. In a sum or a product with Mersenne-31 they took 1.1
    /// to 1.5 cycles less than three chunk sums, two folds and a subtraction.
    #[inline(always)]
    pub(crate) const fn mersenne_word(value: u64) -> u64 {
        let mut folded = value;
        let mut count = 0;
        while count < Self::WORD_FOLDS {
            folded = (folded & Self::P0) + (folded >> Self::BITS);
            count += 1;
        }
        folded
    }

    /// The folds of [`Self::mersenne_word`] that bring every `u64` to at most `p = 2^BITS - 1`.
    const WORD_FOLDS: usize = mersenne_folds(Self::BITS, u64::MAX);

    /// The representation in `[1, p]` of `value` for [`Self::MERSENNE`]: the first fold of
    /// [`Self::mersenne_word`] plus `p`, which leaves every value nonzero, and the folds that bring
    /// the sum to at most `p`, which keep it nonzero.
    #[inline(always)]
    pub(crate) const fn mersenne_representation(value: u64) -> u64 {
        let mut folded = (value & Self::P0) + (value >> Self::BITS) + Self::P0;
        let mut count = 0;
        while count < Self::REPRESENTATION_FOLDS {
            folded = (folded & Self::P0) + (folded >> Self::BITS);
            count += 1;
        }
        folded
    }

    /// The folds of [`Self::mersenne_representation`] after its first; none for any other modulus.
    const REPRESENTATION_FOLDS: usize = if Self::MERSENNE {
        mersenne_folds(Self::BITS, 2 * Self::P0 + (u64::MAX >> Self::BITS))
    } else {
        0
    };

    /// Montgomery form for `p = 2^64 - 2^32 + 1`: [`Self::encode`] takes one Montgomery step
    /// of shifts.
    pub(crate) const SHIFT_ENCODE: bool = Self::MONTGOMERY && Self::SHIFT_REDC;

    /// `R 2^64 mod p`: one Montgomery step of a word times it, which divides by `2^64`, leaves the
    /// word's Montgomery form.
    pub(crate) const R_WORD: Limbs<M> = Self::pow2_mod(Self::RADIX_BITS + 64);

    /// The representation of the word `value`, in constant time: one Montgomery step of `value`
    /// times [`Self::R_WORD`], one row of products against the `L` rows of [`Self::encode`];
    /// `value mod p` on a one-limb field of plain residues; the word itself, below `p`, on a wider
    /// one.
    #[inline(always)]
    pub(crate) const fn encode_word(value: u64) -> Limbs<M> {
        if Self::SHIFT_ENCODE {
            return Self::encode(&[value]);
        }
        if Self::MONTGOMERY {
            let mut out = M::Width::ZERO;
            montgomery_word(
                out.as_mut_slice(),
                Self::R_WORD.as_slice(),
                value,
                Self::P.as_slice(),
                Self::INV,
            );
            return out;
        }
        if Self::LIMBS == 1 {
            return Self::encode(&[value % Self::P0]);
        }
        Self::encode(&[value])
    }

    /// The representation of the integer `value < p`, or of any one-limb `value` when
    /// [`Self::SHIFT_ENCODE`] holds.
    #[inline(always)]
    pub(crate) const fn encode(value: &[u64]) -> Limbs<M> {
        let mut out = M::Width::ZERO;
        if Self::SHIFT_ENCODE {
            let product = widening_mul(limb(value, 0), limb(Self::R2.as_slice(), 0));
            set(out.as_mut_slice(), 0, shift_redc(product));
        } else if Self::MONTGOMERY {
            Self::montgomery_raw(out.as_mut_slice(), value, Self::R2.as_slice());
        } else if Self::MERSENNE && is_zero(value) {
            copy(out.as_mut_slice(), Self::P.as_slice());
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
            Self::redc_raw(out.as_mut_slice(), wide.as_mut_slice());
        } else if !(Self::MERSENNE && equal(value, Self::P.as_slice())) {
            copy(out.as_mut_slice(), value);
        }
        out
    }

    /// The residue in `[0, p)` that a representation holds: the integer, or its Montgomery
    /// form. The raw arithmetic below runs on these residues and leaves results in `[0, p)`.
    #[inline(always)]
    const fn residue(value: &[u64]) -> Limbs<M> {
        let mut out = M::Width::ZERO;
        if !(Self::MERSENNE && equal(value, Self::P.as_slice())) {
            copy(out.as_mut_slice(), value);
        }
        out
    }

    /// The representation of a residue in `[0, p)`: [`Self::residue`] backwards.
    #[inline(always)]
    const fn represent(residue: Limbs<M>) -> Limbs<M> {
        if Self::MERSENNE && is_zero(residue.as_slice()) {
            Self::P
        } else {
            residue
        }
    }

    /// The sum of two representations.
    #[inline(always)]
    pub(crate) const fn add_raw(a: &[u64], b: &[u64]) -> Limbs<M> {
        let mut out = M::Width::ZERO;
        add_mod(
            out.as_mut_slice(),
            Self::residue(a).as_slice(),
            Self::residue(b).as_slice(),
            Self::P.as_slice(),
            Self::SPARE,
        );
        Self::represent(out)
    }

    /// The difference of two representations.
    #[inline(always)]
    pub(crate) const fn sub_raw(a: &[u64], b: &[u64]) -> Limbs<M> {
        let mut out = M::Width::ZERO;
        sub_mod(
            out.as_mut_slice(),
            Self::residue(a).as_slice(),
            Self::residue(b).as_slice(),
            Self::P.as_slice(),
        );
        Self::represent(out)
    }

    /// The negation of a representation.
    #[inline(always)]
    pub(crate) const fn neg_raw(a: &[u64]) -> Limbs<M> {
        let mut out = M::Width::ZERO;
        neg_mod(
            out.as_mut_slice(),
            Self::residue(a).as_slice(),
            Self::P.as_slice(),
        );
        Self::represent(out)
    }

    /// The product of two representations.
    #[inline(always)]
    pub(crate) const fn mul_raw(a: &[u64], b: &[u64]) -> Limbs<M> {
        let mut out = M::Width::ZERO;
        if Self::MONTGOMERY {
            Self::montgomery_raw(out.as_mut_slice(), a, b);
        } else {
            let mut wide = M::Width::WIDE_ZERO;
            mul_wide(
                wide.as_mut_slice(),
                Self::residue(a).as_slice(),
                Self::residue(b).as_slice(),
            );
            Self::fold_raw(&mut out, wide.as_slice());
        }
        Self::represent(out)
    }

    /// The square of a representation.
    #[inline(always)]
    pub(crate) const fn square_raw(a: &[u64]) -> Limbs<M> {
        let mut out = M::Width::ZERO;
        let mut wide = M::Width::WIDE_ZERO;
        square_wide(wide.as_mut_slice(), Self::residue(a).as_slice());
        if Self::MONTGOMERY {
            Self::redc_raw(out.as_mut_slice(), wide.as_mut_slice());
        } else {
            Self::fold_raw(&mut out, wide.as_slice());
        }
        Self::represent(out)
    }

    /// `out = a b / R mod p` for `a b < p R`.
    #[inline(always)]
    const fn montgomery_raw(out: &mut [u64], a: &[u64], b: &[u64]) {
        if Self::HALF_RADIX {
            set(out, 0, half_redc(limb(a, 0) * limb(b, 0), Self::P0, Self::INV));
        } else {
            cios(out, a, b, Self::P.as_slice(), Self::INV, Self::SPARE);
        }
    }

    /// `out = wide / R mod p` for `wide < p R`.
    #[inline(always)]
    const fn redc_raw(out: &mut [u64], wide: &mut [u64]) {
        if Self::HALF_RADIX {
            set(out, 0, half_redc(limb(wide, 0), Self::P0, Self::INV));
        } else {
            redc(out, wide, Self::P.as_slice(), Self::INV, Self::SPARE);
        }
    }

    /// The limbs that the value needs after the first, second and third fold, from the bound on
    /// its bit length.
    const FOLD_LIMBS_1: usize = fold_limbs(Self::BITS, Self::DEFICIT, 1);
    const FOLD_LIMBS_2: usize = fold_limbs(Self::BITS, Self::DEFICIT, 2);
    const FOLD_LIMBS_3: usize = fold_limbs(Self::BITS, Self::DEFICIT, 3);

    /// The limbs that the value needs after `folds` folds.
    pub(crate) const fn fold_limbs(folds: usize) -> usize {
        match folds {
            1 => Self::FOLD_LIMBS_1,
            2 => Self::FOLD_LIMBS_2,
            _ => Self::FOLD_LIMBS_3,
        }
    }

    /// Reduces the full product `wide` into `out` by [`Self::FOLDS`] folds and one conditional
    /// subtraction. Each fold writes the limbs that its bound needs.
    #[inline(always)]
    const fn fold_raw(out: &mut Limbs<M>, wide: &[u64]) {
        let deficit = match Self::DEFICIT {
            Some(deficit) => deficit,
            None => 0,
        };
        let mut first = M::Width::EXTENDED_ZERO;
        let mut second = M::Width::EXTENDED_ZERO;
        fold(
            truncate_mut(first.as_mut_slice(), Self::fold_limbs(1)),
            wide,
            Self::BITS,
            deficit,
        );
        let mut step = 1;
        while step < Self::FOLDS {
            let limbs = Self::fold_limbs(step + 1);
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
        let folded = truncate(folded, Self::fold_limbs(Self::FOLDS));
        copy(out.as_mut_slice(), folded);
        reduce_once(
            out.as_mut_slice(),
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

/// The limbs of the value after `folds` folds of the pseudo-Mersenne reduction, from the bound
/// on its bit length: a product of two residues has `2 bits` bits; a fold of a value of `b` bits
/// leaves at most `max(bits, b - bits + bits(deficit)) + 1` bits.
const fn fold_limbs(bits: usize, deficit: Option<u64>, folds: usize) -> usize {
    let deficit_bits = match deficit {
        Some(deficit) => 64 - deficit.leading_zeros() as usize,
        None => 64,
    };
    let mut bound = 2 * bits;
    let mut step = 0;
    while step < folds {
        let high = bound - bits + deficit_bits;
        bound = if high > bits { high } else { bits } + 1;
        step += 1;
    }
    bound.div_ceil(64)
}

/// The folds that bring a product of two residues modulo `2^bits - deficit` below twice the
/// modulus, with `deficit < 2^64`. Before the first fold the part above `2^bits` is below
/// `2^bits`; each fold multiplies that part by about `deficit / 2^bits`. More than [`MAX_FOLDS`]
/// folds report `MAX_FOLDS + 1`.
const fn folds(bits: usize, deficit: u64) -> usize {
    folds_from(bits, deficit, power_of_two(bits))
}

/// `2^bits`; `None` when it exceeds `u128`.
pub(crate) const fn power_of_two(bits: usize) -> Option<u128> {
    if bits < 128 {
        Some(1u128 << bits)
    } else {
        None
    }
}

/// The folds that bring a value whose part above `2^bits` lies below `high` under twice the
/// modulus `2^bits - deficit`, as [`folds`] counts them; `None` is `2^bits` beyond `u128`.
pub(crate) const fn folds_from(bits: usize, deficit: u64, high: Option<u128>) -> usize {
    let deficit = deficit as u128;
    // `2^bits`; `None` when it exceeds `u128`, which every bound below then satisfies.
    let power = power_of_two(bits);
    // An exclusive bound on the part above `2^bits` before the next fold; `None` is `2^bits`
    // beyond `u128`.
    let mut high = high;
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

/// The folds `x -> (x & p) + (x >> bits)` that bring every value up to `bound` to at most
/// `p = 2^bits - 1`, for `bits < 64`. A fold of the values up to `bound` reaches at most `p` plus
/// the high part less one, or the low and high parts of `bound` itself.
const fn mersenne_folds(bits: usize, bound: u64) -> usize {
    if bits == 0 || bits >= 64 {
        return 0;
    }
    let p = (1u64 << bits) - 1;
    let mut bound = bound;
    let mut count = 0;
    while bound > p {
        let high = bound >> bits;
        let below = p + (high - 1);
        let own = (bound & p) + high;
        bound = if below > own { below } else { own };
        count += 1;
    }
    count
}

/// The prime bases of the Miller–Rabin test that decide every 64-bit number.
const PRIME_BASES: [u64; 12] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];

/// Whether `n` is prime, by the Miller–Rabin test with [`PRIME_BASES`].
const fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    let mut index = 0;
    while index < PRIME_BASES.len() {
        let base = prime_base(index);
        if n == base {
            return true;
        }
        if n.is_multiple_of(base) {
            return false;
        }
        index += 1;
    }
    // `n - 1 = d 2^s` with `d` odd.
    let s = (n - 1).trailing_zeros();
    let d = (n - 1) >> s;
    let mut index = 0;
    while index < PRIME_BASES.len() {
        let mut x = pow_mod(prime_base(index), d, n);
        let mut witness = x != 1 && x != n - 1;
        let mut round = 1;
        while witness && round < s {
            x = mul_mod(x, x, n);
            witness = x != n - 1;
            round += 1;
        }
        if witness {
            return false;
        }
        index += 1;
    }
    true
}

/// Base `index` of [`PRIME_BASES`]; `2` beyond them.
const fn prime_base(index: usize) -> u64 {
    match PRIME_BASES.split_at_checked(index) {
        Some((_, [base, ..])) => *base,
        Some((_, [])) | None => 2,
    }
}

/// `a b mod n`.
const fn mul_mod(a: u64, b: u64, n: u64) -> u64 {
    (a as u128 * b as u128 % n as u128) as u64
}

/// `base^exponent mod n`, by square-and-multiply.
const fn pow_mod(base: u64, exponent: u64, n: u64) -> u64 {
    let mut result = 1 % n;
    let mut square = base % n;
    let mut rest = exponent;
    while rest > 0 {
        if rest & 1 == 1 {
            result = mul_mod(result, square, n);
        }
        square = mul_mod(square, square, n);
        rest >>= 1;
    }
    result
}

/// Montgomery reduction `x / 2^32 mod p` for `p < 2^32` and `x < p 2^32`, with `inv = -p^-1 mod
/// 2^64`, in plonky3's `monty_reduce` order: `x - t p` is a multiple of `2^32` for `t = x p^-1 mod
/// 2^32`, and its high half lies in `(-p, p)`. The selection compiles to `select`.
#[inline(always)]
pub(crate) const fn half_redc(x: u64, p: u64, inv: u64) -> u64 {
    let t = x.wrapping_mul(inv.wrapping_neg()) & 0xffff_ffff;
    let (difference, negative) = x.overflowing_sub(t * p);
    let high = (difference >> 32) as u32;
    let correction = if negative { p as u32 } else { 0 };
    high.wrapping_add(correction) as u64
}

/// [`half_redc`] with the correction from the sign of the difference, by shifts.
#[inline(always)]
pub(crate) const fn half_redc_signed(x: u64, p: u64, inv: u64) -> u64 {
    let t = x.wrapping_mul(inv.wrapping_neg()) & 0xffff_ffff;
    let difference = x.wrapping_sub(t * p) as i64;
    ((difference >> 32) + ((difference >> 63) & p as i64)) as u64
}

/// Montgomery reduction `value / 2^64 mod p` for `p = 2^64 - 2^32 + 1` and `value < p 2^64`, in
/// winter-math's `mont_red_cst` order: `m = low + (low << 32)` with its carry `e`, and
/// `(m p) >> 64 = m - (m >> 32) - e` from `m`. A conversion runs unchained, where this order takes
/// fewer cycles per element than [`Shape::SHIFT_REDC`]'s order, which shortens a chain of products.
/// A borrow adds `p`, which modulo `2^64` subtracts `2^32 - 1`.
pub(crate) const fn shift_redc(value: u128) -> u64 {
    let (low, high) = (value as u64, (value >> 64) as u64);
    let (m, e) = low.overflowing_add(low << 32);
    let quotient = m.wrapping_sub(m >> 32).wrapping_sub(e as u64);
    let (result, borrow) = high.overflowing_sub(quotient);
    result.wrapping_sub(0u32.wrapping_sub(borrow as u32) as u64)
}
