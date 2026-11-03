//! The recursion budget of the divide-and-conquer products and divisions.
//!
//! Each level is its own type, so every level inlines into the one above and the halved lengths
//! fold to constants down to the basecase. A level runs the basecase, the quadratic product or the
//! schoolbook division, below the threshold or when its scratch is too short for the recursion;
//! the scratch length is a constant too, so that choice folds as well. A truncated product splits
//! at its own level and hands its cross products to the level below.

use core::marker::PhantomData;

use crate::div::dc;
use crate::div::reciprocal::Reciprocal3By2;
use crate::div::schoolbook;
use crate::mul::basecase;
use crate::mul::karatsuba;
use crate::mul::truncated;

/// A recursion budget.
pub(crate) trait Depth {
    /// The halvings left.
    const LEVELS: usize;

    /// Writes `a * b` to `out[..2 * a.len()]` for equally long `a` and `b`. `MASKED` runs every
    /// correction under masks: no branch and no memory address depends on the operands.
    fn product_with<const MASKED: bool>(out: &mut [u64], a: &[u64], b: &[u64], scratch: &mut [u64]);

    /// Writes `a * b mod 2^(64 * out.len())` to `out`, `a` and `b` as long as `out`; `MASKED` as
    /// for [`Self::product_with`].
    fn truncated_with<const MASKED: bool>(
        out: &mut [u64],
        a: &[u64],
        b: &[u64],
        scratch: &mut [u64],
    );

    /// [`Self::product_with`] with corrections that may branch on the operands.
    #[inline(always)]
    fn product(out: &mut [u64], a: &[u64], b: &[u64], scratch: &mut [u64]) {
        Self::product_with::<false>(out, a, b, scratch);
    }

    /// Divides the `2 * divisor.len()` limbs of `numerator` by the normalized `divisor`, whose
    /// two leading limbs `leading` divides by: writes the low `divisor.len()` quotient limbs to
    /// `quotient`, leaves the remainder in the low `divisor.len()` limbs of `numerator` and zeros
    /// above them, and returns the top quotient limb, `0` or `1`.
    fn divide(
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

/// The fewest limbs whose products and divisions run as ordinary calls: from there an operation
/// costs hundreds of cycles, against a few for the call, and its copy at every call site costs
/// more compile time than the call costs at run time.
pub(crate) const CALLED_LIMBS: usize = 32;

/// Runs `body`, inlined below [`CALLED_LIMBS`] limbs and as a call of [`called`] from there.
/// `body` captures its operands by value, so their lengths stay constants in the call.
#[inline(always)]
pub(crate) fn by_width<R>(limbs: usize, body: impl FnOnce() -> R) -> R {
    if limbs >= CALLED_LIMBS {
        called(body)
    } else {
        body()
    }
}

/// `body()` as an ordinary function, one per caller of [`by_width`] and width: LLVM inlines it
/// where it judges the copy worth its size.
#[inline]
fn called<R>(body: impl FnOnce() -> R) -> R {
    body()
}

impl Depth for Bottom {
    const LEVELS: usize = 0;

    #[inline(always)]
    fn product_with<const MASKED: bool>(
        out: &mut [u64],
        a: &[u64],
        b: &[u64],
        _scratch: &mut [u64],
    ) {
        basecase(out, a, b);
    }

    #[inline(always)]
    fn truncated_with<const MASKED: bool>(
        out: &mut [u64],
        a: &[u64],
        b: &[u64],
        _scratch: &mut [u64],
    ) {
        basecase(out, a, b);
    }

    #[inline(always)]
    fn divide(
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
    fn product_with<const MASKED: bool>(
        out: &mut [u64],
        a: &[u64],
        b: &[u64],
        scratch: &mut [u64],
    ) {
        if a.len() >= karatsuba::THRESHOLD
            && scratch.len() >= product_scratch(a.len(), Self::LEVELS)
        {
            karatsuba::karatsuba::<D, MASKED>(out, a, b, scratch);
        } else {
            basecase(out, a, b);
        }
    }

    #[inline(always)]
    fn truncated_with<const MASKED: bool>(
        out: &mut [u64],
        a: &[u64],
        b: &[u64],
        scratch: &mut [u64],
    ) {
        if !truncated::quadratic(out.len())
            && scratch.len() >= truncated::scratch_len(out.len(), Self::LEVELS)
        {
            truncated::split_product::<Self, D, MASKED>(out, a, b, scratch);
        } else {
            basecase(out, a, b);
        }
    }

    #[inline(always)]
    fn divide(
        numerator: &mut [u64],
        divisor: &[u64],
        quotient: &mut [u64],
        leading: Reciprocal3By2,
        scratch: &mut [u64],
    ) -> u64 {
        if divisor.len() >= dc::THRESHOLD
            && scratch.len() >= divide_scratch(divisor.len(), Self::LEVELS)
        {
            dc::divide_halves::<D>(numerator, divisor, quotient, leading, scratch)
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
