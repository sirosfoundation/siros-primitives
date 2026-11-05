//! Limb kernels over little-endian `u64` slices. Every kernel is a `const fn`: the derived
//! constants of a field (`R`, `R^2`, the square-root exponents, the quadratic non-residue) come
//! from the same code that runs at run time.
//!
//! A slice reads as zero beyond its length, and a write beyond the length is dropped. After
//! monomorphization every length and index is a constant, so each access folds to a register
//! move and the loops below unroll. The kernels hold no scratch of their own: the caller passes
//! every buffer, sized by its types.

use core::hint::black_box;

/// Drops the carry or borrow out of a result that is known to fit its limbs, or whose wrap
/// cancels an earlier borrow.
#[inline(always)]
pub(crate) const fn fits(_carry: u64) {}

/// Limb `index` of `limbs`; `0` at and beyond the length.
#[inline(always)]
#[expect(
    clippy::indexing_slicing,
    reason = "the index is checked against the length"
)]
pub(crate) const fn limb(limbs: &[u64], index: usize) -> u64 {
    if index < limbs.len() { limbs[index] } else { 0 }
}

/// Writes `value` to limb `index` of `limbs`, within the length.
#[inline(always)]
#[expect(
    clippy::indexing_slicing,
    reason = "the index is checked against the length"
)]
pub(crate) const fn set(limbs: &mut [u64], index: usize, value: u64) {
    if index < limbs.len() {
        limbs[index] = value;
    }
}

/// `a * b`, the full 128-bit product.
///
/// WebAssembly without wide arithmetic has no widening product, and LLVM calls `__multi3` for a
/// `u128` one; four `i64.mul` of 32-bit halves take its place. On x86-64, LLVM does not fuse the
/// split product back into one `mul`.
#[inline(always)]
pub(crate) const fn widening_mul(a: u64, b: u64) -> u128 {
    let (low, high) = mul_add(a, b, 0, 0);
    (low as u128) | ((high as u128) << 64)
}

/// `a * b + c + d` as `(low, high)`; the sum is at most `2^128 - 1`.
#[cfg(not(all(target_arch = "wasm32", not(wide_arithmetic))))]
#[inline(always)]
pub(crate) const fn mul_add(a: u64, b: u64, c: u64, d: u64) -> (u64, u64) {
    let wide = a as u128 * b as u128 + c as u128 + d as u128;
    (wide as u64, (wide >> 64) as u64)
}

/// `a * b + c + d` as `(low, high)`; the sum is at most `2^128 - 1`.
#[cfg(all(target_arch = "wasm32", not(wide_arithmetic)))]
#[inline(always)]
pub(crate) const fn mul_add(a: u64, b: u64, c: u64, d: u64) -> (u64, u64) {
    mul_add_halves(a, b, c, d)
}

/// `a * b + c + d` as `(low, high)` from the four products of 32-bit halves, each partial sum below
/// `2^64`: `(2^32 - 1)^2` plus two terms below `2^32`.
#[cfg(any(test, all(target_arch = "wasm32", not(wide_arithmetic))))]
#[inline(always)]
const fn mul_add_halves(a: u64, b: u64, c: u64, d: u64) -> (u64, u64) {
    const HALF: u64 = 0xFFFF_FFFF;
    let (a0, a1, b0, b1) = (a & HALF, a >> 32, b & HALF, b >> 32);
    let low = a0 * b0 + (c & HALF) + (d & HALF);
    let middle = a1 * b0 + (low >> 32) + (c >> 32);
    let cross = a0 * b1 + (middle & HALF) + (d >> 32);
    let high = a1 * b1 + (middle >> 32) + (cross >> 32);
    ((cross << 32) | (low & HALF), high)
}

/// `a + b + carry` as `(sum, carry_out)`, with carries in `{0, 1}`.
#[inline(always)]
pub(crate) const fn add_carry(a: u64, b: u64, carry: u64) -> (u64, u64) {
    let (sum, first) = a.overflowing_add(b);
    let (sum, second) = sum.overflowing_add(carry);
    (sum, (first | second) as u64)
}

/// `a - b - borrow` as `(difference, borrow_out)`, with borrows in `{0, 1}`.
#[inline(always)]
pub(crate) const fn sub_borrow(a: u64, b: u64, borrow: u64) -> (u64, u64) {
    let (difference, first) = a.overflowing_sub(b);
    let (difference, second) = difference.overflowing_sub(borrow);
    (difference, (first | second) as u64)
}

