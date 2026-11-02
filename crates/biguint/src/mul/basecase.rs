//! The quadratic product under every multiplication: one row per limb of the shorter operand.
//!
//! A row of two carry chains adds each product's low half to the high half before it, then adds
//! the row into the output. A multiply-accumulate row adds each product, the output limb and the
//! carry limb in one `u128` sum. A full product adds its rows of two chains into one output.
//! A product cut below its full length adds even and odd rows into two outputs, whose carry chains
//! run side by side, and adds the two at the end: with multiply-accumulate rows up to
//! [`ACCUMULATE_LIMBS`] limbs, with rows of two chains beyond. Rows longer than [`UNROLLED`] run
//! loops of multiply-accumulate steps. Modelled on Zen 3 (llvm-mca, the bench's loop), each choice
//! takes the fewest cycles in latency and in throughput together at its sizes.
//!
//! [`product`] takes its lengths as types and unrolls rows and limbs in the source, so each length
//! simplifies once. [`basecase`] takes slices, whose lengths become constants only where it is
//! inlined; it unrolls the limbs of a row and loops over the rows.

use generic_array::{ArrayLength, GenericArray};

use crate::kernel::{add_assign, add_mul_row, mul_row, split, split_mut};
use crate::limb::CarryingAdd;
use crate::unroll::{UNROLLED, unroll};

/// The longest operand of a cut product that runs multiply-accumulate rows.
const ACCUMULATE_LIMBS: usize = 5;

/// Writes `a * b mod 2^(64 * O)` to `out`, `O` at most `A + B`, with rows and limbs unrolled up to
/// [`UNROLLED`] limbs.
#[inline(always)]
pub(crate) fn product<A: ArrayLength, B: ArrayLength, O: ArrayLength>(
    out: &mut GenericArray<u64, O>,
    a: &GenericArray<u64, A>,
    b: &GenericArray<u64, B>,
) {
    if A::USIZE > UNROLLED || B::USIZE > UNROLLED {
        basecase(out, a, b);
    } else {
        rows::<true>(out, a, b);
    }
}

/// Writes `a^2` to the `2 * A` limbs of `out`: the products of distinct limbs once, doubled, plus
/// the squares of the limbs, with `A * (A + 1) / 2` multiplications where a product takes `A^2`.
/// Operands of up to [`UNROLLED`] limbs run unrolled; longer ones run the product.
#[inline(always)]
pub(crate) fn square<A: ArrayLength>(out: &mut [u64], a: &GenericArray<u64, A>) {
    if A::USIZE > UNROLLED {
        basecase(out, a, a);
        return;
    }
    for slot in out.iter_mut() {
        *slot = 0;
    }
    // Row `i` adds `a_i * a_(i+1..)` at limb `2i + 1`.
    unroll!(index in 0..A::USIZE.saturating_sub(1) => {
        if let Some(&multiplier) = a.get(index) {
            let (_, above) = split(a, index + 1);
            let (_, window) = split_mut(out, 2 * index + 1);
            add_row(window, above, multiplier, index == 0);
        }
    });
    let mut below = 0;
    unroll!(index in 0..2 * A::USIZE => {
        if let Some(slot) = out.get_mut(index) {
            let limb = *slot;
            *slot = (limb << 1) | below;
            below = limb >> 63;
        }
    });
    let mut carry = 0;
    unroll!(index in 0..A::USIZE => {
        if let Some(&limb) = a.get(index) {
            let square = u128::from(limb) * u128::from(limb);
            if let Some(slot) = out.get_mut(2 * index) {
                (*slot, carry) = slot.add_carry(square as u64, carry);
            }
            if let Some(slot) = out.get_mut(2 * index + 1) {
                (*slot, carry) = slot.add_carry((square >> 64) as u64, carry);
            }
        }
    });
    // The square fits in `2 * A` limbs.
    truncate_carry(carry);
}

