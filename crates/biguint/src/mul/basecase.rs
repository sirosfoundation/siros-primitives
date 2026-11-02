//! The quadratic products under every multiplication: the stream product where the compiler
//! unrolls it completely, rows of fused multiply-accumulate steps elsewhere.

use crate::kernel::{add_mul_row, mul_add, mul_row, split_mut};
use crate::stream::{LimbStream, Unrolled};

/// The most steps of a full stream product that the compiler unrolls completely: a 9 by 9 limb
/// product, 460 instructions against 467 for the rows (rustc 1.99, `-C target-cpu=native`). At
/// 10 by 10 limbs the stream stays a loop of 874 instructions against 574 for the rows.
pub(crate) const FULL_STREAM_STEPS: usize = 126;

/// The most steps of a truncated stream product that the compiler unrolls completely: 12 by 12
/// limbs modulo `2^768`, 454 instructions against 761 for the rows (rustc 1.99,
/// `-C target-cpu=native`).
pub(crate) const TRUNCATED_STREAM_STEPS: usize = 78;

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
/// Each limb of the shorter operand scales the longer one, zero-extended, into the output window,
/// which advances one limb per multiplier limb. Each step is one fused multiply-accumulate with a
/// single carry limb. The accumulation runs over the whole window, so the loop has a constant
/// shape that the compiler unrolls completely.
///
/// Operands of equal length take their multipliers from `a`. LLVM then keeps each multiplier in
/// the implicit `rdx` operand of `mulx` for its whole row; the other orientation reloads `rdx`
/// before most products (512 bits: 205 instructions against 232).
#[inline(always)]
pub(crate) fn stream(out: &mut [u64], a: &[u64], b: &[u64]) {
    for slot in out.iter_mut() {
        *slot = 0;
    }
    let (a, b) = if b.len() >= a.len() { (b, a) } else { (a, b) };
    let mut window = out;
    for &multiplier in b {
        let mut source = Unrolled::new(a);
        let mut carry = 0;
        for slot in window.iter_mut() {
            (*slot, carry) = mul_add(multiplier, source.next_limb(), *slot, carry);
        }
        truncate_carry(carry);
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