/// `out = a + b` over the length of `out`; returns the carry out of the top limb.
#[inline(always)]
pub(crate) const fn add(out: &mut [u64], a: &[u64], b: &[u64]) -> u64 {
    let mut carry = 0;
    let mut index = 0;
    while index < out.len() {
        let (sum, carry_out) = add_carry(limb(a, index), limb(b, index), carry);
        set(out, index, sum);
        carry = carry_out;
        index += 1;
    }
    carry
}

/// `out = a - b` over the length of `out`; returns the borrow out of the top limb.
#[inline(always)]
pub(crate) const fn sub(out: &mut [u64], a: &[u64], b: &[u64]) -> u64 {
    let mut borrow = 0;
    let mut index = 0;
    while index < out.len() {
        let (difference, borrow_out) = sub_borrow(limb(a, index), limb(b, index), borrow);
        set(out, index, difference);
        borrow = borrow_out;
        index += 1;
    }
    borrow
}

/// `out -= a` over the length of `out`; returns the borrow out of the top limb.
#[inline(always)]
pub(crate) const fn sub_assign(out: &mut [u64], a: &[u64]) -> u64 {
    let mut borrow = 0;
    let mut index = 0;
    while index < out.len() {
        let (difference, borrow_out) = sub_borrow(limb(out, index), limb(a, index), borrow);
        set(out, index, difference);
        borrow = borrow_out;
        index += 1;
    }
    borrow
}

/// `out += a & mask` over the length of `out`; returns the carry out of the top limb.
#[inline(always)]
pub(crate) const fn add_assign_masked(out: &mut [u64], a: &[u64], mask: u64) -> u64 {
    let mut carry = 0;
    let mut index = 0;
    while index < out.len() {
        let (sum, carry_out) = add_carry(limb(out, index), limb(a, index) & mask, carry);
        set(out, index, sum);
        carry = carry_out;
        index += 1;
    }
    carry
}

/// All ones when `condition` holds, else zero. The mask passes through [`black_box`], so the
/// compiler cannot tell that it takes only those two values: an operand masked by it stays
/// bitwise arithmetic, where the compiler would otherwise branch on the condition, or rewrite an
/// addition of the mask as a subtraction that breaks the carry chain. crypto-bigint puts the same
/// barrier on its borrow masks. Every secret condition of the constant-time operations becomes
/// a mask here.
#[inline(always)]
pub(crate) const fn mask(condition: bool) -> u64 {
    black_box(0u64.wrapping_sub(condition as u64))
}

/// `out = a` when `condition` holds, else `b`, without a branch.
#[inline(always)]
pub(crate) const fn select(out: &mut [u64], condition: bool, a: &[u64], b: &[u64]) {
    let take = mask(condition);
    let mut index = 0;
    while index < out.len() {
        set(
            out,
            index,
            (limb(a, index) & take) | (limb(b, index) & !take),
        );
        index += 1;
    }
}

/// `out = a` over the length of `out`.
#[inline(always)]
pub(crate) const fn copy(out: &mut [u64], a: &[u64]) {
    let mut index = 0;
    while index < out.len() {
        set(out, index, limb(a, index));
        index += 1;
    }
}

/// Whether every limb is zero.
#[inline(always)]
pub(crate) const fn is_zero(a: &[u64]) -> bool {
    let mut any = 0;
    let mut index = 0;
    while index < a.len() {
        any |= limb(a, index);
        index += 1;
    }
    any == 0
}

/// Whether `a == b` over the length of `a`.
#[inline(always)]
pub(crate) const fn equal(a: &[u64], b: &[u64]) -> bool {
    let mut any = 0;
    let mut index = 0;
    while index < a.len() {
        any |= limb(a, index) ^ limb(b, index);
        index += 1;
    }
    any == 0
}

/// Whether `a < b` over the length of `a`, by the borrow of `a - b`.
#[inline(always)]
pub(crate) const fn below(a: &[u64], b: &[u64]) -> bool {
    let mut borrow = 0;
    let mut index = 0;
    while index < a.len() {
        borrow = sub_borrow(limb(a, index), limb(b, index), borrow).1;
        index += 1;
    }
    borrow != 0
}

