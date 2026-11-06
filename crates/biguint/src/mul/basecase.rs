//! The quadratic product under every multiplication: one row per limb of the shorter operand.
//!
//! Operands of up to [`MAC_ROWS`] limbs run each row as one chain of multiply-accumulate steps:
//! the 256-bit wrapping product took 12.4 cycles this way against 14.0 in groups, and 12.4
//! against 16.0 in constant time. A row of up to [`GROUPED_ROW`] limbs runs two carry chains: it
//! adds each product's low half to the high half before it, then adds a group of [`ROW_GROUP`]
//! such limbs into the output from registers. A product cut below its full length drops the
//! products past its end. Longer rows run loops of multiply-accumulate steps. On WebAssembly every
//! row up to [`GROUPED_ROW`] limbs is one chain: without wide arithmetic one copy per row from
//! `COPIED_ROWS` rows; with it, the products of `COLUMN_LIMBS`-limb operands and the products of
//! slices of up to `COLUMN_LIMBS` limbs run column by column (`columns`).
//!
//! [`product`] and [`square`] take their lengths as types and run one copy of each row and each
//! limb ([`each`]), so the limbs of a product stay in registers. [`basecase`] takes slices,
//! whose lengths become constants only where it is inlined; it loops over the rows and runs one
//! copy per limb up to [`GROUPED_ROW`], the copies past the length folding away.

use generic_array::{ArrayLength, GenericArray};
#[cfg(wide_arithmetic)]
use typenum::U6;
use typenum::{U16, Unsigned};

use crate::each::each;
use crate::kernel::{add_mul_row, mul_row, split, split_mut};
use crate::limb::{CarryingAdd, mul_add, widening_mul};

