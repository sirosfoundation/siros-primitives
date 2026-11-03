//! The quadratic product under every multiplication: one row per limb of the shorter operand.
//!
//! A row of up to [`GROUPED_ROW`] limbs runs two carry chains: it adds each product's low half to
//! the high half before it, then adds a group of [`ROW_GROUP`] such limbs into the output from
//! registers. A product cut below its full length drops the products past its end. Longer rows
//! run loops of multiply-accumulate steps.
//!
//! [`product`] and [`square`] take their lengths as types and run one copy of each row and each
//! limb ([`Indices`]), so the limbs of a product stay in registers. [`basecase`] takes slices,
//! whose lengths become constants only where it is inlined; it loops over the rows and runs one
//! copy per limb up to [`GROUPED_ROW`], the copies past the length folding away.

use generic_array::{ArrayLength, GenericArray};
use typenum::{U16, Unsigned};

use crate::indices::Indices;
use crate::kernel::{add_mul_row, mul_row, split, split_mut};
use crate::limb::CarryingAdd;

/// Writes `a * b mod 2^(64 * out.len())` to `out`, at most `A + B` limbs: one copy per row and
/// per group up to [`GROUPED_ROW`] limbs.
#[inline(always)]
pub(crate) fn product<A: ArrayLength + Indices, B: ArrayLength + Indices>(
    out: &mut [u64],
    a: &GenericArray<u64, A>,
    b: &GenericArray<u64, B>,
) {
    if A::USIZE > GROUPED_ROW || B::USIZE > GROUPED_ROW {
        basecase(out, a, b);
    } else if A::USIZE >= B::USIZE {
        rows::<A, B>(out, a, b);
    } else {
        rows::<B, A>(out, b, a);
    }
}

/// The rows of [`product`]: one per limb of `short`, each over the limbs of `long`.
#[inline(always)]
fn rows<Long: Indices, Short: Indices>(out: &mut [u64], long: &[u64], short: &[u64]) {
    Short::each(
        #[inline(always)]
        |index| {
            if let Some(&multiplier) = short.get(index) {
                let (_, window) = split_mut(out, index);
                Row::new(window, long, multiplier, index == 0).run_copies::<Long>();
            }
        },
    );
}