/// The number of bits needed to write the value: `0` for zero.
#[inline(always)]
pub(crate) const fn bit_len(a: &[u64]) -> usize {
    let mut index = a.len();
    while index > 0 {
        index -= 1;
        let value = limb(a, index);
        if value != 0 {
            return index * 64 + 64 - value.leading_zeros() as usize;
        }
    }
    0
}

/// The index of the lowest set bit; `64 * len` for zero.
#[inline(always)]
pub(crate) const fn trailing_zeros(a: &[u64]) -> usize {
    let mut index = 0;
    while index < a.len() {
        let value = limb(a, index);
        if value != 0 {
            return index * 64 + value.trailing_zeros() as usize;
        }
        index += 1;
    }
    a.len() * 64
}

/// Bit `index` of `a`; `false` beyond the length.
#[inline(always)]
pub(crate) const fn bit(a: &[u64], index: usize) -> bool {
    (limb(a, index / 64) >> (index % 64)) & 1 != 0
}

/// Limb `index` of `a >> shift`.
#[inline(always)]
pub(crate) const fn shifted_limb(a: &[u64], shift: usize, index: usize) -> u64 {
    let limbs = shift / 64;
    let bits = shift % 64;
    let low = limb(a, index + limbs);
    if bits == 0 {
        low
    } else {
        (low >> bits) | (limb(a, index + limbs + 1) << (64 - bits))
    }
}

/// `out = a >> shift` over the length of `out`.
#[inline(always)]
pub(crate) const fn shr(out: &mut [u64], a: &[u64], shift: usize) {
    let mut index = 0;
    while index < out.len() {
        set(out, index, shifted_limb(a, shift, index));
        index += 1;
    }
}

/// Limb `index` of `a mod 2^bits`.
#[inline(always)]
pub(crate) const fn low_limb(a: &[u64], bits: usize, index: usize) -> u64 {
    match bits.checked_sub(index * 64) {
        None | Some(0) => 0,
        Some(kept) if kept >= 64 => limb(a, index),
        Some(kept) => limb(a, index) & ((1u64 << kept) - 1),
    }
}

/// `top:out mod p` for `top:out < 2p`, in place: `p` comes off, and comes back on under the
/// mask of a borrow that `top` does not absorb, which wraps the limbs back to `out`.
#[inline(always)]
pub(crate) const fn reduce_once(out: &mut [u64], top: u64, p: &[u64]) {
    let borrow = sub_assign(out, p);
    fits(add_assign_masked(out, p, mask((borrow != 0) & (top == 0))));
}

/// `out = a + b mod p` for `a, b < p`. `spare` tells that `p` leaves the top bit of the top limb
/// clear, so `a + b` cannot carry out.
#[inline(always)]
pub(crate) const fn add_mod(out: &mut [u64], a: &[u64], b: &[u64], p: &[u64], spare: bool) {
    let carry = add(out, a, b);
    reduce_once(out, if spare { 0 } else { carry }, p);
}

/// `out = a - b mod p` for `a, b < p`: `p` is added back under the mask of the borrow, and the
/// carry of that addition cancels the borrow.
#[inline(always)]
pub(crate) const fn sub_mod(out: &mut [u64], a: &[u64], b: &[u64], p: &[u64]) {
    let borrow = sub(out, a, b);
    fits(add_assign_masked(out, p, mask(borrow != 0)));
}

/// `out = -a mod p` for `a < p`: `0 - a`, which borrows exactly when `a` is not zero.
#[inline(always)]
pub(crate) const fn neg_mod(out: &mut [u64], a: &[u64], p: &[u64]) {
    sub_mod(out, &[], a, p);
}

/// Montgomery multiplication by coarsely integrated operand scanning (Koç, Acar and Kaliski,
/// 1996): `out = a * b / 2^(64 L) mod p` for `a, b < p` of `L` limbs, with `inv = -p^-1 mod
/// 2^64`. `spare` tells that `p < 2^(64 L - 1)`, which keeps the accumulator within `L + 1`
/// limbs and drops the second carry limb, as in gnark-crypto's and ark-ff's no-carry
/// multiplication.
#[inline(always)]
pub(crate) const fn cios(out: &mut [u64], a: &[u64], b: &[u64], p: &[u64], inv: u64, spare: bool) {
    let len = out.len();
    let mut top = 0u64;
    let mut above = 0u64;
    let mut index = 0;
    while index < len {
        set(out, index, 0);
        index += 1;
    }
    let mut i = 0;
    while i < len {
        let multiplier = limb(b, i);
        let mut carry = 0;
        let mut j = 0;
        while j < len {
            let (low, high) = mul_add(limb(a, j), multiplier, limb(out, j), carry);
            set(out, j, low);
            carry = high;
            j += 1;
        }
        let (sum, carry_out) = add_carry(top, carry, 0);
        top = sum;
        if !spare {
            above = carry_out;
        }
        let m = limb(out, 0).wrapping_mul(inv);
        let (_, mut carry) = mul_add(m, limb(p, 0), limb(out, 0), 0);
        let mut j = 1;
        while j < len {
            let (low, high) = mul_add(m, limb(p, j), limb(out, j), carry);
            set(out, j - 1, low);
            carry = high;
            j += 1;
        }
        let (sum, carry_out) = add_carry(top, carry, 0);
        set(out, len - 1, sum);
        top = if spare { 0 } else { above + carry_out };
        i += 1;
    }
    reduce_once(out, top, p);
}

