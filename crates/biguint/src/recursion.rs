//! The recursion of the divide-and-conquer products and divisions, over lengths that types fix.
//!
//! A level is a depth, the halvings left, and a length, a type whose constant [`Len::LEN`] counts
//! the operand limbs: the limbs of a width, or a part that a split takes of a longer length. Each
//! Karatsuba product, truncated split and divide-and-conquer division is one function per depth
//! and length. Its lengths and its scratch length are constants inside it: its splits, its
//! kernels and its choice between recursion and the basecase fold. The basecase, the quadratic
//! product or the schoolbook division, runs below the threshold or when the scratch is too short
//! for the recursion.
//!
//! A level is a call, with three exceptions that inline: the first level, into the operation
//! that owns its buffers ([`Depth::entry_product`]); a Karatsuba or division level whose halves
//! run the basecase, into the level above it; and the basecases. A function then holds at most
//! two levels and the basecases under them.
//!
//! A level function takes slices and checks once that they hold its lengths, which makes those
//! lengths known to the compiler. Slices of other lengths run the basecase of their run-time
//! lengths. Builds with debug assertions erase the lengths ([`Width`]): their levels read the
//! lengths of their slices, and every level below the first is a call.

use core::marker::PhantomData;

use typenum::Unsigned;

use crate::div::dc;
use crate::div::reciprocal::Reciprocal3By2;
use crate::div::schoolbook;
use crate::mul::basecase;
use crate::mul::karatsuba;
use crate::mul::truncated;

/// A limb count fixed at compile time, or erased to the run-time length of the slice that holds
/// it.
pub(crate) trait Len {
    /// The limbs of a fixed length.
    const LEN: usize;
    /// Whether the length is erased.
    const ERASED: bool;

    /// The limbs of a slice of `actual` limbs that holds this length: `actual` when the length
    /// is erased, [`Self::LEN`] otherwise.
    #[inline(always)]
    fn of(actual: usize) -> usize {
        if Self::ERASED { actual } else { Self::LEN }
    }
}

/// The length type of an `A`-limb operand: [`Limbs<A>`]. Builds with debug assertions erase it,
/// so that one function per depth serves every width and their code stays small.
pub(crate) type Width<A> = <A as WidthOf>::Len;

/// Maps a `typenum` length to its [`Width`].
pub(crate) trait WidthOf {
    /// The length type.
    type Len: Len;
}

impl<A: Unsigned> WidthOf for A {
    #[cfg(not(debug_assertions))]
    type Len = Limbs<A>;
    #[cfg(debug_assertions)]
    type Len = Erased;
}

/// The limbs that the `typenum` length `A` counts.
#[cfg(not(debug_assertions))]
pub(crate) struct Limbs<A>(PhantomData<A>);

#[cfg(not(debug_assertions))]
impl<A: Unsigned> Len for Limbs<A> {
    const LEN: usize = A::USIZE;
    const ERASED: bool = false;
}

/// A length that the slice holding it carries.
#[cfg(debug_assertions)]
pub(crate) struct Erased;

#[cfg(debug_assertions)]
impl Len for Erased {
    const LEN: usize = 0;
    const ERASED: bool = true;
}

/// The larger half of `P`: `ceil(P / 2)`.
pub(crate) struct HalfUp<P>(PhantomData<P>);

impl<P: Len> Len for HalfUp<P> {
    const LEN: usize = P::LEN - P::LEN / 2;
    const ERASED: bool = P::ERASED;
}

/// The smaller half of `P`: `floor(P / 2)`.
pub(crate) struct HalfDown<P>(PhantomData<P>);

impl<P: Len> Len for HalfDown<P> {
    const LEN: usize = P::LEN / 2;
    const ERASED: bool = P::ERASED;
}

/// `P - Q`, or zero when `Q` is longer.
pub(crate) struct Difference<P, Q>(PhantomData<(P, Q)>);

impl<P: Len, Q: Len> Len for Difference<P, Q> {
    const LEN: usize = P::LEN.saturating_sub(Q::LEN);
    const ERASED: bool = P::ERASED || Q::ERASED;
}

/// `slice` when it holds exactly `len` limbs.
#[inline(always)]
pub(crate) fn exact(slice: &[u64], len: usize) -> Option<&[u64]> {
    if slice.len() == len {
        slice.get(..len)
    } else {
        None
    }
}

/// `slice` when it holds exactly `len` limbs.
#[inline(always)]
pub(crate) fn exact_mut(slice: &mut [u64], len: usize) -> Option<&mut [u64]> {
    if slice.len() == len {
        slice.get_mut(..len)
    } else {
        None
    }
}

