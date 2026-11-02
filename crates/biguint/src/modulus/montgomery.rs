//! Montgomery reduction (Montgomery, 1985) with `R = 2^(64 * k)` for a `k`-limb odd modulus `m`:
//! `t * R^-1 mod m` for `t < m * R`. Each of the `k` rows adds the multiple `q * m` that clears the
//! row's bottom limb, `q = t_i * (-m^-1) mod 2^64`; the top `k` limbs then hold a value below `2m`.

use crate::kernel::{add_mul_row, fits, reduced, split, split_mut, sub_assign, subtract_if_above};
use crate::limb::CarryingAdd;
use crate::unroll::{UNROLLED, unroll};

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

/// Writes `t * R^-1 mod m` to `out`, `k = m.len()` limbs, for `t < m * R` in the low `2k` limbs of
/// `t`, which the reduction overwrites. `inverse` is `-m^-1 mod 2^64`.
///
/// An unoptimized build calls the kernel, as it does [`crate::mul::basecase`].
#[cfg_attr(debug_assertions, inline(never))]
#[cfg_attr(not(debug_assertions), inline(always))]
pub(crate) fn reduce(out: &mut [u64], t: &mut [u64], modulus: &[u64], inverse: u64) {
    let k = modulus.len();
    // The carry out of the limb above each row: the rows' sums reach `2^(64 * (2k)) + ...`, one
    // bit above the `2k` limbs.
    let mut above = 0;
    unroll!(index in 0..k => {
        let (_, window) = split_mut(t, index);
        if let Some(&low) = window.first() {
            let q = low.wrapping_mul(inverse);
            let (row, rest) = split_mut(window, k);
            let carry = add_row(row, modulus, q);
            if let Some(slot) = rest.first_mut() {
                let (sum, first) = slot.add_carry(carry, 0);
                let (sum, second) = sum.add_carry(above, 0);
                *slot = sum;
                // `t_(i+k) + carry + above` stays below `2^65`.
                above = first + second;
            }
        }
    });
    let (_, high) = split(t, k);
    for (slot, &limb) in out.iter_mut().zip(high) {
        *slot = limb;
    }
    if above != 0 {
        // The value is at least `2^(64 * k)`, above `m`; the borrow cancels the carry.
        fits(sub_assign(out, modulus));
    } else {
        // Below `2m`, one subtraction at most brings it below `m`.
        reduced(subtract_if_above(out, modulus));
    }
}

/// Adds `modulus * q` to `row`, as long, and returns the carry limb above it: the multiply chain
/// into a buffer, then the add chain, for rows of up to [`UNROLLED`] limbs; the multiply-accumulate
/// loop beyond.
#[inline(always)]
fn add_row(row: &mut [u64], modulus: &[u64], q: u64) -> u64 {
    let len = modulus.len().min(row.len());
    if len > UNROLLED {
        return add_mul_row(row, modulus, q);
    }
    let mut products = [0u64; UNROLLED];
    let (mut carry, mut high) = (0, 0);
    unroll!(index in 0..len => {
        if let (Some(&limb), Some(slot)) = (modulus.get(index), products.get_mut(index)) {
            let product = u128::from(limb) * u128::from(q);
            (*slot, carry) = (product as u64).add_carry(high, carry);
            high = (product >> 64) as u64;
        }
    });
    // A high half is at most `2^64 - 2`.
    let top = high + carry;
    let mut carry = 0;
    unroll!(index in 0..len => {
        if let (Some(slot), Some(&limb)) = (row.get_mut(index), products.get(index)) {
            (*slot, carry) = slot.add_carry(limb, carry);
        }
    });
    // `row + modulus * q` is below `2^(64 * (len + 1))`.
    top + carry
}