/// Montgomery reduction of the `2 L`-limb `wide` by separated operand scanning: `out = wide /
/// 2^(64 L) mod p` for `wide < p * 2^(64 L)`. `spare` as in [`cios`].
#[inline(always)]
pub(crate) const fn redc(out: &mut [u64], wide: &mut [u64], p: &[u64], inv: u64, spare: bool) {
    let len = out.len();
    let mut above = 0u64;
    let mut i = 0;
    while i < len {
        let m = limb(wide, i).wrapping_mul(inv);
        let (_, mut carry) = mul_add(m, limb(p, 0), limb(wide, i), 0);
        let mut j = 1;
        while j < len {
            let (low, high) = mul_add(m, limb(p, j), limb(wide, i + j), carry);
            set(wide, i + j, low);
            carry = high;
            j += 1;
        }
        // The carry out of the row's top limb lands where the next row adds its own.
        let (sum, carry_out) = add_carry(limb(wide, i + len), carry, above);
        set(wide, i + len, sum);
        above = carry_out;
        i += 1;
    }
    let mut index = 0;
    while index < len {
        set(out, index, limb(wide, index + len));
        index += 1;
    }
    reduce_once(out, if spare { 0 } else { above }, p);
}

/// `wide = a * b`, schoolbook, over `a.len() + b.len()` limbs.
#[inline(always)]
pub(crate) const fn mul_wide(wide: &mut [u64], a: &[u64], b: &[u64]) {
    let mut index = 0;
    while index < wide.len() {
        set(wide, index, 0);
        index += 1;
    }
    let mut i = 0;
    while i < b.len() {
        let multiplier = limb(b, i);
        let mut carry = 0;
        let mut j = 0;
        while j < a.len() {
            let (low, high) = mul_add(limb(a, j), multiplier, limb(wide, i + j), carry);
            set(wide, i + j, low);
            carry = high;
            j += 1;
        }
        set(wide, i + a.len(), carry);
        i += 1;
    }
}

/// `wide = a * a` over `2 a.len()` limbs: each cross product once, doubled, plus the squares
/// on the diagonal.
#[inline(always)]
pub(crate) const fn square_wide(wide: &mut [u64], a: &[u64]) {
    let len = a.len();
    let mut index = 0;
    while index < wide.len() {
        set(wide, index, 0);
        index += 1;
    }
    let mut i = 0;
    while i < len {
        let multiplier = limb(a, i);
        let mut carry = 0;
        let mut j = i + 1;
        while j < len {
            let (low, high) = mul_add(limb(a, j), multiplier, limb(wide, i + j), carry);
            set(wide, i + j, low);
            carry = high;
            j += 1;
        }
        set(wide, i + len, carry);
        i += 1;
    }
    // Double the cross products; their sum is below `2^(128 L - 1)`, so no bit is lost.
    let mut index = 2 * len;
    while index > 1 {
        index -= 1;
        set(
            wide,
            index,
            (limb(wide, index) << 1) | (limb(wide, index - 1) >> 63),
        );
    }
    set(wide, 0, limb(wide, 0) << 1);
    let mut carry = 0;
    let mut i = 0;
    while i < len {
        let value = limb(a, i);
        let (low, high) = mul_add(value, value, limb(wide, 2 * i), carry);
        set(wide, 2 * i, low);
        let (sum, carry_out) = add_carry(limb(wide, 2 * i + 1), high, 0);
        set(wide, 2 * i + 1, sum);
        carry = carry_out;
        i += 1;
    }
}

