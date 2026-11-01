//! The quadratic products under every multiplication: the stream product where the compiler
//! unrolls it completely, rows of fused multiply-accumulate steps elsewhere.

use crate::kernel::{add_mul_row, mul_row, split_mut};
use crate::stream::{MulIter, ZeroExtended, add_into};

/// The most steps of a full stream product that the compiler unrolls completely (a 7 by 7 limb
/// product), measured with rustc 1.99. Up to there the unrolled stream beats the rows by up to
/// 12%; past it the stream stays a loop and the rows win.
pub(crate) const FULL_STREAM_STEPS: usize = 77;

/// The most steps of a truncated stream product that the compiler unrolls completely (9 by 9
/// limbs modulo `2^576`), measured with rustc 1.99.
pub(crate) const TRUNCATED_STREAM_STEPS: usize = 45;

/// The steps of the stream product of an `a`-limb and a `b`-limb operand into `out` limbs: each
/// limb of the shorter operand adds a row over the rest of the output window.
#[inline(always)]
pub(crate) const fn stream_steps(a: usize, b: usize, out: usize) -> usize {
    let shorter = if a < b { a } else { b };
    let rows = if shorter < out { shorter } else { out };
    rows * out - rows * rows.saturating_sub(1) / 2
}

/// Writes `a * b mod 2^(64 * out.len())` to `out`: the stream product where the compiler unrolls
/// it completely, the rows elsewhere. `out` is at most `a.len() + b.len()` limbs long.
#[inline(always)]
pub(crate) fn basecase(out: &mut [u64], a: &[u64], b: &[u64]) {
    let limit = if out.len() >= a.len() + b.len() {
        FULL_STREAM_STEPS
    } else {
        TRUNCATED_STREAM_STEPS
    };
    if stream_steps(a.len(), b.len(), out.len()) <= limit {
        stream(out, a, b);
    } else {
        rows(out, a, b);
    }
}

/// Writes `a * b mod 2^(64 * out.len())` to `out` as the stream product.
///
/// The longer operand's [`MulIter`] stream is scaled by each limb of the shorter one and added
/// into the output window, which advances one limb per multiplier limb. The accumulation runs
/// over the whole window, so the loop has a constant shape that the compiler unrolls into two
/// interleaved carry chains.
#[inline(always)]
pub(crate) fn stream(out: &mut [u64], a: &[u64], b: &[u64]) {
    for slot in out.iter_mut() {
        *slot = 0;
    }
    let (a, b) = if b.len() > a.len() { (b, a) } else { (a, b) };
    let mut window = out;
    for &multiplier in b {
        truncate_carry(add_into(
            MulIter::new(ZeroExtended::new(a.iter().copied()), multiplier),
            window,
        ));
        window = match window.split_first_mut() {
            Some((_, rest)) => rest,
            None => break,
        };
    }
}

/// Writes `a * b mod 2^(64 * out.len())` to `out`: one fused multiply-accumulate row of the longer
/// operand per limb of the shorter, each row one limb higher. The first row overwrites; `out`
/// above the full product keeps its limbs.
#[inline(always)]
pub(crate) fn rows(out: &mut [u64], a: &[u64], b: &[u64]) {
    let (long, short) = if a.len() >= b.len() { (a, b) } else { (b, a) };
    let Some((&first, rest)) = short.split_first() else {
        return;
    };
    {
        let (row, above) = split_mut(out, long.len());
        let carry = mul_row(row, long, first);
        if let Some(slot) = above.first_mut() {
            *slot = carry;
        }
    }
    for (index, &multiplier) in rest.iter().enumerate() {
        let (_, window) = split_mut(out, index + 1);
        let (row, above) = split_mut(window, long.len());
        let carry = add_mul_row(row, long, multiplier);
        if let Some(slot) = above.first_mut() {
            *slot = carry;
        }
    }
}

/// Drops the carry out of the output window. The window is the product width the caller chose:
/// for a wrapping product the carry is the part above it, for a full product it is zero.
#[inline(always)]
pub(crate) fn truncate_carry(_carry: u64) {}
