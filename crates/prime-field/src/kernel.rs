//! Limb kernels over little-endian `u64` slices. Every kernel is a `const fn`: the derived
//! constants of a field (`R`, `R^2`, the square-root exponents, the quadratic non-residue) come
//! from the same code that runs at run time.
//!
//! A slice reads as zero beyond its length, and a write beyond the length is dropped. After
//! monomorphization every length and index is a constant, so each access folds to a register
//! move and the loops below unroll. The kernels hold no scratch of their own: the caller passes
//! every buffer, sized by its types.

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

/// `a * b + c + d` as `(low, high)`; the sum is at most `2^128 - 1`.
#[inline(always)]
pub(crate) const fn mul_add(a: u64, b: u64, c: u64, d: u64) -> (u64, u64) {
    let wide = a as u128 * b as u128 + c as u128 + d as u128;
    (wide as u64, (wide >> 64) as u64)
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

/// `out = a` when `condition` holds, else `b`, without a branch: the mask form, which the
/// compiler lowers to one conditional move per limb.
#[inline(always)]
pub(crate) const fn select(out: &mut [u64], condition: bool, a: &[u64], b: &[u64]) {
    let mask = 0u64.wrapping_sub(condition as u64);
    let mut index = 0;
    while index < out.len() {
        set(
            out,
            index,
            (limb(a, index) & mask) | (limb(b, index) & !mask),
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

/// `out = value - p` when `top:value >= p`, else `value`, for `top:value < 2p` held in `out`.
/// `scratch` has the length of `out`.
#[inline(always)]
pub(crate) const fn reduce_once(out: &mut [u64], scratch: &mut [u64], top: u64, p: &[u64]) {
    let borrow = sub(scratch, out, p);
    let take = (top != 0) | (borrow == 0);
    let mask = 0u64.wrapping_sub(take as u64);
    let mut index = 0;
    while index < out.len() {
        set(
            out,
            index,
            (limb(scratch, index) & mask) | (limb(out, index) & !mask),
        );
        index += 1;
    }
}

/// `out = a + b mod p` for `a, b < p`. `spare` tells that `p` leaves the top bit of the top limb
/// clear, so `a + b` cannot carry out. `scratch` has the length of `out`.
#[inline(always)]
pub(crate) const fn add_mod(
    out: &mut [u64],
    scratch: &mut [u64],
    a: &[u64],
    b: &[u64],
    p: &[u64],
    spare: bool,
) {
    let carry = add(out, a, b);
    reduce_once(out, scratch, if spare { 0 } else { carry }, p);
}

/// `out = a - b mod p` for `a, b < p`. `scratch` has the length of `out`.
#[inline(always)]
pub(crate) const fn sub_mod(out: &mut [u64], scratch: &mut [u64], a: &[u64], b: &[u64], p: &[u64]) {
    let borrow = sub(out, a, b);
    fits(add(scratch, out, p));
    let mask = 0u64.wrapping_sub(borrow);
    let mut index = 0;
    while index < out.len() {
        set(
            out,
            index,
            (limb(scratch, index) & mask) | (limb(out, index) & !mask),
        );
        index += 1;
    }
}

/// `out = -a mod p` for `a < p`.
#[inline(always)]
pub(crate) const fn neg_mod(out: &mut [u64], a: &[u64], p: &[u64]) {
    fits(sub(out, p, a));
    let mask = 0u64.wrapping_sub(!is_zero(a) as u64);
    let mut index = 0;
    while index < out.len() {
        set(out, index, limb(out, index) & mask);
        index += 1;
    }
}

/// Montgomery multiplication by coarsely integrated operand scanning (Koç, Acar and Kaliski,
/// 1996): `out = a * b / 2^(64 L) mod p` for `a, b < p` of `L` limbs, with `inv = -p^-1 mod
/// 2^64`. `spare` tells that `p < 2^(64 L - 1)`, which keeps the accumulator within `L + 1`
/// limbs and drops the second carry limb, as in gnark-crypto's and ark-ff's no-carry
/// multiplication. `scratch` has the length of `out`.
#[inline(always)]
pub(crate) const fn cios(
    out: &mut [u64],
    scratch: &mut [u64],
    a: &[u64],
    b: &[u64],
    p: &[u64],
    inv: u64,
    spare: bool,
) {
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
    reduce_once(out, scratch, top, p);
}

/// Montgomery reduction of the `2 L`-limb `wide` by separated operand scanning: `out = wide /
/// 2^(64 L) mod p` for `wide < p * 2^(64 L)`. `spare` as in [`cios`]; `scratch` has the length of
/// `out`.
#[inline(always)]
pub(crate) const fn redc(
    out: &mut [u64],
    scratch: &mut [u64],
    wide: &mut [u64],
    p: &[u64],
    inv: u64,
    spare: bool,
) {
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
    reduce_once(out, scratch, if spare { 0 } else { above }, p);
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