/// The first `len` limbs of `slice`, when it holds as many.
#[inline(always)]
pub(crate) fn prefix_mut(slice: &mut [u64], len: usize) -> Option<&mut [u64]> {
    slice.get_mut(..len)
}

/// [`basecase`] at the run-time lengths of its slices: the product of a level whose slices miss
/// its lengths.
#[cold]
#[inline(never)]
pub(crate) fn basecase_of_slices(out: &mut [u64], a: &[u64], b: &[u64]) {
    basecase(out, a, b);
}

/// [`schoolbook`] at the run-time lengths of its slices: the division of a level whose slices
/// miss its lengths.
#[cold]
#[inline(never)]
pub(crate) fn schoolbook_of_slices(
    numerator: &mut [u64],
    divisor: &[u64],
    quotient: &mut [u64],
    leading: Reciprocal3By2,
) -> u64 {
    schoolbook(numerator, divisor, quotient, leading)
}

/// A recursion budget.
pub(crate) trait Depth {
    /// The halvings left.
    const LEVELS: usize;

    /// Writes `a * b` to `out[..2 * L]` for `a` and `b` of `L` limbs. `MASKED` runs every
    /// correction under masks: no branch and no memory address depends on the operands.
    fn product_with<L: Len, const MASKED: bool>(
        out: &mut [u64],
        a: &[u64],
        b: &[u64],
        scratch: &mut [u64],
    );

    /// Writes `a * b mod 2^(64 * L)` to `out`, `a`, `b` and `out` of `L` limbs; `MASKED` as for
    /// [`Self::product_with`].
    fn truncated_with<L: Len, const MASKED: bool>(
        out: &mut [u64],
        a: &[u64],
        b: &[u64],
        scratch: &mut [u64],
    );

    /// [`Self::product_with`] with corrections that may branch on the operands.
    #[inline(always)]
    fn product<L: Len>(out: &mut [u64], a: &[u64], b: &[u64], scratch: &mut [u64]) {
        Self::product_with::<L, false>(out, a, b, scratch);
    }

    /// [`Self::product_with`] with its first split inlined, for the function that owns `out` and
    /// `scratch`: their initialization and the split's writes sit in one function, where the
    /// compiler drops the zeros that the writes overwrite.
    fn entry_product<L: Len, const MASKED: bool>(
        out: &mut [u64],
        a: &[u64],
        b: &[u64],
        scratch: &mut [u64],
    );

    /// [`Self::truncated_with`] with its first split inlined, as for [`Self::entry_product`].
    fn entry_truncated<L: Len, const MASKED: bool>(
        out: &mut [u64],
        a: &[u64],
        b: &[u64],
        scratch: &mut [u64],
    );

    /// Divides the `2 * L` limbs of `numerator` by the normalized `L`-limb `divisor`, whose two
    /// leading limbs `leading` divides by: writes the low `L` quotient limbs to `quotient`, leaves
    /// the remainder in the low `L` limbs of `numerator` and zeros above them, and returns the top
    /// quotient limb, `0` or `1`.
    fn divide<L: Len>(
        numerator: &mut [u64],
        divisor: &[u64],
        quotient: &mut [u64],
        leading: Reciprocal3By2,
        scratch: &mut [u64],
    ) -> u64;
}

/// No halving left.
pub(crate) struct Bottom;

/// One more halving than `D`.
pub(crate) struct Level<D>(PhantomData<D>);

/// Three halvings: the divide-and-conquer algorithms reach their basecases for operands of up to
/// eight times their thresholds.
pub(crate) type Halvings = Level<Level<Level<Bottom>>>;

impl Depth for Bottom {
    const LEVELS: usize = 0;

    #[inline(always)]
    fn product_with<L: Len, const MASKED: bool>(
        out: &mut [u64],
        a: &[u64],
        b: &[u64],
        _scratch: &mut [u64],
    ) {
        basecase(out, a, b);
    }

    #[inline(always)]
    fn truncated_with<L: Len, const MASKED: bool>(
        out: &mut [u64],
        a: &[u64],
        b: &[u64],
        _scratch: &mut [u64],
    ) {
        basecase(out, a, b);
    }

    #[inline(always)]
    fn entry_product<L: Len, const MASKED: bool>(
        out: &mut [u64],
        a: &[u64],
        b: &[u64],
        _scratch: &mut [u64],
    ) {
        basecase(out, a, b);
    }

    #[inline(always)]
    fn entry_truncated<L: Len, const MASKED: bool>(
        out: &mut [u64],
        a: &[u64],
        b: &[u64],
        _scratch: &mut [u64],
    ) {
        basecase(out, a, b);
    }

