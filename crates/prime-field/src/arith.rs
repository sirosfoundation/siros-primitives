//! The runtime arithmetic of a field: the variable-time kernels of [`Fp`](crate::Fp) and the
//! constant-time kernels of [`ConstantTime`](crate::ConstantTime), on the representation that the
//! modulus selects.
//!
//! `CT` selects the constant-time form. The variable-time form takes a branch where the cycles
//! of the branch with its mispredictions undercut the conditional moves: the final subtraction
//! of a product whose result rarely reaches `p` ([`Shape::FINAL_BRANCH`]). There the top limb
//! alone decides: a smaller top limb ends the product, any other takes `p` off, and `p` comes
//! back in the rare case that the subtraction borrows. Every other reduction runs without a
//! branch, since a sum or a difference reaches `p` or borrows for half the operands: it either
//! selects between two carry chains by conditional moves, or adds `p` under the mask of a
//! borrow, whichever costs fewer instructions for the limbs of `p` ([`Shape::REDUCE_MASKED`],
//! [`Shape::SUB_MASKED`]).

use biguint::Bits;
use typenum::Unsigned;

use crate::field::Modulus;
use crate::kernel::{equal, fits, is_zero, limb, low_limb, set, shifted_limb, widening_mul};
use crate::limb::{SignedPowers, adc, mac, sbb, select};
use crate::params::{ExtendedLimbs, Limbs, Shape, WideLimbs};
use crate::unrolled::{UNROLLED, unroll};
use crate::width::Width;

impl<M: Modulus> Shape<M> {
    /// Whether the variable-time sums and differences of several limbs branch on their correction.
    /// On WebAssembly the borrow chain of the correction and the selects of a branch-free one cost
    /// more than a branch: the 254-bit sum took 24.7 cycles branch-free against 16.0 with the
    /// branch in Chrome. Elsewhere the flags of the carry chain feed conditional moves.
    const BRANCHES: bool = cfg!(target_arch = "wasm32");

    /// Whether Montgomery products of [`LOOPED_LIMBS`] limbs or more run the loops of
    /// [`looped`]. On WebAssembly without wide arithmetic the unrolled kernels of such widths
    /// outgrow the registers that the browsers allocate: a 381-bit product took Firefox 1,637
    /// cycles unrolled against 923 in loops, and Chrome 880 against 726. With wide arithmetic the
    /// unrolled interleaved product runs fastest: 190 cycles in Chrome against 248 in loops.
    const LOOPED: bool = cfg!(target_arch = "wasm32")
        && !cfg!(wide_arithmetic)
        && Self::MONTGOMERY
        && Self::LIMBS >= LOOPED_LIMBS;

    /// `a + b`, in constant time for `CT`.
    #[inline(always)]
    pub(crate) fn add<const CT: bool>(a: &Limbs<M>, b: &Limbs<M>) -> Limbs<M> {
        if Self::LIMBS == 1 {
            return Self::single(Self::add1(limb(a, 0), limb(b, 0)));
        }
        let (sum, carry) = Self::add_limbs(a, b);
        if !CT && Self::BRANCHES {
            // A sum at or above `p` loses `p`.
            return if (Self::SPARE || carry == 0) && Self::below_p_vartime(&sum) {
                sum
            } else {
                Self::sub_limbs(&sum, &Self::P).0
            };
        }
        Self::reduce_select(sum, if Self::SPARE { 0 } else { carry })
    }

    /// Whether the integer `value` lies below `p`, by its limbs from the top down to the first that
    /// differs from `p`'s.
    #[inline(always)]
    fn below_p_vartime(value: &Limbs<M>) -> bool {
        let mut index = Self::LIMBS;
        while index > 0 {
            index -= 1;
            let (limb, bound) = (limb(value, index), limb(&Self::P, index));
            if limb != bound {
                return limb < bound;
            }
        }
        false
    }

    /// `a - b`, in constant time for `CT`.
    #[inline(always)]
    pub(crate) fn sub<const CT: bool>(a: &Limbs<M>, b: &Limbs<M>) -> Limbs<M> {
        if Self::LIMBS == 1 {
            return Self::single(Self::sub1(limb(a, 0), limb(b, 0)));
        }
        let (difference, borrow) = Self::sub_limbs(a, b);
        if !CT && Self::BRANCHES {
            // A difference that borrowed gains `p`.
            return if borrow == 0 {
                difference
            } else {
                Self::add_limbs(&difference, &Self::P).0
            };
        }
        let spread = 0u64.wrapping_sub(u64::from(borrow));
        if Self::SUB_MASKED {
            return Self::add_masked(difference, spread);
        }
        let (wrapped, carry) = Self::add_limbs(&difference, &Self::P);
        // On `L + 1` limbs the difference carries `-borrow` on top: adding `p` carries out of
        // the top exactly when the difference borrowed. With a spare bit, the carry out of the
        // limbs alone tells.
        let wrapped_out = if Self::SPARE {
            carry
        } else {
            adc(carry, spread, 0).1
        };
        Self::select_limbs(wrapped_out != 0, &wrapped, &difference)
    }

    /// `-a`.
    #[inline(always)]
    pub(crate) fn neg(a: &Limbs<M>) -> Limbs<M> {
        if Self::LIMBS == 1 {
            return Self::single(Self::neg1(limb(a, 0)));
        }
        Self::sub::<true>(&M::Width::ZERO, a)
    }