/// Writes `a * b mod 2^(64 * out.len())` to `out`, at most `A + B` limbs: one copy per row and
/// per group up to [`GROUPED_ROW`] limbs.
#[inline(always)]
pub(crate) fn product<A: ArrayLength, B: ArrayLength>(
    out: &mut [u64],
    a: &GenericArray<u64, A>,
    b: &GenericArray<u64, B>,
) {
    #[cfg(wide_arithmetic)]
    if const { A::USIZE == COLUMN_LIMBS || B::USIZE == COLUMN_LIMBS } && out.len() <= COLUMN_LIMBS {
        columns(out, a, b);
        return;
    }
    #[cfg(all(target_arch = "wasm32", not(wide_arithmetic)))]
    if const { B::USIZE >= COPIED_ROWS && A::USIZE <= MAC_ROWS && B::USIZE <= MAC_ROWS } {
        copied_mac_rows::<B>(out, a, b);
        return;
    }
    #[cfg(wide_arithmetic)]
    if const { A::USIZE == CHAINED_LIMBS && B::USIZE == CHAINED_LIMBS }
        && out.len() == CHAINED_LIMBS
    {
        chained_rows(out, a, b);
        return;
    }
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

/// The operand length whose products run [`columns`] with wide arithmetic, and the most limbs of a
/// product of slices that does. Chrome and Firefox took the 1024-bit wrapping product in 315 and
/// 567 cycles by columns against 337 and 742 by rows; the 2048-bit one, whose Karatsuba leaves
/// are 8-limb full products, in 2,317 and 3,520 against 2,493 and 3,676. At 6 and 8 limbs the two
/// forms tie, and the rows run fewer instructions.
#[cfg(wide_arithmetic)]
const COLUMN_LIMBS: usize = ColumnLimbs::USIZE;

/// [`COLUMN_LIMBS`] as a type.
#[cfg(wide_arithmetic)]
type ColumnLimbs = U16;

/// Writes `a * b mod 2^(64 * out.len())` to `out`, at most [`COLUMN_LIMBS`] limbs, column by
/// column (Comba): each column sums the low halves of its products into one `u128` and the high
/// halves, which belong to the next column, into another. The products of a column are
/// independent: with wide arithmetic, each is one `i64.mul_wide_u` and two `i64.add128`, and only
/// the two sums carry from column to column.
#[cfg(wide_arithmetic)]
#[inline(always)]
fn columns(out: &mut [u64], a: &[u64], b: &[u64]) {
    let len = out.len();
    let mut carry: u128 = 0;
    let mut highs: u128 = 0;
    each::<ColumnLimbs>(
        #[inline(always)]
        |column| {
            if column >= len {
                return;
            }
            let mut lows = carry + highs;
            let mut next = 0u128;
            let last = column + 1 == len;
            for (index, &limb) in a.iter().enumerate().take(column + 1) {
                if let Some(&factor) = b.get(column - index) {
                    if last {
                        lows = lows.wrapping_add(u128::from(limb.wrapping_mul(factor)));
                    } else {
                        let product = widening_mul(limb, factor);
                        lows += u128::from(product as u64);
                        next += product >> 64;
                    }
                }
            }
            if let Some(slot) = out.get_mut(column) {
                *slot = lows as u64;
            }
            carry = lows >> 64;
            highs = next;
        },
    );
}

/// The longest operands whose rows run one chain of multiply-accumulate steps each. On
/// WebAssembly a step takes its additions into the product, where the second carry chain of a
/// grouped row costs a comparison per carry: a 384-bit wrapping product took 464 instructions in
/// chains against 857 in groups.
const MAC_ROWS: usize = if cfg!(target_arch = "wasm32") {
    GROUPED_ROW
} else {
    4
};

/// Writes `a * b mod 2^(64 * out.len())` to `out`: one row per limb of `b`, each a chain of
/// multiply-accumulate steps into `out`.
///
/// Each step reads its multiplier from `b`. A multiplier read once per row took the 256-bit
/// product 5 moves into the multiplication's implicit register where 3 do, and the 512-bit full
/// product 15 where 4 do.
#[inline(always)]
fn mac_rows(out: &mut [u64], a: &[u64], b: &[u64]) {
    for slot in out.iter_mut() {
        *slot = 0;
    }
    for index in 0..b.len() {
        mac_row!(out, a, b, index);
    }
}

/// The operand length whose wrapping products run [`chained_rows`] with wide arithmetic: Chrome and
/// Firefox took the 384-bit one in 43.1 and 70.7 cycles against 54.0 and 66.7 in [`mac_rows`]. At 4
/// and 8 limbs [`mac_rows`] took 16.8 and 26.0, 70.9 and 126, against 18.4 and 27.6, 80.8 and 128.
#[cfg(wide_arithmetic)]
const CHAINED_LIMBS: usize = 6;

/// Writes `a * b mod 2^(64 * CHAINED_LIMBS)` to `out`, from the rows in a local sum: one row per limb of `a`, every
/// step a full multiply-accumulate with the sum's limb added before the carry.
#[cfg(wide_arithmetic)]
#[inline(always)]
fn chained_rows(out: &mut [u64], a: &[u64], b: &[u64]) {
    let mut sum = [0u64; CHAINED_LIMBS];
    each::<U6>(
        #[inline(always)]
        |index| {
            if let Some(&multiplier) = a.get(index) {
                let mut carry = 0;
                each::<U6>(
                    #[inline(always)]
                    |column| {
                        if let (Some(&limb), Some(slot)) =
                            (b.get(column), sum.get_mut(index + column))
                        {
                            (*slot, carry) = mul_add(multiplier, limb, *slot, carry);
                        }
                    },
                );
            }
        },
    );
    for (slot, limb) in out.iter_mut().zip(sum) {
        *slot = limb;
    }
}

/// [`mac_rows`] with one copy per row, `Rows` the length of `b`.
#[cfg(all(target_arch = "wasm32", not(wide_arithmetic)))]
#[inline(always)]
fn copied_mac_rows<Rows: Unsigned>(out: &mut [u64], a: &[u64], b: &[u64]) {
    for slot in out.iter_mut() {
        *slot = 0;
    }
    each::<Rows>(
        #[inline(always)]
        |index| mac_row!(out, a, b, index),
    );
}

/// Adds row `index` of [`mac_rows`], `a * b[index]`, to `out`.
macro_rules! mac_row {
    ($out:ident, $a:ident, $b:ident, $index:ident) => {{
        let (_, window) = split_mut($out, $index);
        let len = $a.len().min(window.len());
        // A full row ends in a fresh limb above its products. A cut row's last product lands on
        // the window's top limb, which takes its low half.
        let full = window.len() > len;
        let chain = if full { len } else { len.saturating_sub(1) };
        let (row, rest) = split_mut(window, chain);
        let mut carry = 0;
        for (&limb, slot) in $a.iter().zip(row.iter_mut()) {
            if let Some(&multiplier) = $b.get($index) {
                (*slot, carry) = mul_add(limb, multiplier, carry, *slot);
            }
        }
        if full {
            if let Some(slot) = rest.first_mut() {
                *slot = carry;
            }
        } else if let (Some(slot), Some(&limb), Some(&multiplier)) =
            (rest.first_mut(), $a.get(chain), $b.get($index))
        {
            *slot = limb
                .wrapping_mul(multiplier)
                .wrapping_add(*slot)
                .wrapping_add(carry);
        }
    }};
}
use mac_row;

/// The fewest rows that run [`copied_mac_rows`] on WebAssembly without wide arithmetic,
/// where a loop over the rows did not unroll. The 384-bit wrapping product took Chrome 116 cycles
/// in copies against 185 in the loop, Firefox 182 against 198; the 1024-bit one 1,235 against 1,340
/// and 1,506 against 1,589. The 256-bit one took Chrome 50.1 and Firefox 74.6 in copies against
/// 47.7 and 67.2. With wide arithmetic, where 16-limb operands run by columns, the copies of 6 and
/// 8 rows took the two browsers 122 and 204 cycles in sum against 118 and 203 in the loop.
#[cfg(all(target_arch = "wasm32", not(wide_arithmetic)))]
const COPIED_ROWS: usize = 5;

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
                if MAC_ROWS == GROUPED_ROW {
                    add_row_chain(window, above, multiplier, index == 0);
                } else {
                    add_row::<A>(window, above, multiplier, index == 0);
                }
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
                let square = widening_mul(limb, limb);
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
    #[cfg(wide_arithmetic)]
    if out.len() <= COLUMN_LIMBS {
        columns(out, a, b);
        return;
    }
    let (long, short) = if a.len() >= b.len() { (a, b) } else { (b, a) };
    for (index, &multiplier) in short.iter().enumerate() {
        let (_, window) = split_mut(out, index);
        if long.len().min(window.len()) > GROUPED_ROW || MAC_ROWS == GROUPED_ROW {
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
                    let product = widening_mul(limb, multiplier);
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