/// Writes `a * b mod 2^(64 * out.len())` to `out`. `out` is at most `a.len() + b.len()` limbs
/// long.
///
/// An unoptimized build calls the kernel: inlined, every unrolled copy keeps its own stack slots
/// in the caller's frame, and a 4096-bit product inlines dozens of kernels.
#[cfg_attr(debug_assertions, inline(never))]
#[cfg_attr(not(debug_assertions), inline(always))]
pub(crate) fn basecase(out: &mut [u64], a: &[u64], b: &[u64]) {
    rows::<false>(out, a, b);
}

/// The rows of [`basecase`], unrolled when `UNROLL`.
#[inline(always)]
fn rows<const UNROLL: bool>(out: &mut [u64], a: &[u64], b: &[u64]) {
    let (long, short) = if a.len() >= b.len() { (a, b) } else { (b, a) };
    if long.len() > UNROLLED || out.len() >= long.len() + short.len() {
        if UNROLL {
            unroll!(index in 0..short.len() => {
                if let Some(&multiplier) = short.get(index) {
                    full_row(out, long, index, multiplier);
                }
            });
        } else {
            for (index, &multiplier) in short.iter().enumerate() {
                full_row(out, long, index, multiplier);
            }
        }
    } else {
        split_rows::<UNROLL>(out, long, short);
    }
}

/// Row `index` of a product into one output.
#[inline(always)]
fn full_row(out: &mut [u64], long: &[u64], index: usize, multiplier: u64) {
    let (_, window) = split_mut(out, index);
    add_row(window, long, multiplier, index == 0);
}

/// A cut product into two outputs: `out` takes the even rows, a buffer the odd ones. Each output
/// starts at zero, so a row adds onto zeros above the rows before it.
#[inline(always)]
fn split_rows<const UNROLL: bool>(out: &mut [u64], long: &[u64], short: &[u64]) {
    for slot in out.iter_mut() {
        *slot = 0;
    }
    let mut buffer = [0u64; 2 * UNROLLED];
    let (odd, _) = split_mut(&mut buffer, out.len());
    if UNROLL {
        unroll!(index in 0..short.len() => {
            if let Some(&multiplier) = short.get(index) {
                split_row(out, odd, long, index, multiplier);
            }
        });
    } else {
        for (index, &multiplier) in short.iter().enumerate() {
            split_row(out, odd, long, index, multiplier);
        }
    }
    truncate_carry(add_assign(out, odd));
}

/// Row `index` of a cut product into `even` or `odd` by its parity.
#[inline(always)]
fn split_row(even: &mut [u64], odd: &mut [u64], long: &[u64], index: usize, multiplier: u64) {
    let output = if index.is_multiple_of(2) { even } else { odd };
    let (_, window) = split_mut(output, index);
    if long.len() <= ACCUMULATE_LIMBS {
        accumulate_row(window, long, multiplier, index < 2);
    } else {
        add_row(window, long, multiplier, index < 2);
    }
}

/// Adds `long * multiplier` to `window`, cut at its end, or writes it there when `first`, in rows
/// of two carry chains. The limb above the products is fresh: the rows before it in this output
/// reach below it.
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

/// [`add_row`] in multiply-accumulate steps: each product with the window's limb and the carry
/// limb in one `u128` sum, `long` at most [`UNROLLED`] limbs.
#[inline(always)]
fn accumulate_row(window: &mut [u64], long: &[u64], multiplier: u64, first: bool) {
    let len = long.len().min(window.len());
    let full = window.len() > len;
    let mut carry = 0;
    unroll!(index in 0..len => {
        if let (Some(&limb), Some(slot)) = (long.get(index), window.get_mut(index)) {
            let addend = if first { 0 } else { *slot };
            if full || index + 1 < len {
                let sum = u128::from(limb) * u128::from(multiplier)
                    + u128::from(addend)
                    + u128::from(carry);
                *slot = sum as u64;
                carry = (sum >> 64) as u64;
            } else {
                *slot = limb.wrapping_mul(multiplier).wrapping_add(addend).wrapping_add(carry);
            }
        }
    });
    if full && let Some(slot) = window.get_mut(len) {
        *slot = carry;
    }
}

/// Drops the carry out of the output window. The window is the product width the caller chose:
/// for a wrapping product the carry is the part above it, for a full product it is zero.
#[inline(always)]
pub(crate) fn truncate_carry(_carry: u64) {}