    #[inline(always)]
    fn divide<L: Len>(
        numerator: &mut [u64],
        divisor: &[u64],
        quotient: &mut [u64],
        leading: Reciprocal3By2,
        _scratch: &mut [u64],
    ) -> u64 {
        schoolbook(numerator, divisor, quotient, leading)
    }
}

impl<D: Depth> Depth for Level<D> {
    const LEVELS: usize = D::LEVELS + 1;

    #[inline(always)]
    fn product_with<L: Len, const MASKED: bool>(
        out: &mut [u64],
        a: &[u64],
        b: &[u64],
        scratch: &mut [u64],
    ) {
        let n = L::of(a.len());
        if n >= karatsuba::THRESHOLD && scratch.len() >= product_scratch(n, Self::LEVELS) {
            if !L::ERASED && (D::LEVELS == 0 || n - n / 2 < karatsuba::THRESHOLD) {
                // A step over basecases inlines into its caller.
                karatsuba::karatsuba_inlined::<D, L, MASKED>(out, a, b, scratch);
            } else {
                karatsuba::karatsuba::<D, L, MASKED>(out, a, b, scratch);
            }
        } else {
            basecase(out, a, b);
        }
    }

    #[inline(always)]
    fn truncated_with<L: Len, const MASKED: bool>(
        out: &mut [u64],
        a: &[u64],
        b: &[u64],
        scratch: &mut [u64],
    ) {
        let n = L::of(out.len());
        if !truncated::quadratic(n) && scratch.len() >= truncated::scratch_len(n, Self::LEVELS) {
            truncated::split_product::<Self, D, L, MASKED>(out, a, b, scratch);
        } else {
            basecase(out, a, b);
        }
    }

    #[inline(always)]
    fn entry_product<L: Len, const MASKED: bool>(
        out: &mut [u64],
        a: &[u64],
        b: &[u64],
        scratch: &mut [u64],
    ) {
        let n = L::of(a.len());
        if n >= karatsuba::THRESHOLD && scratch.len() >= product_scratch(n, Self::LEVELS) {
            karatsuba::karatsuba_inlined::<D, L, MASKED>(out, a, b, scratch);
        } else {
            basecase(out, a, b);
        }
    }

    #[inline(always)]
    fn entry_truncated<L: Len, const MASKED: bool>(
        out: &mut [u64],
        a: &[u64],
        b: &[u64],
        scratch: &mut [u64],
    ) {
        let n = L::of(out.len());
        if !truncated::quadratic(n) && scratch.len() >= truncated::scratch_len(n, Self::LEVELS) {
            truncated::split_product_inlined::<Self, D, L, MASKED>(out, a, b, scratch);
        } else {
            basecase(out, a, b);
        }
    }

    #[inline(always)]
    fn divide<L: Len>(
        numerator: &mut [u64],
        divisor: &[u64],
        quotient: &mut [u64],
        leading: Reciprocal3By2,
        scratch: &mut [u64],
    ) -> u64 {
        let n = L::of(divisor.len());
        if n >= dc::THRESHOLD && scratch.len() >= divide_scratch(n, Self::LEVELS) {
            if !L::ERASED && (D::LEVELS == 0 || n - n / 2 < dc::THRESHOLD) {
                // A step over schoolbook divisions inlines into its caller.
                dc::divide_halves_inlined::<D, L>(numerator, divisor, quotient, leading, scratch)
            } else {
                dc::divide_halves::<D, L>(numerator, divisor, quotient, leading, scratch)
            }
        } else {
            schoolbook(numerator, divisor, quotient, leading)
        }
    }
}

/// The scratch limbs of a product of two `len`-limb operands with `levels` halvings left: the
/// difference product of each Karatsuba level, `2 * ceil(len / 2)` limbs, along the chain of upper
/// halves.
#[inline(always)]
pub(crate) const fn product_scratch(len: usize, levels: usize) -> usize {
    let mut total = 0;
    let (mut len, mut levels) = (len, levels);
    while levels > 0 && len >= karatsuba::THRESHOLD {
        len -= len / 2;
        total += 2 * len;
        levels -= 1;
    }
    total
}

/// The scratch limbs of a division by a `len`-limb divisor with `levels` halvings left: the
/// product limbs of each level along the chain of upper halves, plus the deepest product's own
/// scratch.
#[inline(always)]
pub(crate) const fn divide_scratch(len: usize, levels: usize) -> usize {
    let (mut total, mut largest) = (0, 0);
    let (mut len, mut levels) = (len, levels);
    while levels > 0 && len >= dc::THRESHOLD {
        let lower = len / 2;
        let with_product = total + len + product_scratch(lower, levels - 1);
        if with_product > largest {
            largest = with_product;
        }
        total += len;
        len -= lower;
        levels -= 1;
    }
    if total > largest { total } else { largest }
}
