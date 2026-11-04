//! Division steps with a precomputed reciprocal, after Möller and Granlund, "Improved division
//! by invariant integers" (IEEE Transactions on Computers 60(2), 2011).
//!
//! On x86-64 the reciprocal of a limb is one 128-by-64 division; elsewhere Algorithm 2 computes it
//! without a division. Algorithm 4 divides two limbs by one, and Algorithm 5 divides three limbs by
//! two; each costs two or three multiplications.
//! Algorithm 6 extends the reciprocal to a two-limb divisor. Algorithms 4 to 6 are written once
//! over the word size, so the tests run the same code on 8-bit words, where every boundary case is
//! reachable.

/// Algorithms 4, 5 and 6 over the word `$word` (`β = 2^BITS`) and its double `$double`.
macro_rules! division_steps {
    ($word:ty, $double:ty) => {
        const BITS: u32 = <$word>::BITS;

        /// `high * β + low`.
        #[inline(always)]
        pub(crate) const fn join(high: $word, low: $word) -> $double {
            ((high as $double) << BITS) | low as $double
        }

        /// The high word of `value`.
        #[inline(always)]
        const fn high(value: $double) -> $word {
            (value >> BITS) as $word
        }

        /// `(q, r)` with `high * β + low = q * d + r` and `r < d`, for a normalized `d` with
        /// reciprocal `v = floor((β^2 - 1) / d) - β` and `high < d` (Algorithm 4).
        #[inline(always)]
        pub(super) fn div_2_by_1(
            high_word: $word,
            low: $word,
            divisor: $word,
            reciprocal: $word,
        ) -> ($word, $word) {
            let estimate =
                <$double>::from(reciprocal) * <$double>::from(high_word) + join(high_word, low);
            let fraction = estimate as $word;
            let quotient = high(estimate).wrapping_add(1);
            let remainder = low.wrapping_sub(quotient.wrapping_mul(divisor));
            // The candidate is one too large when the remainder wraps past the fraction.
            let (quotient, remainder) =
                one_less(remainder > fraction, quotient, remainder, divisor);
            // Rarely, it is one too small.
            if remainder >= divisor {
                one_more(quotient, remainder, divisor)
            } else {
                (quotient, remainder)
            }
        }

        /// [`div_2_by_1`] with both corrections applied under masks, so that no branch and no
        /// select depends on the operands.
        #[inline(always)]
        pub(super) fn div_2_by_1_masked(
            high_word: $word,
            low: $word,
            divisor: $word,
            reciprocal: $word,
        ) -> ($word, $word) {
            let estimate =
                <$double>::from(reciprocal) * <$double>::from(high_word) + join(high_word, low);
            let fraction = estimate as $word;
            let quotient = high(estimate).wrapping_add(1);
            let remainder = low.wrapping_sub(quotient.wrapping_mul(divisor));
            // All ones when the candidate is one too large: the remainder wrapped past the
            // fraction.
            let excess = below(fraction, remainder);
            let quotient = quotient.wrapping_add(excess);
            let remainder = remainder.wrapping_add(divisor & excess);
            // All ones when it is one too small.
            let short = !below(remainder, divisor);
            (
                quotient.wrapping_sub(short),
                remainder.wrapping_sub(divisor & short),
            )
        }

        /// `(quotient - 1, remainder + divisor)` when `excess` holds, `(quotient, remainder)`
        /// otherwise. `excess` holds in about half the steps, so a branch on it mispredicts; the
        /// unpredictable selects become conditional moves, which LLVM keeps inside a loop.
        #[inline(always)]
        fn one_less(
            excess: bool,
            quotient: $word,
            remainder: $word,
            divisor: $word,
        ) -> ($word, $word) {
            (
                ::core::hint::select_unpredictable(excess, quotient.wrapping_sub(1), quotient),
                ::core::hint::select_unpredictable(
                    excess,
                    remainder.wrapping_add(divisor),
                    remainder,
                ),
            )
        }

        /// `(quotient + 1, remainder - divisor)`.
        #[inline(always)]
        fn one_more(quotient: $word, remainder: $word, divisor: $word) -> ($word, $word) {
            crate::limb::rare();
            (quotient + 1, remainder - divisor)
        }

        /// [`crate::limb::mask_below`] over the word: all ones when `a < b`, zero otherwise.
        #[inline(always)]
        const fn below(a: $word, b: $word) -> $word {
            high((a as $double).wrapping_sub(b as $double))
        }

        /// `floor((β^3 - 1) / d) - β` for `d = high * β + low` with `high` normalized, from the
        /// reciprocal of `high` (Algorithm 6). The corrections subtract masks of `high`
        /// ([`below`]): LLVM turns a select of `high` into a branch inside a loop.
        #[inline(always)]
        pub(super) fn reciprocal_3_by_2(high_word: $word, low: $word, reciprocal: $word) -> $word {
            // Fold `low` into the product `(β + v) * high`. A carry out takes one from `v` and
            // `high` from the product, twice when the product reaches `high`.
            let product = high_word.wrapping_mul(reciprocal).wrapping_add(low);
            let carry = below(product, low);
            let twice = carry & !below(product, high_word);
            let reciprocal = reciprocal.wrapping_add(carry).wrapping_add(twice);
            let product = product
                .wrapping_sub(high_word & carry)
                .wrapping_sub(high_word & twice);
            // Fold the high word of `v * low` in: a carry out takes one from `v`, twice when the
            // two-word product reaches the divisor.
            let cross = <$double>::from(reciprocal) * <$double>::from(low);
            let (product, carry) = product.overflowing_add(high(cross));
            let twice = carry & (join(product, cross as $word) >= join(high_word, low));
            reciprocal
                .wrapping_sub(<$word>::from(carry))
                .wrapping_sub(<$word>::from(twice))
        }

        /// `(q, r)` with `top * β + low = q * d + r` and `r < d`, for a normalized two-word `d`
        /// with reciprocal `floor((β^3 - 1) / d) - β` and `top < d` (Algorithm 5).
        #[inline(always)]
        pub(super) fn div_3_by_2(
            top: $double,
            low: $word,
            divisor: $double,
            reciprocal: $word,
        ) -> ($word, $double) {
            let estimate = <$double>::from(high(top)) * <$double>::from(reciprocal) + top;
            let (quotient, fraction) = (high(estimate), estimate as $word);
            let remainder_high = (top as $word).wrapping_sub(quotient.wrapping_mul(high(divisor)));
            let remainder = join(remainder_high, low)
                .wrapping_sub(<$double>::from(divisor as $word) * <$double>::from(quotient))
                .wrapping_sub(divisor);
            let quotient = quotient.wrapping_add(1);
            // The candidate is one too large when the remainder's high word reaches the fraction.
            let (quotient, remainder) =
                one_less_wide(high(remainder) >= fraction, quotient, remainder, divisor);
            // Rarely, it is one too small.
            if remainder >= divisor {
                one_more_wide(quotient, remainder, divisor)
            } else {
                (quotient, remainder)
            }
        }

        /// [`one_less`] with a two-word remainder.
        #[inline(always)]
        fn one_less_wide(
            excess: bool,
            quotient: $word,
            remainder: $double,
            divisor: $double,
        ) -> ($word, $double) {
            (
                ::core::hint::select_unpredictable(excess, quotient.wrapping_sub(1), quotient),
                ::core::hint::select_unpredictable(
                    excess,
                    remainder.wrapping_add(divisor),
                    remainder,
                ),
            )
        }

        /// `(quotient + 1, remainder - divisor)`.
        #[inline(always)]
        fn one_more_wide(
            quotient: $word,
            remainder: $double,
            divisor: $double,
        ) -> ($word, $double) {
            crate::limb::rare();
            (quotient + 1, remainder - divisor)
        }
    };
}

