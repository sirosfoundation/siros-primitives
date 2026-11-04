//! The quadratic product under every multiplication: one row per limb of the shorter operand.
//!
//! Operands of up to [`MAC_ROWS`] limbs run each row as one chain of multiply-accumulate steps:
//! the 256-bit wrapping product took 13.0 cycles this way against 14.0 in groups, and 13.0
//! against 16.0 in constant time. A row of up to [`GROUPED_ROW`] limbs runs two carry chains: it adds each product's low half to
//! the high half before it, then adds a group of [`ROW_GROUP`] such limbs into the output from
//! registers. A product cut below its full length drops the products past its end. Longer rows
//! run loops of multiply-accumulate steps.
//!
//! [`product`] and [`square`] take their lengths as types and run one copy of each row and each
//! limb ([`each`]), so the limbs of a product stay in registers. [`basecase`] takes slices,
//! whose lengths become constants only where it is inlined; it loops over the rows and runs one
//! copy per limb up to [`GROUPED_ROW`], the copies past the length folding away.

use generic_array::{ArrayLength, GenericArray};
use typenum::{U16, Unsigned};

use crate::each::each;
use crate::kernel::{add_mul_row, mul_row, split, split_mut};
use crate::limb::CarryingAdd;

/// Writes `a * b mod 2^(64 * out.len())` to `out`, at most `A + B` limbs: one copy per row and
/// per group up to [`GROUPED_ROW`] limbs.
#[inline(always)]
pub(crate) fn product<A: ArrayLength, B: ArrayLength>(
    out: &mut [u64],
    a: &GenericArray<u64, A>,
    b: &GenericArray<u64, B>,
) {
    if A::USIZE <= MAC_ROWS && B::USIZE <= MAC_ROWS {
        mac_rows(out, a, b);
    } else if A::USIZE > GROUPED_ROW || B::USIZE > GROUPED_ROW {
        basecase(out, a, b);
    } else if A::USIZE >= B::USIZE {
        rows::<A, B>(out, a, b);
    } else {
        rows::<B, A>(out, b, a);
    }
}

/// The longest operands whose rows run one chain of multiply-accumulate steps each.
const MAC_ROWS: usize = 4;

/// Writes `a * b mod 2^(64 * out.len())` to `out`: one row per limb of `b`, each a chain of
/// multiply-accumulate steps into `out`.
#[inline(always)]
fn mac_rows(out: &mut [u64], a: &[u64], b: &[u64]) {
    for slot in out.iter_mut() {
        *slot = 0;
    }
    for (index, &multiplier) in b.iter().enumerate() {
        let (_, window) = split_mut(out, index);
        let mut slots = window.iter_mut();
        let mut carry = 0;
        for (&limb, slot) in a.iter().zip(slots.by_ref()) {
            let product = u128::from(limb) * u128::from(multiplier) + u128::from(carry);
            let (low, overflow) = slot.overflowing_add(product as u64);
            *slot = low;
            carry = (product >> 64) as u64 + u64::from(overflow);
        }
        if let Some(slot) = slots.next() {
            *slot = carry;
        }
    }
}

/// The rows of [`product`]: one per limb of `short`, each over the limbs of `long`.
#[inline(always)]
fn rows<Long: Unsigned, Short: Unsigned>(out: &mut [u64], long: &[u64], short: &[u64]) {
    each::<Short>(
        #[inline(always)]
        |index| {
            if let Some(&multiplier) = short.get(index) {
                let (_, window) = split_mut(out, index);
                add_row::<Long>(window, long, multiplier, index == 0);
            }
        },
    );
}

/// Writes `a^2` to the `2 * A` limbs of `out`: the products of distinct limbs once, doubled, plus
/// the squares of the limbs, with `A * (A + 1) / 2` multiplications where a product takes `A^2`.
/// Operands of up to [`GROUPED_ROW`] limbs run one copy per row; longer ones run the product.
#[inline(always)]
pub(crate) fn square<A: ArrayLength>(out: &mut [u64], a: &GenericArray<u64, A>) {
    if A::USIZE > GROUPED_ROW {
        basecase(out, a, a);
        return;
    }
    for slot in out.iter_mut() {
        *slot = 0;
    }
    // Row `i` adds `a_i * a_(i+1..)` at limb `2i + 1`.
    each::<A>(
        #[inline(always)]
        |index| {
            if let Some(&multiplier) = a.get(index)
                && index + 1 < A::USIZE
            {
                let (_, above) = split(a, index + 1);
                let (_, window) = split_mut(out, 2 * index + 1);
                add_row::<A>(window, above, multiplier, index == 0);
            }
        },
    );
    // Doubles the cross products, two limbs per index; they sum below `2^(128 A - 1)`.
    let mut below = 0;
    each::<A>(
        #[inline(always)]
        |index| {
            if let Some(pair) = out.get_mut(2 * index..2 * index + 2) {
                for slot in pair {
                    let limb = *slot;
                    *slot = (limb << 1) | below;
                    below = limb >> 63;
                }
            }
        },
    );
    let mut carry = 0;
    each::<A>(
        #[inline(always)]
        |index| {
            if let (Some(&limb), Some(pair)) = (a.get(index), out.get_mut(2 * index..2 * index + 2))
            {
                let square = u128::from(limb) * u128::from(limb);
                for (slot, half) in pair.iter_mut().zip([square as u64, (square >> 64) as u64]) {
                    (*slot, carry) = slot.add_carry(half, carry);
                }
            }
        },
    );
    // The square fits in `2 * A` limbs.
    truncate_carry(carry);
}