    /// `a * b`. Builds with debug assertions call it: inlined, it would copy every limb step
    /// into each operation of the caller.
    #[cfg_attr(debug_assertions, inline(never))]
    #[cfg_attr(not(debug_assertions), inline(always))]
    pub(crate) fn mul<const CT: bool>(a: &Limbs<M>, b: &Limbs<M>) -> Limbs<M> {
        if Self::LIMBS == 1 {
            return Self::single(Self::mul1::<CT>(limb(a, 0), limb(b, 0)));
        }
        if Self::PSEUDO_MERSENNE {
            return Self::fold::<CT>(&Self::product(a, b));
        }
        if Self::LOOPED {
            let mut out = M::Width::ZERO;
            let (p, inv, spare) = (&Self::P, Self::INV, Self::SPARE);
            let mut wide = M::Width::WIDE_ZERO;
            looped::sos(out.as_mut_slice(), wide.as_mut_slice(), a, b, p, inv, spare);
            return out;
        }
        if Self::SEPARATED {
            Self::montgomery_reduce::<CT, true>(Self::product(a, b))
        } else {
            Self::montgomery_interleaved::<CT>(a, b)
        }
    }

    /// `a * a`, called as [`Self::mul`] is.
    #[cfg_attr(debug_assertions, inline(never))]
    #[cfg_attr(not(debug_assertions), inline(always))]
    pub(crate) fn square<const CT: bool>(a: &Limbs<M>) -> Limbs<M> {
        if Self::LIMBS == 1 {
            let value = limb(a, 0);
            return Self::single(Self::mul1::<CT>(value, value));
        }
        if cfg!(wide_arithmetic) && Self::LIMBS > 4 {
            // The product of `a` by itself: 180 cycles over BLS12-381 in Chrome against 280 for
            // the symmetric square.
            return Self::mul::<CT>(a, a);
        }
        let mut wide = Self::square_product(a);
        if Self::LOOPED {
            let mut out = M::Width::ZERO;
            let (p, inv, spare) = (&Self::P, Self::INV, Self::SPARE);
            looped::redc(out.as_mut_slice(), wide.as_mut_slice(), p, inv, spare);
            return out;
        }
        if Self::PSEUDO_MERSENNE {
            Self::fold::<CT>(&wide)
        } else {
            Self::montgomery_reduce::<CT, false>(wide)
        }
    }

    /// Whether the integer `value` lies below `p`, by the borrow of `value - p`.
    #[inline(always)]
    pub(crate) fn below_p(value: &Limbs<M>) -> bool {
        Self::sub_limbs(value, &Self::P).1 != 0
    }

    /// The representation of the integer `value < p`, in constant time: its Montgomery form by a
    /// product with `R^2`, `p` for `0` in a Mersenne field, else the integer.
    #[inline(always)]
    pub(crate) fn representation_of(value: &Limbs<M>) -> Limbs<M> {
        if Self::MONTGOMERY {
            Self::mul::<true>(value, &Self::R2)
        } else if Self::MERSENNE {
            Self::select_limbs(is_zero(value), &Self::P, value)
        } else {
            *value
        }
    }

    /// The integer below `p` that the representation `value` holds, in constant time: a
    /// Montgomery product by the integer `1`, `0` for `p` in a Mersenne field, else the
    /// representation.
    #[inline(always)]
    pub(crate) fn integer_of(value: &Limbs<M>) -> Limbs<M> {
        if Self::MONTGOMERY {
            Self::mul::<true>(value, &Self::UNIT)
        } else if Self::MERSENNE {
            Self::select_limbs(equal(value, &Self::P), &M::Width::ZERO, value)
        } else {
            *value
        }
    }

    /// The integer `1`.
    const UNIT: Limbs<M> = {
        let mut out = M::Width::ZERO;
        set(out.as_mut_slice(), 0, 1);
        out
    };

    /// Montgomery products reduce the full product row by row, where the modulus folds its
    /// reduction rows: `-p^-1 = 1 mod 2^64` makes each row's multiplier a limb of the product.
    pub(crate) const SEPARATED: bool = Self::INV == 1;

    /// Whether the one-chain reduction of a full product multiplies by each limb of `p` of three
    /// signed powers of two by shifts, additions and subtractions, which leaves the multiplier to
    /// the `L^2` products of the operands: P-256's product takes 16 multiplications instead of 20, and
    /// measured 52.5 cycles of latency against 59.8 at the same throughput. A square, of
    /// `L (L + 1) / 2` products, measured 3.8 cycles more latency this way, and the interleaved
    /// product of Tom-256 2.5 cycles more throughput; with the limbs of one or two terms as well,
    /// which LLVM folds into the sums around them, the P-256 product measured 6 cycles more
    /// throughput. WebAssembly keeps its multiplications.
    const P_BY_SHIFTS: bool = !cfg!(target_arch = "wasm32");