/// One fold of the pseudo-Mersenne reduction for `p = 2^bits - deficit`: `out = (value mod
/// 2^bits) + (value >> bits) * deficit`, over the length of `out`.
#[inline(always)]
pub(crate) const fn fold(out: &mut [u64], value: &[u64], bits: usize, deficit: u64) {
    let mut carry = 0;
    let mut index = 0;
    while index < out.len() {
        let (sum, carry_out) = mul_add(
            shifted_limb(value, bits, index),
            deficit,
            low_limb(value, bits, index),
            carry,
        );
        set(out, index, sum);
        carry = carry_out;
        index += 1;
    }
}

/// `a mod divisor` for a nonzero `divisor`.
pub(crate) const fn rem_u64(a: &[u64], divisor: u64) -> u64 {
    let mut remainder = 0u64;
    let mut index = a.len();
    while index > 0 {
        index -= 1;
        let wide = ((remainder as u128) << 64) | limb(a, index) as u128;
        remainder = (wide % divisor as u128) as u64;
    }
    remainder
}

/// The Jacobi symbol `(a / p)` for an odd `p > a > 0`, by quadratic reciprocity: `1` for a
/// quadratic residue, `-1` for a non-residue, `0` when they share a factor.
pub(crate) const fn jacobi(a: u64, p: &[u64]) -> i32 {
    let low = limb(p, 0);
    let mut result = 1;
    let mut a = a;
    // `(2 / p) = -1` exactly when `p = 3, 5 mod 8`.
    while a.is_multiple_of(2) {
        a /= 2;
        if low % 8 == 3 || low % 8 == 5 {
            result = -result;
        }
    }
    if a == 1 {
        return result;
    }
    // `(a / p) = (p / a)` up to the sign `(-1)^((a - 1)(p - 1) / 4)`; `p mod a` then fits a limb.
    if a % 4 == 3 && low % 4 == 3 {
        result = -result;
    }
    let mut n = a;
    let mut a = rem_u64(p, n);
    loop {
        if a == 0 {
            return if n == 1 { result } else { 0 };
        }
        while a.is_multiple_of(2) {
            a /= 2;
            if n % 8 == 3 || n % 8 == 5 {
                result = -result;
            }
        }
        if a % 4 == 3 && n % 4 == 3 {
            result = -result;
        }
        let next = n % a;
        n = a;
        a = next;
    }
}

/// The first `len` limbs of `limbs`; all of them when `len` exceeds the length.
#[inline(always)]
pub(crate) const fn truncate(limbs: &[u64], len: usize) -> &[u64] {
    match limbs.split_at_checked(len) {
        Some((kept, _)) => kept,
        None => limbs,
    }
}

/// The first `len` limbs of `limbs`; all of them when `len` exceeds the length.
#[inline(always)]
pub(crate) const fn truncate_mut(limbs: &mut [u64], len: usize) -> &mut [u64] {
    let len = if len < limbs.len() { len } else { limbs.len() };
    limbs.split_at_mut(len).0
}

/// `-p^-1 mod 2^64` for odd `p`, by Newton's iteration from the inverse modulo `2^3`.
pub(crate) const fn inv64(p: u64) -> u64 {
    // `p * p == 1 mod 8` for odd `p`, so `p` is its own inverse modulo `2^3`; each step doubles
    // the correct bits: 3, 6, 12, 24, 48, 96.
    let mut inverse = p;
    let mut step = 0;
    while step < 5 {
        inverse = inverse.wrapping_mul(2u64.wrapping_sub(p.wrapping_mul(inverse)));
        step += 1;
    }
    inverse.wrapping_neg()
}

#[cfg(test)]
mod tests {
    use super::mul_add_halves;

    /// The split product of WebAssembly builds against `u128` arithmetic, on every target.
    #[test]
    fn split_product_matches_wide_arithmetic() {
        const EDGES: [u64; 7] = [
            0,
            1,
            2,
            u32::MAX as u64,
            1 << 63,
            u64::MAX,
            0xa5a5_a5a5_5a5a_5a5a,
        ];
        for a in EDGES {
            for b in EDGES {
                for c in EDGES {
                    for d in EDGES {
                        let wide = u128::from(a) * u128::from(b) + u128::from(c) + u128::from(d);
                        assert_eq!(
                            mul_add_halves(a, b, c, d),
                            (wide as u64, (wide >> 64) as u64)
                        );
                    }
                }
            }
        }
    }
}
