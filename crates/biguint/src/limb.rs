//! Single-limb carry primitives: the carry chain under every multi-limb add and subtract.
//!
//! Builds that enable `adx` on `x86_64` use the `adc`/`sbb` intrinsics. Every other build uses
//! two overflowing operations, which LLVM schedules better there.

/// All ones when `a < b`, zero otherwise: the high limb of the two-limb difference. A correction
/// that depends on the values applies such a mask: a branch on it mispredicts about once in two.
/// LLVM keeps this mask arithmetic; it turns a mask from a comparison into a select, and the select
/// into a branch inside a loop.
#[inline(always)]
pub(crate) const fn mask_below(a: u64, b: u64) -> u64 {
    ((a as u128).wrapping_sub(b as u128) >> 64) as u64
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

#[cfg(not(all(target_arch = "x86_64", target_feature = "adx")))]
use portable as arch;

#[cfg(any(test, not(all(target_arch = "x86_64", target_feature = "adx"))))]
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
