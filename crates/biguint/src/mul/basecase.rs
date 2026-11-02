//! The quadratic product under every multiplication: one row per limb of the shorter operand. A
//! row of up to [`UNROLLED`] products runs as two carry chains unrolled in the source: the multiply
//! chain adds each product's low half to the high half before it, then the add chain adds the row
//! into the output. Longer rows run loops of fused multiply-accumulate steps with one carry limb.

use crate::kernel::{add_mul_row, mul_row, split_mut};
use crate::limb::CarryingAdd;
use crate::unroll::{UNROLLED, unroll};

/// Writes `a * b mod 2^(64 * out.len())` to `out`. `out` is at most `a.len() + b.len()` limbs
/// long.
///
/// An unoptimized build calls the kernel: inlined, every unrolled copy keeps its own stack slots
/// in the caller's frame, and a 4096-bit product inlines dozens of kernels.
#[cfg_attr(debug_assertions, inline(never))]
#[cfg_attr(not(debug_assertions), inline(always))]
pub(crate) fn basecase(out: &mut [u64], a: &[u64], b: &[u64]) {
    let (long, short) = if a.len() >= b.len() { (a, b) } else { (b, a) };
    unroll!(index in 0..short.len() => {
        if let Some(&multiplier) = short.get(index) {
            let (_, window) = split_mut(out, index);
            add_row(window, long, multiplier, index == 0);
        }
    });
}

/// Adds `long * multiplier` to `window`, cut at its end, or writes it there when `first`. The
/// rows before it covered the window's limbs up to `long.len()`, so the limb above the products
/// is fresh.
#[inline(always)]
fn add_row(window: &mut [u64], long: &[u64], multiplier: u64, first: bool) {
    let len = long.len().min(window.len());
    if len > UNROLLED {
        let (row, above) = split_mut(window, len);
        let carry = if first {
            mul_row(row, long, multiplier)
        } else {
            add_mul_row(row, long, multiplier)
        };
        if let Some(slot) = above.first_mut() {
            *slot = carry;
        }
        return;
    }
    // A full row ends in a limb above the window's products; a cut row's last product lands on
    // the window's top limb, which takes its low half.
    let full = window.len() > len;
    let mut row = [0u64; UNROLLED + 1];
    let (mut carry, mut high) = (0, 0);
    unroll!(index in 0..len => {
        if let (Some(&limb), Some(slot)) = (long.get(index), row.get_mut(index)) {
            if full || index + 1 < len {
                let product = u128::from(limb) * u128::from(multiplier);
                (*slot, carry) = (product as u64).add_carry(high, carry);
                high = (product >> 64) as u64;
            } else {
                *slot = limb.wrapping_mul(multiplier).wrapping_add(high).wrapping_add(carry);
            }
        }
    });
    if full && let Some(slot) = row.get_mut(len) {
        // A high half is at most `2^64 - 2`.
        *slot = high + carry;
    }
    if first {
        unroll!(index in 0..len + usize::from(full) => {
            if let (Some(slot), Some(&limb)) = (window.get_mut(index), row.get(index)) {
                *slot = limb;
            }
        });
    } else {
        let mut carry = 0;
        unroll!(index in 0..len => {
            if let (Some(slot), Some(&limb)) = (window.get_mut(index), row.get(index)) {
                (*slot, carry) = slot.add_carry(limb, carry);
            }
        });
        if full && let (Some(slot), Some(&limb)) = (window.get_mut(len), row.get(len)) {
            // The sum of the rows so far fits below the limb above this one.
            *slot = limb + carry;
        }
    }
}

/// Drops the carry out of the output window. The window is the product width the caller chose:
/// for a wrapping product the carry is the part above it, for a full product it is zero.
#[inline(always)]
pub(crate) fn truncate_carry(_carry: u64) {}
