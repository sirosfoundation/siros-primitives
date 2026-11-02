//! The quadratic product under every multiplication: one row per limb of the shorter operand.
//!
//! A row of two carry chains adds each product's low half to the high half before it, then adds
//! the row into the output, four limbs at a time in registers. A product cut below its full length
//! drops the products past its end. Rows longer than [`UNROLLED`] run loops of multiply-accumulate
//! steps.
//!
//! [`product`] takes its lengths as types and unrolls rows and limbs in the source, so each length
//! simplifies once. [`basecase`] takes slices, whose lengths become constants only where it is
//! inlined; it unrolls the limbs of a row and loops over the rows.

use generic_array::{ArrayLength, GenericArray};

use crate::kernel::{add_mul_row, mul_row, split, split_mut};
use crate::limb::CarryingAdd;
use crate::unroll::{UNROLLED, unroll};

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
    if UNROLL {
        unroll!(index in 0..short.len() => {
            if let Some(&multiplier) = short.get(index) {
                row(out, long, index, multiplier);
            }
        });
    } else {
        for (index, &multiplier) in short.iter().enumerate() {
            row(out, long, index, multiplier);
        }
    }
}

/// Row `index` of a product.
#[inline(always)]
fn row(out: &mut [u64], long: &[u64], index: usize, multiplier: u64) {
    let (_, window) = split_mut(out, index);
    add_row(window, long, multiplier, index == 0);
}

/// Adds `long * multiplier` to `window`, cut at its end, or writes it there when `first`, in two
/// carry chains: a group of [`ROW_GROUP`] products, each low half plus the high half before it,
/// then the group added into the window. The group stays in registers. The limb above the
/// products is fresh: the rows before it in this output reach below it.
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
    let mut group = [0u64; ROW_GROUP];
    let (mut carry, mut high, mut sum_carry) = (0, 0, 0);
    unroll!(index in 0..len => {
        if let (Some(&limb), Some(slot)) = (long.get(index), group.get_mut(index % ROW_GROUP)) {
            if full || index + 1 < len {
                let product = u128::from(limb) * u128::from(multiplier);
                (*slot, carry) = (product as u64).add_carry(high, carry);
                high = (product >> 64) as u64;
            } else {
                *slot = limb.wrapping_mul(multiplier).wrapping_add(high).wrapping_add(carry);
            }
        }
        // The group ends here: it goes into the window.
        if index % ROW_GROUP == ROW_GROUP - 1 || index + 1 == len {
            let start = index - index % ROW_GROUP;
            let (_, slots) = split_mut(window, start);
            for (slot, &product) in slots.iter_mut().zip(group.iter().take(index % ROW_GROUP + 1)) {
                if first {
                    *slot = product;
                } else {
                    (*slot, sum_carry) = slot.add_carry(product, sum_carry);
                }
            }
        }
    });
    if full && let Some(slot) = window.get_mut(len) {
        // A high half is at most `2^64 - 2`, and the sum of the rows so far fits below the limb
        // above this one.
        *slot = high + carry + sum_carry;
    }
}

/// The products of a group of [`add_row`], which stay in registers.
const ROW_GROUP: usize = 4;

/// Drops the carry out of the output window. The window is the product width the caller chose:
/// for a wrapping product the carry is the part above it, for a full product it is zero.
#[inline(always)]
pub(crate) fn truncate_carry(_carry: u64) {}