mod limb {
    division_steps!(u64, u128);
}

pub(super) use limb::join;

/// A limb with its top bit set.
#[derive(Clone, Copy)]
pub(super) struct NormalizedLimb(u64);

impl NormalizedLimb {
    /// `limb`, whose top bit the normalization has set. Setting it again tells the compiler that
    /// the divisor is not zero.
    #[inline(always)]
    pub(super) const fn top_set(limb: u64) -> Self {
        Self(limb | 1 << 63)
    }
}

/// A normalized limb `d` with its reciprocal `floor((2^128 - 1) / d) - 2^64`.
#[derive(Clone, Copy)]
pub(crate) struct Reciprocal {
    divisor: u64,
    reciprocal: u64,
}

impl Reciprocal {
    #[inline(always)]
    pub(super) fn new(divisor: NormalizedLimb) -> Self {
        Self {
            divisor: divisor.0,
            reciprocal: reciprocal_word(divisor),
        }
    }

    /// `(q, r)` with `high * 2^64 + low = q * d + r` and `r < d`, for `high < d`.
    #[inline(always)]
    pub(super) fn div_rem(self, high: u64, low: u64) -> (u64, u64) {
        limb::div_2_by_1(high, low, self.divisor, self.reciprocal)
    }
    /// [`Self::div_rem`] with its corrections under masks: no branch and no select depends on
    /// `high` or `low`.
    #[inline(always)]
    pub(super) fn div_rem_masked(self, high: u64, low: u64) -> (u64, u64) {
        limb::div_2_by_1_masked(high, low, self.divisor, self.reciprocal)
    }
}

/// A normalized two-limb divisor `d` with its reciprocal `floor((2^192 - 1) / d) - 2^64`.
#[derive(Clone, Copy)]
pub(crate) struct Reciprocal3By2 {
    divisor: u128,
    reciprocal: u64,
}

impl Reciprocal3By2 {
    /// The reciprocal of `high * 2^64 + low`.
    #[inline(always)]
    pub(super) fn new(high: NormalizedLimb, low: u64) -> Self {
        Self {
            divisor: join(high.0, low),
            reciprocal: limb::reciprocal_3_by_2(high.0, low, reciprocal_word(high)),
        }
    }