/// Writes `a * b mod 2^(64 * out.len())` to `out`. `out` is at most `a.len() + b.len()` limbs
/// long.
///
/// Inlined, the lengths of its slices fold to constants: as a call, a 2048-bit product took twice
/// the cycles.
#[inline(always)]
pub(crate) fn basecase(out: &mut [u64], a: &[u64], b: &[u64]) {
    let (long, short) = if a.len() >= b.len() { (a, b) } else { (b, a) };
    for (index, &multiplier) in short.iter().enumerate() {
        let (_, window) = split_mut(out, index);
        if long.len().min(window.len()) > GROUPED_ROW {
            add_row_chain(window, long, multiplier, index == 0);
        } else {
            // The length is a constant where the product is inlined: the copies past it fold.
            add_row::<GroupedRow>(window, long, multiplier, index == 0);
        }
    }
}

/// The longest row that runs as two carry chains in groups; longer rows run one chain of
/// multiply-accumulate steps.
type GroupedRow = U16;

/// [`GroupedRow`] as a number.
const GROUPED_ROW: usize = GroupedRow::USIZE;

/// The products of a group of [`add_row`], which stay in registers.
const ROW_GROUP: usize = 4;

/// Adds `long * multiplier` to `window`, cut at its end, or writes it there when `first`, with
/// one copy per limb of `Copies`, which `long` fills or, for a row of slices, reaches. The limb
/// above the products is fresh: the rows before it in this output reach below it. The carries
/// are locals of this function, so that they stay in registers and flags.
#[inline(always)]
fn add_row<Copies: Unsigned>(window: &mut [u64], long: &[u64], multiplier: u64, first: bool) {
    let len = long.len().min(window.len());
    // A full row ends in a limb above the window's products; a cut row's last product lands on
    // the window's top limb, which takes its low half.
    let full = window.len() > len;
    let mut group = [0u64; ROW_GROUP];
    let (mut carry, mut high, mut sum_carry) = (0, 0, 0);
    each::<Copies>(
        #[inline(always)]
        |index| {
            if index >= len {
                return;
            }
            if let (Some(&limb), Some(slot)) = (long.get(index), group.get_mut(index % ROW_GROUP)) {
                if full || index + 1 < len {
                    let product = u128::from(limb) * u128::from(multiplier);
                    (*slot, carry) = (product as u64).add_carry(high, carry);
                    high = (product >> 64) as u64;
                } else {
                    *slot = limb
                        .wrapping_mul(multiplier)
                        .wrapping_add(high)
                        .wrapping_add(carry);
                }
            }
            // The group ends here: it goes into the window.
            if index % ROW_GROUP == ROW_GROUP - 1 || index + 1 == len {
                let start = index - index % ROW_GROUP;
                let (_, slots) = split_mut(window, start);
                let products = group.iter().take(index % ROW_GROUP + 1);
                for (slot, &product) in slots.iter_mut().zip(products) {
                    if first {
                        *slot = product;
                    } else {
                        (*slot, sum_carry) = slot.add_carry(product, sum_carry);
                    }
                }
            }
        },
    );
    if full && let Some(slot) = window.get_mut(len) {
        // A high half is at most `2^64 - 2`, and the sum of the rows so far fits below the limb
        // above this one.
        *slot = high + carry + sum_carry;
    }
}

/// Adds `long * multiplier` to `window` as [`add_row`] does, in one chain of multiply-accumulate
/// steps, for rows longer than [`GROUPED_ROW`].
#[inline(always)]
fn add_row_chain(window: &mut [u64], long: &[u64], multiplier: u64, first: bool) {
    let len = long.len().min(window.len());
    let (row, above) = split_mut(window, len);
    let carry = if first {
        mul_row(row, long, multiplier)
    } else {
        add_mul_row(row, long, multiplier)
    };
    if let Some(slot) = above.first_mut() {
        *slot = carry;
    }
}

/// Drops the carry out of the output window. The window is the product width the caller chose:
/// for a wrapping product the carry is the part above it, for a full product it is zero.
#[inline(always)]
pub(crate) fn truncate_carry(_carry: u64) {}