/// Writes `a^2` to the `2 * A` limbs of `out`: the products of distinct limbs once, doubled, plus
/// the squares of the limbs, with `A * (A + 1) / 2` multiplications where a product takes `A^2`.
/// Operands of up to [`GROUPED_ROW`] limbs run one copy per row; longer ones run the product.
#[inline(always)]
pub(crate) fn square<A: ArrayLength + Indices>(out: &mut [u64], a: &GenericArray<u64, A>) {
    if A::USIZE > GROUPED_ROW {
        basecase(out, a, a);
        return;
    }
    for slot in out.iter_mut() {
        *slot = 0;
    }
    // Row `i` adds `a_i * a_(i+1..)` at limb `2i + 1`.
    A::each(
        #[inline(always)]
        |index| {
            if let Some(&multiplier) = a.get(index)
                && index + 1 < A::USIZE
            {
                let (_, above) = split(a, index + 1);
                let (_, window) = split_mut(out, 2 * index + 1);
                Row::new(window, above, multiplier, index == 0).run_copies::<A>();
            }
        },
    );
    // Doubles the cross products, two limbs per index; they sum below `2^(128 A - 1)`.
    let mut below = 0;
    A::each(
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
    A::each(
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
/// An unoptimized build calls the kernel: inlined, every row keeps its own stack slots in the
/// caller's frame, and a 4096-bit product inlines dozens of kernels.
#[cfg_attr(debug_assertions, inline(never))]
#[cfg_attr(not(debug_assertions), inline(always))]
pub(crate) fn basecase(out: &mut [u64], a: &[u64], b: &[u64]) {
    let (long, short) = if a.len() >= b.len() { (a, b) } else { (b, a) };
    for (index, &multiplier) in short.iter().enumerate() {
        let (_, window) = split_mut(out, index);
        let row = Row::new(window, long, multiplier, index == 0);
        if row.len > GROUPED_ROW {
            row.run_chain();
        } else {
            // The length is a constant where the product is inlined: the copies past it fold.
            row.run_copies::<GroupedRow>();
        }
    }
}

/// The longest row that runs as two carry chains in groups; longer rows run one chain of
/// multiply-accumulate steps.
type GroupedRow = U16;

/// [`GroupedRow`] as a number.
const GROUPED_ROW: usize = GroupedRow::USIZE;

/// The products of a group of a [`Row`], which stay in registers.
const ROW_GROUP: usize = 4;

/// `long * multiplier` added to `window`, cut at its end, or written there when `first`. The
/// limb above the products is fresh: the rows before it in this output reach below it.
struct Row<'a> {
    window: &'a mut [u64],
    long: &'a [u64],
    multiplier: u64,
    first: bool,
    /// The limbs of the row: those of `long` that `window` holds.
    len: usize,
    /// A full row ends in a limb above the window's products; a cut row's last product lands on
    /// the window's top limb, which takes its low half.
    full: bool,
    /// The products of the current group, each low half plus the high half before it.
    group: [u64; ROW_GROUP],
    /// The carry of the chain of products, and the high half of the last product.
    carry: u64,
    high: u64,
    /// The carry of the chain that adds the groups into the window.
    sum_carry: u64,
}

impl<'a> Row<'a> {
    #[inline(always)]
    fn new(window: &'a mut [u64], long: &'a [u64], multiplier: u64, first: bool) -> Self {
        let len = long.len().min(window.len());
        Self {
            full: window.len() > len,
            window,
            long,
            multiplier,
            first,
            len,
            group: [0; ROW_GROUP],
            carry: 0,
            high: 0,
            sum_carry: 0,
        }
    }

    /// One copy per limb of `Long`, which `long` fills or, for a row of slices, reaches.
    #[inline(always)]
    fn run_copies<Long: Indices>(mut self) {
        Long::each(
            #[inline(always)]
            |index| self.step(index),
        );
        self.finish();
    }

    /// One chain of multiply-accumulate steps, for rows longer than [`GROUPED_ROW`].
    #[inline(always)]
    fn run_chain(self) {
        let (row, above) = split_mut(self.window, self.len);
        let carry = if self.first {
            mul_row(row, self.long, self.multiplier)
        } else {
            add_mul_row(row, self.long, self.multiplier)
        };
        if let Some(slot) = above.first_mut() {
            *slot = carry;
        }
    }

    /// The product of limb `index` into the group; at the end of a group, the group added into
    /// the window.
    #[inline(always)]
    fn step(&mut self, index: usize) {
        if index >= self.len {
            return;
        }
        if let (Some(&limb), Some(slot)) =
            (self.long.get(index), self.group.get_mut(index % ROW_GROUP))
        {
            if self.full || index + 1 < self.len {
                let product = u128::from(limb) * u128::from(self.multiplier);
                (*slot, self.carry) = (product as u64).add_carry(self.high, self.carry);
                self.high = (product >> 64) as u64;
            } else {
                *slot = limb
                    .wrapping_mul(self.multiplier)
                    .wrapping_add(self.high)
                    .wrapping_add(self.carry);
            }
        }
        if index % ROW_GROUP == ROW_GROUP - 1 || index + 1 == self.len {
            let start = index - index % ROW_GROUP;
            let (_, slots) = split_mut(self.window, start);
            let group = self.group.iter().take(index % ROW_GROUP + 1);
            for (slot, &product) in slots.iter_mut().zip(group) {
                if self.first {
                    *slot = product;
                } else {
                    (*slot, self.sum_carry) = slot.add_carry(product, self.sum_carry);
                }
            }
        }
    }

    /// The limb above a full row. A high half is at most `2^64 - 2`, and the sum of the rows so
    /// far fits below it.
    #[inline(always)]
    fn finish(self) {
        if self.full
            && let Some(slot) = self.window.get_mut(self.len)
        {
            *slot = self.high + self.carry + self.sum_carry;
        }
    }
}

/// Drops the carry out of the output window. The window is the product width the caller chose:
/// for a wrapping product the carry is the part above it, for a full product it is zero.
#[inline(always)]
pub(crate) fn truncate_carry(_carry: u64) {}
