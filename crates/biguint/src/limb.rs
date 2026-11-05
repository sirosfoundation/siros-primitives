//! Single-limb carry primitives: the carry chain under every multi-limb add and subtract.
//!
//! Builds that enable `adx` on `x86_64` use the `adc`/`sbb` intrinsics, and WebAssembly builds with
//! wide arithmetic `i64.add128` and `i64.sub128`. Every other build uses two overflowing
//! operations, which LLVM schedules better there.

/// All ones when `a < b`, zero otherwise: the high limb of the two-limb difference. A correction
/// that depends on the values applies such a mask: a branch on it mispredicts about once in two.
/// LLVM keeps this mask arithmetic; it turns a mask from a comparison into a select, and the select
/// into a branch inside a loop.
#[inline(always)]
pub(crate) const fn mask_below(a: u64, b: u64) -> u64 {
    ((a as u128).wrapping_sub(b as u128) >> 64) as u64
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

/// `a * b + c + d` as `(low, high)`, which is at most `2^128 - 1`, as one `u128` expression: the
/// form that LLVM schedules best in straight-line code, where it sees every step of an unrolled
/// product at once.
#[cfg(not(all(target_arch = "wasm32", not(wide_arithmetic))))]
#[inline(always)]
pub(crate) const fn mul_add(a: u64, b: u64, c: u64, d: u64) -> (u64, u64) {
    let wide = a as u128 * b as u128 + c as u128 + d as u128;
    (wide as u64, (wide >> 64) as u64)
}

/// `a * b + c + d` as `(low, high)`, which is at most `2^128 - 1`.
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

/// Marks the path that calls it as rare, which LLVM then branches around. A rare correction without
/// it became conditional moves that every division step ran: 8 of the 64 instructions per quotient
/// limb of a 256-bit modular product.
#[cold]
#[inline(always)]
pub(crate) const fn rare() {}

/// Add-with-carry on a `u64` limb.
pub(crate) trait CarryingAdd: Sized {
    /// `self + rhs + carry` as `(sum mod 2^64, carry_out)`, with carries in `{0, 1}`.
    fn add_carry(self, rhs: Self, carry: Self) -> (Self, Self);
}

/// Subtract-with-borrow on a `u64` limb.
pub(crate) trait BorrowingSub: Sized {
    /// `self - rhs - borrow` as `(difference mod 2^64, borrow_out)`, with borrows in `{0, 1}`.
    fn sub_borrow(self, rhs: Self, borrow: Self) -> (Self, Self);
}

impl CarryingAdd for u64 {
    #[inline(always)]
    fn add_carry(self, rhs: u64, carry: u64) -> (u64, u64) {
        let (sum, carry_out) = arch::add_carry(carry as u8, self, rhs);
        (sum, u64::from(carry_out))
    }
}

impl BorrowingSub for u64 {
    #[inline(always)]
    fn sub_borrow(self, rhs: u64, borrow: u64) -> (u64, u64) {
        let (difference, borrow_out) = arch::sub_borrow(borrow as u8, self, rhs);
        (difference, u64::from(borrow_out))
    }
}

#[cfg(all(target_arch = "x86_64", target_feature = "adx"))]
mod arch {
    use core::arch::x86_64::{_addcarry_u64, _subborrow_u64};

    #[inline(always)]
    pub(super) fn add_carry(carry: u8, lhs: u64, rhs: u64) -> (u64, u8) {
        let mut sum = 0;
        let carry_out = _addcarry_u64(carry, lhs, rhs, &mut sum);
        (sum, carry_out)
    }

    #[inline(always)]
    pub(super) fn sub_borrow(borrow: u8, lhs: u64, rhs: u64) -> (u64, u8) {
        let mut difference = 0;
        let borrow_out = _subborrow_u64(borrow, lhs, rhs, &mut difference);
        (difference, borrow_out)
    }
}

#[cfg(not(any(all(target_arch = "x86_64", target_feature = "adx"), wide_arithmetic)))]
use portable as arch;

#[cfg(wide_arithmetic)]
use wide as arch;

/// The carry chain of WebAssembly builds with wide arithmetic: one `u128` sum or difference,
/// which runs as `i64.add128` or `i64.sub128`.
#[cfg(wide_arithmetic)]
mod wide {
    #[inline(always)]
    pub(super) fn add_carry(carry: u8, lhs: u64, rhs: u64) -> (u64, u8) {
        let sum = u128::from(lhs) + u128::from(rhs) + u128::from(carry);
        (sum as u64, (sum >> 64) as u8)
    }

    #[inline(always)]
    pub(super) fn sub_borrow(borrow: u8, lhs: u64, rhs: u64) -> (u64, u8) {
        let difference = u128::from(lhs)
            .wrapping_sub(u128::from(rhs))
            .wrapping_sub(u128::from(borrow));
        (difference as u64, (difference >> 127) as u8)
    }
}

#[cfg(any(
    test,
    not(any(all(target_arch = "x86_64", target_feature = "adx"), wide_arithmetic))
))]
mod portable {
    #[inline(always)]
    pub(super) fn add_carry(carry: u8, lhs: u64, rhs: u64) -> (u64, u8) {
        let (sum, carry_a) = lhs.overflowing_add(rhs);
        let (sum, carry_b) = sum.overflowing_add(u64::from(carry));
        (sum, u8::from(carry_a | carry_b))
    }

    #[inline(always)]
    pub(super) fn sub_borrow(borrow: u8, lhs: u64, rhs: u64) -> (u64, u8) {
        let (difference, borrow_a) = lhs.overflowing_sub(rhs);
        let (difference, borrow_b) = difference.overflowing_sub(u64::from(borrow));
        (difference, u8::from(borrow_a | borrow_b))
    }
}

#[cfg(test)]
mod tests;