    /// The forms of the limbs of `p` that [`unroll!`] copies; a limb past them, or of more than
    /// three terms, is `None`.
    #[expect(
        clippy::indexing_slicing,
        reason = "const evaluation, with every index below the length"
    )]
    const P_FORMS: [Option<SignedPowers>; UNROLLED] = {
        let mut forms = [None; UNROLLED];
        let mut j = 0;
        while j < UNROLLED && j < Self::LIMBS {
            forms[j] = SignedPowers::of(limb(Self::P.as_slice(), j));
            j += 1;
        }
        forms
    };

    /// `m p_j + c + d` as `(low, high)`, for the limb `p_j` of `p`, by shifts where `SHIFTS` and
    /// [`Self::P_BY_SHIFTS`] hold and the limb has a form of three terms.
    #[inline(always)]
    fn mac_p<const SHIFTS: bool>(m: u64, j: usize, c: u64, d: u64) -> (u64, u64) {
        if SHIFTS
            && Self::P_BY_SHIFTS
            && let Some(Some(form)) = Self::P_FORMS.get(j)
            && form.terms() == 3
        {
            let (low, high) = form.mul(m);
            let (low, first) = adc(0, low, c);
            let (low, second) = adc(0, low, d);
            return (low, high + u64::from(first) + u64::from(second));
        }
        mac(m, limb(Self::P.as_slice(), j), c, d)
    }

    /// Runs `body` on each limb index of an element, as [`Self::each`] runs indices.
    #[inline(always)]
    pub(crate) fn each_limb(body: impl FnMut(usize)) {
        Self::each::<<M::Width as Bits>::Limbs>(body);
    }

    /// [`Self::each_limb`] over the `2 L` limbs of a full product.
    #[inline(always)]
    fn each_wide(body: impl FnMut(usize)) {
        Self::each::<<M::Width as Width>::Wide>(body);
    }

    /// [`Self::each_limb`] over the `L + 1` limbs of a partly folded product.
    #[inline(always)]
    pub(crate) fn each_extended(body: impl FnMut(usize)) {
        Self::each::<<M::Width as Width>::Extended>(body);
    }

    /// Runs `body` on each pair `(i, j)` of limb indices, row `i` by row `i`, as one run over the
    /// `L^2` pairs, so that each call knows both indices. A loop nested in a loop body inlines
    /// into that body while `i` is unknown, where LLVM reassociates `a b + c + d` into
    /// `(c + d) + a b`: a second carry chain, held in a register.
    #[inline(always)]
    fn each_pair(mut body: impl FnMut(usize, usize)) {
        Self::each::<<M::Width as Width>::Pairs>(
            #[inline(always)]
            |pair| body(pair / Self::LIMBS, pair % Self::LIMBS),
        );
    }

    /// Runs `body` on each index below `N`, in order. For fields of up to 16 limbs, builds without
    /// debug assertions call it at a literal index for each index below 16, where rustc inlines it
    /// at a constant, and loop past 16; wider fields and builds with debug assertions loop. The
    /// calls stay in this crate: a typed recursion in `biguint`, or a copy of it here, measured up
    /// to 30% more cycles.
    #[inline(always)]
    fn each<N: Unsigned>(mut body: impl FnMut(usize)) {
        let len = N::USIZE;
        #[cfg(not(debug_assertions))]
        if Self::LIMBS <= 16 {
            if len > 0 {
                body(0);
            }
            if len > 1 {
                body(1);
            }
            if len > 2 {
                body(2);
            }
            if len > 3 {
                body(3);
            }
            if len > 4 {
                body(4);
            }
            if len > 5 {
                body(5);
            }
            if len > 6 {
                body(6);
            }
            if len > 7 {
                body(7);
            }
            if len > 8 {
                body(8);
            }
            if len > 9 {
                body(9);
            }
            if len > 10 {
                body(10);
            }
            if len > 11 {
                body(11);
            }
            if len > 12 {
                body(12);
            }
            if len > 13 {
                body(13);
            }
            if len > 14 {
                body(14);
            }
            if len > 15 {
                body(15);
            }
            for index in 16..len {
                body(index);
            }
            return;
        }
        for index in 0..len {
            body(index);
        }
    }

    // One limb.

    /// The representation whose low limb is `value`.
    #[inline(always)]
    fn single(value: u64) -> Limbs<M> {
        let mut out = M::Width::ZERO;
        set(out.as_mut_slice(), 0, value);
        out
    }

    /// `a + b` on one limb.
    #[inline(always)]
    fn add1(a: u64, b: u64) -> u64 {
        if Self::MERSENNE {
            Self::mersenne_fold(a, b)
        } else {
            Self::add_residues1(a, b)
        }
    }

    /// `a + b mod p` for residues `a, b < p` on one limb.
    #[inline(always)]
    pub(crate) fn add_residues1(a: u64, b: u64) -> u64 {
        if Self::SPARE {
            // `a + b < 2 p` fits the limb.
            let sum = a + b;
            let (reduced, borrow) = sum.overflowing_sub(Self::P0);
            select(borrow, sum, reduced)
        } else {
            // `a + b = a - (p - b)`, which borrows exactly when `a + b < p`.
            let (difference, borrow) = a.overflowing_sub(Self::P0 - b);
            select(borrow, difference.wrapping_add(Self::P0), difference)
        }
    }

    /// `a - b` on one limb.
    #[inline(always)]
    fn sub1(a: u64, b: u64) -> u64 {
        if Self::MERSENNE {
            // `a - b` lies in `[1 - p, p - 1]`, and the result in `[1, p]`: `p` comes on when
            // `a <= b`, which the flags of the subtraction tell.
            let difference = a.wrapping_sub(b);
            select(a <= b, difference.wrapping_add(Self::P0), difference)
        } else {
            let (difference, borrow) = a.overflowing_sub(b);
            select(borrow, difference.wrapping_add(Self::P0), difference)
        }
    }

    /// `-a` on one limb.
    #[inline(always)]
    fn neg1(a: u64) -> u64 {
        if Self::MERSENNE {
            // `0` is held as `p`, and stays.
            select(a == Self::P0, a, Self::P0.wrapping_sub(a))
        } else {
            select(a == 0, a, Self::P0.wrapping_sub(a))
        }
    }

    /// `a * b` on one limb.
    #[inline(always)]
    pub(crate) fn mul1<const CT: bool>(a: u64, b: u64) -> u64 {
        if Self::MERSENNE {
            // `x = high 2^BITS + low` with `2^BITS = 1`: `high + low`, at least `1` since the
            // factors are, and below `2^(BITS + 1) - 2`.
            let (low, high) = if Self::PRODUCT_FITS_LIMB {
                let product = a * b;
                (product & Self::P0, product >> Self::BITS)
            } else {
                let product = widening_mul(a, b);
                (product as u64 & Self::P0, (product >> Self::BITS) as u64)
            };
            Self::mersenne_fold(low, high)
        } else {
            Self::montgomery1::<CT>(a, b)
        }
    }

    /// `x + y` reduced into `[1, p]`, for a Mersenne modulus and `2 <= x + y <= 2^(BITS + 1) -
    /// 2`: a sum above `p` is one at or above `2^BITS`. At 31 and 63 bits the sum's signed
    /// overflow flags it.
    #[inline(always)]
    fn mersenne_fold(x: u64, y: u64) -> u64 {
        if Self::BITS == 31 {
            // On 32 bits, so that one addition yields both the sum and its flag.
            let (sum, above) = (x as i32).overflowing_add(y as i32);
            let sum = sum as u32;
            u64::from(select(above, sum.wrapping_sub(Self::P0 as u32), sum))
        } else if Self::BITS == 63 {
            let (sum, above) = (x as i64).overflowing_add(y as i64);
            let sum = sum as u64;
            select(above, sum.wrapping_sub(Self::P0), sum)
        } else {
            let sum = x + y;
            select(sum >> Self::BITS != 0, sum.wrapping_sub(Self::P0), sum)
        }
    }

    /// The Montgomery product `a b / 2^64 mod p` on one limb.
    #[inline(always)]
    pub(crate) fn montgomery1<const CT: bool>(a: u64, b: u64) -> u64 {
        if Self::PRODUCT_FITS_LIMB {
            // `x + m p = 0 mod 2^64` with `x < 2^64`: the low limbs carry exactly when `x` is
            // not zero, and the result `(m p >> 64) + carry` is at most `p`. It equals `p` only
            // when `x` is a nonzero multiple of `p`, which a prime modulus rules out.
            let product = a * b;
            let m = product.wrapping_mul(Self::INV);
            let high = (widening_mul(m, Self::P0) >> 64) as u64;
            let result = high + u64::from(product != 0);
            if Self::PRIME_LIMB {
                result
            } else {
                select(result == Self::P0, 0, result)
            }
        } else if Self::SHIFT_REDC {
            // With `p^-1 = 1 + 2^32 mod 2^64`, `m = low p^-1` is `low + (low << 32)`, and
            // `(m p) >> 64 = m - (m >> 32) - e` for the carry `e` of that sum, as in winter-math's
            // `mont_red_cst`. `m >> 32` is the 32-bit sum of the halves of `low`, which does not
            // wait for `m`. The result `high - (m p >> 64)` lies in `(-p, p)`.
            let product = widening_mul(a, b);
            let (low, high) = (product as u64, (product >> 64) as u64);
            let (m, e) = adc(0, low, low << 32);
            let shifted = u64::from(((low >> 32) as u32).wrapping_add(low as u32));
            let (quotient, _) = sbb(e, m, shifted);
            let (result, borrow) = sbb(0, high, quotient);
            select(borrow != 0, result.wrapping_add(Self::P0), result)
        } else {
            Self::redc1::<CT>(widening_mul(a, b))
        }
    }

    /// Montgomery reduction `value / 2^64 mod p` of `value < p 2^64` on one limb.
    #[inline(always)]
    pub(crate) fn redc1<const CT: bool>(value: u128) -> u64 {
        let (low, high) = (value as u64, (value >> 64) as u64);
        let m = low.wrapping_mul(Self::INV);
        let reduction = (widening_mul(m, Self::P0) >> 64) as u64;
        let (result, carry) = adc(u8::from(low != 0), high, reduction);
        Self::reduce1::<CT>(result, carry)
    }

    /// One limb with `p^-1 = 1 + 2^32 mod 2^64`, that is `p = 2^64 - 2^32 + 1`: the Montgomery
    /// multiplier is a shift and an addition, and the multiple of `p` its shift and subtractions.
    pub(crate) const SHIFT_REDC: bool =
        Self::LIMBS == 1 && Self::INV == (1u64 << 32).wrapping_add(1).wrapping_neg();

    /// `carry 2^64 + t mod p` for `carry 2^64 + t < 2 p`, on one limb.
    #[inline(always)]
    pub(crate) fn reduce1<const CT: bool>(t: u64, carry: u8) -> u64 {
        let (reduced, borrow) = sbb(0, t, Self::P0);
        select(borrow > carry, t, reduced)
    }

    // Several limbs.

    /// `a + b` over the limbs; the carry out of the top limb.
    #[inline(always)]
    fn add_limbs(a: &[u64], b: &[u64]) -> (Limbs<M>, u8) {
        let mut out = M::Width::ZERO;
        let mut carry = 0;
        Self::each_limb(
            #[inline(always)]
            |j| {
                let (sum, carry_out) = adc(carry, limb(a, j), limb(b, j));
                set(out.as_mut_slice(), j, sum);
                carry = carry_out;
            },
        );
        (out, carry)
    }

    /// `a - b` over the limbs; the borrow out of the top limb.
    #[inline(always)]
    fn sub_limbs(a: &[u64], b: &[u64]) -> (Limbs<M>, u8) {
        let mut out = M::Width::ZERO;
        let mut borrow = 0;
        Self::each_limb(
            #[inline(always)]
            |j| {
                let (difference, borrow_out) = sbb(borrow, limb(a, j), limb(b, j));
                set(out.as_mut_slice(), j, difference);
                borrow = borrow_out;
            },
        );
        (out, borrow)
    }

    /// `t + (p & mask)` modulo `2^(64 L)`. The limbs of `p` that are zero or all ones fold the
    /// mask away.
    #[inline(always)]
    fn add_masked(mut t: Limbs<M>, mask: u64) -> Limbs<M> {
        let mut carry = 0;
        Self::each_limb(
            #[inline(always)]
            |j| {
                let (sum, carry_out) = adc(
                    carry,
                    limb(t.as_slice(), j),
                    limb(Self::P.as_slice(), j) & mask,
                );
                set(t.as_mut_slice(), j, sum);
                carry = carry_out;
            },
        );
        t
    }

    /// `a` when `condition` holds, else `b`, limb by limb with conditional moves.
    #[inline(always)]
    pub(crate) fn select_limbs(condition: bool, a: &Limbs<M>, b: &Limbs<M>) -> Limbs<M> {
        let mut out = M::Width::ZERO;
        Self::each_limb(
            #[inline(always)]
            |j| {
                set(
                    out.as_mut_slice(),
                    j,
                    select(condition, limb(a.as_slice(), j), limb(b.as_slice(), j)),
                );
            },
        );
        out
    }

    /// `top 2^(64 L) + t mod p` for a value below `2 p`, without a branch: `t - p`, then `t` back
    /// by conditional moves, or `p` back under a mask when [`Self::REDUCE_MASKED`] holds.
    #[inline(always)]
    pub(crate) fn reduce_select(t: Limbs<M>, top: u8) -> Limbs<M> {
        // `-top`, taken while a carry out of a sum still stands in the flags.
        let spread = 0u64.wrapping_sub(u64::from(top));
        let (reduced, borrow) = Self::sub_limbs(&t, &Self::P);
        if Self::REDUCE_MASKED {
            // `top 2^(64 L) + t - p` borrows exactly when the value stays below `p`.
            let below = if Self::SPARE {
                borrow
            } else {
                sbb(borrow, u64::from(top), 0).1
            };
            return Self::add_masked(reduced, 0u64.wrapping_sub(u64::from(below)));
        }
        // `borrow - top`: `1` exactly when the value stays below `p`, since a set top implies a
        // borrow.
        let keep = if Self::SPARE {
            u64::from(borrow)
        } else {
            adc(borrow, spread, 0).0
        };
        Self::select_limbs(keep != 0, &t, &reduced)
    }

    /// The final reduction of a product, `top 2^(64 L) + t mod p` for a value below `2 p`: by a
    /// branch on the top limb in the variable-time form when [`Self::FINAL_BRANCH`] holds.
    #[inline(always)]
    fn reduce_final<const CT: bool>(t: Limbs<M>, top: u8) -> Limbs<M> {
        if CT || !Self::FINAL_BRANCH {
            return Self::reduce_select(t, top);
        }
        // [`Self::reduce_branch`], written out: a call to it moves the register allocation of
        // the BN254 square root.
        if top != 0 || limb(t.as_slice(), Self::LIMBS - 1) >= Self::P_TOP {
            let (reduced, borrow) = Self::sub_limbs(&t, &Self::P);
            return if borrow > top {
                Self::add_masked(reduced, u64::MAX)
            } else {
                reduced
            };
        }
        t
    }

    /// `top 2^(64 L) + t mod p` for a value below `2 p`, by a branch on the top limb, for values
    /// that rarely reach `p`.
    #[inline(always)]
    pub(crate) fn reduce_branch(t: Limbs<M>, top: u8) -> Limbs<M> {
        if top != 0 || limb(t.as_slice(), Self::LIMBS - 1) >= Self::P_TOP {
            // Below `p` only when the top limbs are equal; the top absorbs the borrow of a value at
            // or above `2^(64 L)`.
            let (reduced, borrow) = Self::sub_limbs(&t, &Self::P);
            return if borrow > top {
                Self::add_masked(reduced, u64::MAX)
            } else {
                reduced
            };
        }
        t
    }

    /// The instructions that masking limb `value` of `p` costs: none for zero and all ones,
    /// a 32-bit move for `2^32 - 1`, a move and an `and` otherwise.
    const fn masked_limb_cost(value: u64) -> usize {
        match value {
            0 | u64::MAX => 0,
            0xffff_ffff => 1,
            _ => 2,
        }
    }

    /// The instructions that masking `p` costs.
    const MASK_COST: usize = {
        let mut cost = 0;
        let mut j = 0;
        while j < Self::LIMBS {
            cost += Self::masked_limb_cost(limb(Self::P.as_slice(), j));
            j += 1;
        }
        cost
    };

    /// The final subtraction adds `p` back under a mask where that costs fewer instructions than
    /// the conditional moves: the masked form adds a carry chain, the mask, the masked limbs of
    /// `p`, and without a spare bit a carry kept in a register (`setb`, `movzx`, `sbb`); the
    /// conditional form copies and moves each limb, and without a spare bit keeps the carry by
    /// `sbb` and `adc`. Only moduli whose limbs are mostly all ones gain (P-521).
    pub(crate) const REDUCE_MASKED: bool =
        Self::LIMBS + Self::MASK_COST + 1 + if Self::SPARE { 0 } else { 3 }
            < 2 * Self::LIMBS + if Self::SPARE { 0 } else { 2 };

    /// A difference adds `p` under the mask of its borrow where masking `p` costs fewer
    /// instructions than the conditional form's copy, carry chain and conditional move per limb,
    /// with its `sbb` and `adc` that keep the borrow without a spare bit, and beyond four limbs,
    /// where the conditional form's `3 L` live values (both chains and the limbs of `p`) spill:
    /// on BLS12-381 the masked form measures half a cycle to a cycle below it in throughput and
    /// latency. Only BN254, of arbitrary limbs with a spare bit, keeps the conditional form.
    pub(crate) const SUB_MASKED: bool =
        Self::MASK_COST + 1 < 2 * Self::LIMBS + if Self::SPARE { 0 } else { 2 } || Self::LIMBS > 4;

    /// A product's final subtraction is a branch in the variable-time form: a Montgomery product
    /// reaches `p` with probability about `p / 4R` (Walter, *Montgomery exponentiation needs no
    /// final subtractions*, 1999), at most a sixteenth when `p < R / 4`.
    pub(crate) const FINAL_BRANCH: bool =
        Self::MONTGOMERY && Self::LIMBS > 1 && Self::P_TOP >> 62 == 0;

    /// Beyond four limbs, a row of products adds the low halves of its products in one carry
    /// chain and the high halves, one limb up, in a second ([`Self::add_row`]): a BLS12-381
    /// product measures 13% fewer cycles than with one chain of multiply-accumulates, a P-521
    /// product 5%. On four limbs the one chain takes fewer cycles, as the constant limbs of `p`
    /// fold into its additions: a P-256 product takes 25% more with two. WebAssembly runs one chain:
    /// the second costs a comparison per carry, and a P-521 product took Chrome 1,023 cycles with
    /// two against 668 with one. Its symmetric squares keep two, [`Self::square_product`].
    pub(crate) const TWO_CHAINS: bool = Self::LIMBS > 4 && !cfg!(target_arch = "wasm32");

    /// The full product `a b` in `2 L` limbs, by rows of one limb of `b`.
    #[inline(always)]
    fn product(a: &[u64], b: &[u64]) -> WideLimbs<M> {
        let mut wide = M::Width::WIDE_ZERO;
        unroll!(i in 0, Self::LIMBS => {
            let multiplier = limb(b, i);
            if Self::TWO_CHAINS {
                let product = widening_mul(limb(a, 0), multiplier);
                let (low, first) = adc(0, limb(wide.as_slice(), i), product as u64);
                set(wide.as_mut_slice(), i, low);
                // Limb `i + L` is zero before the row, and the row's sum fits through it.
                let (top, carries) =
                    Self::add_row(wide.as_mut_slice(), i, multiplier, a, first, 0);
                set(wide.as_mut_slice(), i + Self::LIMBS, top);
                fits(carries);
            } else {
                let mut carry = 0;
                unroll!(j in 0, Self::LIMBS => {
                    let (low, high) =
                        mac(limb(a, j), multiplier, limb(wide.as_slice(), i + j), carry);
                    set(wide.as_mut_slice(), i + j, low);
                    carry = high;
                });
                set(wide.as_mut_slice(), i + Self::LIMBS, carry);
            }
        });
        wide
    }

    /// `acc[at..at + L] + top 2^(64 (at + L)) += x v` for the caller that has added the low half
    /// of `x v[0]` at limb `at`, with the carry `first`: the low halves of the other products add
    /// in one carry chain and the high halves, one limb up, in a second. Returns the new limb
    /// `at + L` and the carries out of it.
    #[inline(always)]
    fn add_row(acc: &mut [u64], at: usize, x: u64, v: &[u64], first: u8, top: u64) -> (u64, u64) {
        let mut highs = M::Width::ZERO;
        set(
            highs.as_mut_slice(),
            0,
            (widening_mul(x, limb(v, 0)) >> 64) as u64,
        );
        let mut carry = first;
        unroll!(j in 1, Self::LIMBS => {
            let product = widening_mul(x, limb(v, j));
            let (sum, carry_out) = adc(carry, limb(acc, at + j), product as u64);
            set(acc, at + j, sum);
            set(highs.as_mut_slice(), j, (product >> 64) as u64);
            carry = carry_out;
        });
        let (top, low_carry) = adc(carry, limb(acc, at + Self::LIMBS), top);
        let mut carry = 0;
        unroll!(j in 1, Self::LIMBS => {
            let (sum, carry_out) = adc(
                carry,
                limb(acc, at + j),
                limb(highs.as_slice(), j.wrapping_sub(1)),
            );
            set(acc, at + j, sum);
            carry = carry_out;
        });
        let (top, high_carry) = adc(carry, top, limb(highs.as_slice(), Self::LIMBS - 1));
        (top, u64::from(low_carry) + u64::from(high_carry))
    }

    /// The full square `a a` in `2 L` limbs: each cross product once, doubled, plus the squares
    /// on the diagonal. Beyond four limbs its rows run two carry chains on every target: on
    /// WebAssembly a 381-bit square took Chrome 580 cycles this way against 700 with one.
    #[inline(always)]
    fn square_product(a: &[u64]) -> WideLimbs<M> {
        let mut wide = M::Width::WIDE_ZERO;
        // Limb `i + L` is zero before row `i`, and the row's sum fits through it.
        if Self::LIMBS > 4 {
            Self::each_limb(
                #[inline(always)]
                |i| {
                    if i + 1 < Self::LIMBS {
                        let multiplier = limb(a, i);
                        let mut highs = M::Width::ZERO;
                        let mut carry = 0;
                        Self::each_limb(
                            #[inline(always)]
                            |j| {
                                if j > i {
                                    let product = widening_mul(limb(a, j), multiplier);
                                    let (sum, carry_out) =
                                        adc(carry, limb(wide.as_slice(), i + j), product as u64);
                                    set(wide.as_mut_slice(), i + j, sum);
                                    set(highs.as_mut_slice(), j, (product >> 64) as u64);
                                    carry = carry_out;
                                }
                            },
                        );
                        set(wide.as_mut_slice(), i + Self::LIMBS, u64::from(carry));
                        let mut carry = 0;
                        Self::each_limb(
                            #[inline(always)]
                            |j| {
                                if j > i {
                                    let (sum, carry_out) = adc(
                                        carry,
                                        limb(wide.as_slice(), i + j + 1),
                                        limb(highs.as_slice(), j),
                                    );
                                    set(wide.as_mut_slice(), i + j + 1, sum);
                                    carry = carry_out;
                                }
                            },
                        );
                        fits(u64::from(carry));
                    }
                },
            );
        } else {
            let mut carry = 0;
            Self::each_pair(
                #[inline(always)]
                |i, j| {
                    if j > i {
                        let (low, high) =
                            mac(limb(a, j), limb(a, i), limb(wide.as_slice(), i + j), carry);
                        set(wide.as_mut_slice(), i + j, low);
                        carry = high;
                        if j + 1 == Self::LIMBS {
                            set(wide.as_mut_slice(), i + Self::LIMBS, carry);
                            carry = 0;
                        }
                    }
                },
            );
        }
        // The cross products sum below `2^(128 L - 1)`: doubling loses no bit.
        let mut carry = 0;
        Self::each_wide(
            #[inline(always)]
            |k| {
                if k >= 1 {
                    let value = limb(wide.as_slice(), k);
                    let (doubled, carry_out) = adc(carry, value, value);
                    set(wide.as_mut_slice(), k, doubled);
                    carry = carry_out;
                }
            },
        );
        let mut carry = 0;
        Self::each_limb(
            #[inline(always)]
            |i| {
                let value = limb(a, i);
                let square = widening_mul(value, value);
                let (low, carry_low) = adc(carry, limb(wide.as_slice(), 2 * i), square as u64);
                let (high, carry_high) = adc(
                    carry_low,
                    limb(wide.as_slice(), 2 * i + 1),
                    (square >> 64) as u64,
                );
                set(wide.as_mut_slice(), 2 * i, low);
                set(wide.as_mut_slice(), 2 * i + 1, high);
                carry = carry_high;
            },
        );
        wide
    }

    /// Montgomery reduction of the full product `wide < p 2^(64 L)`, row by row: `wide / 2^(64
    /// L) mod p`.
    #[inline(always)]
    fn montgomery_reduce<const CT: bool, const SHIFTS: bool>(mut wide: WideLimbs<M>) -> Limbs<M> {
        if Self::TWO_CHAINS {
            return Self::montgomery_reduce_two_chains::<CT>(wide);
        }
        let mut above = 0;
        let mut m = 0;
        let mut carry = 0;
        Self::each_pair(
            #[inline(always)]
            |i, j| {
                if j == 0 {
                    m = limb(wide.as_slice(), i).wrapping_mul(Self::INV);
                    // The low half cancels limb `i`: LLVM folds the product by `p0` into the sum.
                    carry = mac(m, Self::P0, limb(wide.as_slice(), i), 0).1;
                } else {
                    let (low, high) =
                        Self::mac_p::<SHIFTS>(m, j, limb(wide.as_slice(), i + j), carry);
                    set(wide.as_mut_slice(), i + j, low);
                    carry = high;
                }
                if j + 1 == Self::LIMBS {
                    // The carry out of the row's top limb lands where the next row adds its own.
                    let (sum, carry_out) =
                        adc(above, limb(wide.as_slice(), i + Self::LIMBS), carry);
                    set(wide.as_mut_slice(), i + Self::LIMBS, sum);
                    above = carry_out;
                }
            },
        );
        let mut out = M::Width::ZERO;
        Self::each_limb(
            #[inline(always)]
            |j| {
                set(
                    out.as_mut_slice(),
                    j,
                    limb(wide.as_slice(), j + Self::LIMBS),
                );
            },
        );
        Self::reduce_final::<CT>(out, if Self::SPARE { 0 } else { above })
    }

    /// Montgomery multiplication by coarsely integrated operand scanning: each row adds `a`
    /// times one limb of `b`, then the multiple of `p` that clears the low limb, shifted out.
    /// `p < 2^(64 L - 1)` keeps the accumulator within `L + 1` limbs.
    #[inline(always)]
    fn montgomery_interleaved<const CT: bool>(a: &[u64], b: &[u64]) -> Limbs<M> {
        if Self::TWO_CHAINS {
            return Self::montgomery_interleaved_two_chains::<CT>(a, b);
        }
        let mut out = M::Width::ZERO;
        let mut top = 0u64;
        unroll!(i in 0, Self::LIMBS => {
            let multiplier = limb(b, i);
            let mut carry = 0;
            unroll!(j in 0, Self::LIMBS => {
                let (low, high) = mac(limb(a, j), multiplier, limb(out.as_slice(), j), carry);
                set(out.as_mut_slice(), j, low);
                carry = high;
            });
            let (row_top, above) = adc(0, top, carry);
            let m = limb(out.as_slice(), 0).wrapping_mul(Self::INV);
            let (_, mut carry) = mac(m, Self::P0, limb(out.as_slice(), 0), 0);
            unroll!(j in 1, Self::LIMBS => {
                let (low, high) =
                    mac(m, limb(Self::P.as_slice(), j), limb(out.as_slice(), j), carry);
                set(out.as_mut_slice(), j.wrapping_sub(1), low);
                carry = high;
            });
            let (sum, carry_out) = adc(0, row_top, carry);
            set(out.as_mut_slice(), Self::LIMBS - 1, sum);
            top = if Self::SPARE {
                0
            } else {
                u64::from(above) + u64::from(carry_out)
            };
        });
        Self::reduce_final::<CT>(out, top as u8)
    }

    /// [`Self::montgomery_reduce`] by rows of [`Self::add_row`].
    #[inline(always)]
    fn montgomery_reduce_two_chains<const CT: bool>(mut wide: WideLimbs<M>) -> Limbs<M> {
        // The carries out of a row's top limb, which the next row adds one limb up.
        let mut above = 0u64;
        unroll!(i in 0, Self::LIMBS => {
            let low = limb(wide.as_slice(), i);
            let m = low.wrapping_mul(Self::INV);
            // `low` plus the low half of `m p0` is a multiple of `2^64`: it carries unless `low`
            // is zero.
            let (top, carries) = Self::add_row(
                wide.as_mut_slice(),
                i,
                m,
                Self::P.as_slice(),
                u8::from(low != 0),
                above,
            );
            set(wide.as_mut_slice(), i + Self::LIMBS, top);
            above = carries;
        });
        let mut out = M::Width::ZERO;
        unroll!(j in 0, Self::LIMBS => {
            set(out.as_mut_slice(), j, limb(wide.as_slice(), j + Self::LIMBS));
        });
        Self::reduce_final::<CT>(out, if Self::SPARE { 0 } else { above as u8 })
    }

    /// [`Self::montgomery_interleaved`] by rows of [`Self::add_row`].
    #[inline(always)]
    fn montgomery_interleaved_two_chains<const CT: bool>(a: &[u64], b: &[u64]) -> Limbs<M> {
        // The accumulator, `acc[..L]`, and the limb above it, `acc[L]`.
        let mut acc = M::Width::EXTENDED_ZERO;
        unroll!(i in 0, Self::LIMBS => {
            let multiplier = limb(b, i);
            let product = widening_mul(limb(a, 0), multiplier);
            let (low, first) = adc(0, limb(acc.as_slice(), 0), product as u64);
            set(acc.as_mut_slice(), 0, low);
            let (row_top, above) = Self::add_row(acc.as_mut_slice(), 0, multiplier, a, first, 0);
            set(acc.as_mut_slice(), Self::LIMBS, row_top);
            let low = limb(acc.as_slice(), 0);
            let m = low.wrapping_mul(Self::INV);
            let (row_top, carries) = Self::add_row(
                acc.as_mut_slice(),
                0,
                m,
                Self::P.as_slice(),
                u8::from(low != 0),
                0,
            );
            // The low limb is now zero: the accumulator shifts down one limb.
            unroll!(j in 0, Self::LIMBS - 1 => {
                let next = limb(acc.as_slice(), j + 1);
                set(acc.as_mut_slice(), j, next);
            });
            set(acc.as_mut_slice(), Self::LIMBS - 1, row_top);
            set(
                acc.as_mut_slice(),
                Self::LIMBS,
                if Self::SPARE { 0 } else { above + carries },
            );
        });
        let mut out = M::Width::ZERO;
        unroll!(j in 0, Self::LIMBS => {
            set(out.as_mut_slice(), j, limb(acc.as_slice(), j));
        });
        Self::reduce_final::<CT>(out, limb(acc.as_slice(), Self::LIMBS) as u8)
    }

    /// The full product `wide` reduced modulo the pseudo-Mersenne `p = 2^BITS - c` by
    /// [`Self::FOLDS`] folds `low + high c`, each over the limbs its bound needs, and one
    /// conditional subtraction.
    #[inline(always)]
    fn fold<const CT: bool>(wide: &WideLimbs<M>) -> Limbs<M> {
        let mut first = M::Width::EXTENDED_ZERO;
        let mut second = M::Width::EXTENDED_ZERO;
        Self::fold_once(&mut first, Self::fold_limbs(1), wide.as_slice());
        let mut step = 1;
        while step < Self::FOLDS {
            if step % 2 == 1 {
                Self::fold_once(&mut second, Self::fold_limbs(step + 1), first.as_slice());
            } else {
                Self::fold_once(&mut first, Self::fold_limbs(step + 1), second.as_slice());
            }
            step += 1;
        }
        let folded = if Self::FOLDS % 2 == 1 {
            &first
        } else {
            &second
        };
        let mut out = M::Width::ZERO;
        unroll!(j in 0, Self::LIMBS => {
            set(out.as_mut_slice(), j, limb(folded.as_slice(), j));
        });
        Self::reduce_final::<CT>(out, limb(folded.as_slice(), Self::LIMBS) as u8)
    }

    /// `out = (value mod 2^BITS) + (value >> BITS) c` over the first `len` limbs of `out`. For
    /// `c = 1` (P-521) the fold is a sum, on the carry chain of `adc`.
    #[inline(always)]
    pub(crate) fn fold_once(out: &mut ExtendedLimbs<M>, len: usize, value: &[u64]) {
        let deficit = Self::DEFICIT_VALUE;
        if deficit == 1 {
            let mut carry = 0;
            unroll!(j in 0, len => {
                let (sum, carry_out) = adc(
                    carry,
                    shifted_limb(value, Self::BITS, j),
                    low_limb(value, Self::BITS, j),
                );
                set(out.as_mut_slice(), j, sum);
                carry = carry_out;
            });
            return;
        }
        let mut carry = 0;
        unroll!(j in 0, len => {
            let (sum, high) = mac(
                shifted_limb(value, Self::BITS, j),
                deficit,
                low_limb(value, Self::BITS, j),
                carry,
            );
            set(out.as_mut_slice(), j, sum);
            carry = high;
        });
    }
}

/// The fewest limbs of a [`Shape::LOOPED`] product.
const LOOPED_LIMBS: usize = 6;

/// The Montgomery kernels as calls on slices, whose lengths the compiler does not know, so that
/// their loops stay loops.
mod looped {
    /// The product into `wide`, then its reduction into `out`, by
    /// [`kernel::mul_wide`](crate::kernel::mul_wide) and [`redc`]: the separated form runs fastest.
    #[inline(never)]
    pub(super) fn sos(
        out: &mut [u64],
        wide: &mut [u64],
        a: &[u64],
        b: &[u64],
        p: &[u64],
        inv: u64,
        spare: bool,
    ) {
        crate::kernel::mul_wide(wide, a, b);
        crate::kernel::redc(out, wide, p, inv, spare);
    }

    /// [`kernel::redc`](crate::kernel::redc), of the unrolled square: a square runs fastest with
    /// its symmetric product unrolled and its reduction looped.
    #[inline(never)]
    pub(super) fn redc(out: &mut [u64], wide: &mut [u64], p: &[u64], inv: u64, spare: bool) {
        crate::kernel::redc(out, wide, p, inv, spare);
    }
}