    /// The divisor's two limbs.
    #[inline(always)]
    pub(super) const fn divisor(self) -> u128 {
        self.divisor
    }

    /// `(q, r)` with `top * 2^64 + low = q * d + r` and `r < d`, for `top < d`.
    #[inline(always)]
    pub(super) fn div_rem(self, top: u128, low: u64) -> (u64, u128) {
        limb::div_3_by_2(top, low, self.divisor, self.reciprocal)
    }
}

/// `floor((2^128 - 1) / d) - 2^64` for a normalized limb `d`.
#[inline(always)]
fn reciprocal_word(divisor: NormalizedLimb) -> u64 {
    #[cfg(target_arch = "x86_64")]
    {
        divided_reciprocal(divisor)
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        newton_reciprocal(divisor)
    }
}

/// The reciprocal as the quotient of `(β - 1 - d) β + (β - 1)` by `d`, which fits in a limb because
/// `β - 1 - d < d`: the compiler's 128-bit division reaches its one-instruction path, `div` on
/// x86-64. On Zen 3 a 256-bit division by a 128-bit divisor takes 91 cycles this way, against 105
/// with Algorithm 2 and the seed table, and 111 with the seed by a 32-bit division.
#[cfg(any(test, target_arch = "x86_64"))]
#[inline(always)]
fn divided_reciprocal(divisor: NormalizedLimb) -> u64 {
    let d = divisor.0;
    ((u128::from(!d) << 64 | u128::from(u64::MAX)) / u128::from(d)) as u64
}

/// The reciprocal by Algorithm 2: a seed for the top nine bits, two Newton steps, then a step that
/// adds zero or one.
#[cfg(any(test, not(target_arch = "x86_64")))]
#[inline(always)]
fn newton_reciprocal(divisor: NormalizedLimb) -> u64 {
    let d = divisor.0;
    let d0 = d & 1;
    let d9 = d >> 55;
    let d40 = (d >> 24) + 1;
    let d63 = (d >> 1) + d0;

    let v0 = reciprocal_seed(d9);
    let v1 = (v0 << 11)
        .wrapping_sub(v0.wrapping_mul(v0).wrapping_mul(d40) >> 40)
        .wrapping_sub(1);
    let v2 = (v1 << 13)
        .wrapping_add(v1.wrapping_mul((1u64 << 60).wrapping_sub(v1.wrapping_mul(d40))) >> 47);
    let e = ((v2 >> 1) & d0.wrapping_neg()).wrapping_sub(v2.wrapping_mul(d63));
    let v3 = (high_product(v2, e, 0) >> 1).wrapping_add(v2 << 31);
    v3.wrapping_sub(high_product(v3, d, d)).wrapping_sub(d)
}

/// The numerator of the reciprocal seed: `floor(SEED_NUMERATOR / d9)` approximates `2^74 / d`.
#[cfg(any(test, not(target_arch = "x86_64")))]
const SEED_NUMERATOR: u32 = (1 << 19) - 3 * (1 << 8);

/// `floor(SEED_NUMERATOR / d9)` for `d9` in `256..512`, the top nine bits of a normalized limb.
#[cfg(all(
    not(feature = "not-precomputed"),
    any(test, not(target_arch = "x86_64"))
))]
#[inline(always)]
fn reciprocal_seed(d9: u64) -> u64 {
    RECIPROCAL_SEEDS
        .get((d9 & 0xff) as usize)
        .map_or(0, |seed| u64::from(*seed))
}

/// `floor(SEED_NUMERATOR / d9)` for `d9` in `256..512`, the top nine bits of a normalized limb.
#[cfg(all(feature = "not-precomputed", any(test, not(target_arch = "x86_64"))))]
#[inline(always)]
fn reciprocal_seed(d9: u64) -> u64 {
    u64::from(SEED_NUMERATOR.checked_div(d9 as u32).unwrap_or(0))
}

/// The seeds for `d9 = 256..512`.
#[cfg(all(
    not(feature = "not-precomputed"),
    any(test, not(target_arch = "x86_64"))
))]
static RECIPROCAL_SEEDS: [u16; 256] = reciprocal_seeds();

#[cfg(all(
    not(feature = "not-precomputed"),
    any(test, not(target_arch = "x86_64"))
))]
#[expect(
    clippy::indexing_slicing,
    reason = "constant evaluation: an index out of range fails the build"
)]
const fn reciprocal_seeds() -> [u16; 256] {
    let mut seeds = [0; 256];
    let mut index = 0;
    while index < 256 {
        seeds[index] = (SEED_NUMERATOR / (index as u32 + 256)) as u16;
        index += 1;
    }
    seeds
}

/// The high limb of `a * b + c`.
#[cfg(any(test, not(target_arch = "x86_64")))]
#[inline(always)]
fn high_product(a: u64, b: u64, c: u64) -> u64 {
    ((u128::from(a) * u128::from(b) + u128::from(c)) >> 64) as u64
}

#[cfg(test)]
mod tests;
