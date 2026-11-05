//! Montgomery reduction (Montgomery, 1985) with `R = 2^(64 * k)` for a `k`-limb odd modulus `m`:
//! `t * R^-1 mod m` for `t < m * R`. Each of the `k` rows adds the multiple `q * m` that clears the
//! row's bottom limb, `q = t_i * (-m^-1) mod 2^64`; the top `k` limbs then hold a value below `2m`.

use generic_array::{ArrayLength, GenericArray};

use crate::each::each;
use crate::kernel::{
    add_mul_row, reduced, split, split_mut, subtract_when_above, subtract_when_above_masked,
};
use crate::limb::{CarryingAdd, widening_mul};

/// `-m^-1 mod 2^64` for an odd `m` whose low limb is `low`. Each Newton step
/// `x <- x * (2 - m * x)` doubles the correct low bits; an odd `m` is its own inverse modulo 8.
#[inline(always)]
pub(crate) const fn negated_inverse(low: u64) -> u64 {
    let mut inverse = low;
    let mut step = 0;
    // 3, 6, 12, 24, 48 and 96 correct bits.
    while step < 5 {
        inverse = inverse.wrapping_mul(2u64.wrapping_sub(low.wrapping_mul(inverse)));
        step += 1;
    }
    inverse.wrapping_neg()
}

/// `t * R^-1 mod m` in `k = m.len()` limbs, for `t < m * R` in the low `2k` limbs of `t`, which the
/// reduction overwrites. `inverse` is `-m^-1 mod 2^64`.
///
/// Natively it inlines into the product, the square and the value of a residue, which return its
/// limbs: as a call, it wrote them to a copy that the caller read back with vector loads, which
/// waited for its scalar stores to retire.
#[cfg_attr(target_arch = "wasm32", inline)]
#[cfg_attr(not(target_arch = "wasm32"), inline(always))]
pub(crate) fn reduce<L: ArrayLength>(
    t: &mut [u64],
    modulus: &GenericArray<u64, L>,
    inverse: u64,
) -> GenericArray<u64, L> {
    let mut out = GenericArray::default();
    let above = fold::<L>(t, modulus, inverse);
    let (_, high) = split(t, L::USIZE);
    // Below `2m`, one subtraction at most brings the value below `m`. A carry out of the top limb
    // places it at or above `2^(64 * k)`, above `m`; the borrow cancels the carry.
    reduced(subtract_when_above(&mut out, high, modulus, above));
    out
}

/// [`reduce`] with its final subtraction under masks at every length: no branch and no memory
/// address depends on `t`.
#[inline]
pub(crate) fn reduce_masked<L: ArrayLength>(
    out: &mut [u64],
    t: &mut [u64],
    modulus: &GenericArray<u64, L>,
    inverse: u64,
) {
    let above = fold::<L>(t, modulus, inverse);
    let (_, high) = split(t, L::USIZE);
    subtract_when_above_masked(out, high, modulus, above);
}

/// The rows of [`reduce`]: adds to `t` the multiples of the modulus that clear its low `k` limbs,
/// which leaves the value below `2m` in the limbs above them, and returns the carry out of the top.
#[inline(always)]
fn fold<L: ArrayLength>(t: &mut [u64], modulus: &GenericArray<u64, L>, inverse: u64) -> u64 {
    let k = L::USIZE;
    // The carry out of the limb above each row: the rows' sums reach `2^(64 * (2k)) + ...`, one
    // bit above the `2k` limbs.
    let mut above = 0;
    each_row::<L>(
        #[inline(always)]
        |index| {
            let (_, window) = split_mut(t, index);
            if let Some(&low) = window.first() {
                let q = low.wrapping_mul(inverse);
                let (row, rest) = split_mut(window, k);
                let carry = add_row::<L>(row, modulus, q);
                if let Some(slot) = rest.first_mut() {
                    let (sum, first) = slot.add_carry(carry, 0);
                    let (sum, second) = sum.add_carry(above, 0);
                    *slot = sum;
                    // `t_(i+k) + carry + above` stays below `2^65`.
                    above = first + second;
                }
            }
        },
    );
    above
}

/// Runs `row` on each row index of a `L`-limb modulus: one copy per row up to [`BUFFERED_ROW`]
/// limbs, a loop beyond.
#[inline(always)]
fn each_row<L: ArrayLength>(mut row: impl FnMut(usize)) {
    if L::USIZE > BUFFERED_ROW {
        for index in 0..L::USIZE {
            row(index);
        }
    } else {
        each::<L>(row);
    }
}

/// The longest modulus whose rows run one copy per row and per limb, each row two carry chains;
/// longer moduli loop over rows of one chain of multiply-accumulate steps.
const BUFFERED_ROW: usize = 16;

/// Adds `modulus * q` to `row`, as long, and returns the carry limb above it: the multiply chain
/// into a buffer, then the add chain, for rows of up to [`BUFFERED_ROW`] limbs; the
/// multiply-accumulate loop beyond.
#[inline(always)]
fn add_row<L: ArrayLength>(row: &mut [u64], modulus: &GenericArray<u64, L>, q: u64) -> u64 {
    if L::USIZE > BUFFERED_ROW {
        return add_mul_row(row, modulus, q);
    }
    let mut products = GenericArray::<u64, L>::default();
    let (mut carry, mut high) = (0, 0);
    each::<L>(
        #[inline(always)]
        |index| {
            if let (Some(&limb), Some(slot)) = (modulus.get(index), products.get_mut(index)) {
                let product = widening_mul(limb, q);
                (*slot, carry) = (product as u64).add_carry(high, carry);
                high = (product >> 64) as u64;
            }
        },
    );
    // A high half is at most `2^64 - 2`.
    let top = high + carry;
    let mut carry = 0;
    each::<L>(
        #[inline(always)]
        |index| {
            if let (Some(slot), Some(&limb)) = (row.get_mut(index), products.get(index)) {
                (*slot, carry) = slot.add_carry(limb, carry);
            }
        },
    );
    // `row + modulus * q` is below `2^(64 * (len + 1))`.
    top + carry
}
