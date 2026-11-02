//! The primitives of the runtime kernels: limb loops unrolled in the source, carry chains on the
//! `adc` and `sbb` intrinsics on x86-64, and selections by `core::hint::select_unpredictable`.
//!
//! [`unroll!`] copies a loop body once per index below 16, each copy behind a test of its index
//! against the length. After monomorphization every length is a constant, so the copies past the
//! length fold away and the rest is straight-line code that one optimization pipeline turns into
//! clean carry chains. A loop carries the indices from 16 on, so every width works. Builds with
//! debug assertions, which do not optimize, run the loop alone.

/// The indices that [`unroll!`] copies; a loop runs the rest.
#[cfg(not(debug_assertions))]
pub(crate) const UNROLLED: usize = 16;

/// Runs `$body` with `$i` bound to each index of `$start..$len`, in order.
macro_rules! unroll {
    ($i:ident in $start:expr, $len:expr => $body:block) => {
        $crate::unrolled::unroll!(@copies $i, $start, $len, $body; 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15)
    };
    (@copies $i:ident, $start:expr, $len:expr, $body:block; $($k:literal)*) => {{
        let unroll_start: usize = $start;
        let unroll_len: usize = $len;
        #[cfg(not(debug_assertions))]
        let unroll_rest: usize = {
            $(
                if $crate::unrolled::covers(unroll_start, unroll_len, $k) {
                    let $i: usize = $k;
                    $body
                }
            )*
            $crate::unrolled::past_copies(unroll_start)
        };
        #[cfg(debug_assertions)]
        let unroll_rest: usize = unroll_start;
        let mut $i: usize = unroll_rest;
        while $i < unroll_len {
            $body
            $i += 1;
        }
    }};
}

pub(crate) use unroll;

/// Whether the copy of `index` runs for the range `start..len`.
#[cfg(not(debug_assertions))]
#[inline(always)]
pub(crate) const fn covers(start: usize, len: usize, index: usize) -> bool {
    start <= index && index < len
}

/// The first index that the loop after the copies runs for a range from `start`.
#[cfg(not(debug_assertions))]
#[inline(always)]
pub(crate) const fn past_copies(start: usize) -> usize {
    if start > UNROLLED { start } else { UNROLLED }
}

/// `a + b + carry` as `(sum, carry_out)`, with carries in `{0, 1}`.
#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub(crate) fn adc(carry: u8, a: u64, b: u64) -> (u64, u8) {
    let mut sum = 0;
    let carry = core::arch::x86_64::_addcarry_u64(carry, a, b, &mut sum);
    (sum, carry)
}

/// `a - b - borrow` as `(difference, borrow_out)`, with borrows in `{0, 1}`.
#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub(crate) fn sbb(borrow: u8, a: u64, b: u64) -> (u64, u8) {
    let mut difference = 0;
    let borrow = core::arch::x86_64::_subborrow_u64(borrow, a, b, &mut difference);
    (difference, borrow)
}

/// `a + b + carry` as `(sum, carry_out)`, with carries in `{0, 1}`.
#[cfg(not(target_arch = "x86_64"))]
#[inline(always)]
pub(crate) fn adc(carry: u8, a: u64, b: u64) -> (u64, u8) {
    let wide = u128::from(a) + u128::from(b) + u128::from(carry);
    (wide as u64, (wide >> 64) as u8)
}

/// `a - b - borrow` as `(difference, borrow_out)`, with borrows in `{0, 1}`.
#[cfg(not(target_arch = "x86_64"))]
#[inline(always)]
pub(crate) fn sbb(borrow: u8, a: u64, b: u64) -> (u64, u8) {
    let wide = u128::from(a)
        .wrapping_sub(u128::from(b))
        .wrapping_sub(u128::from(borrow));
    (wide as u64, (wide >> 127) as u8)
}

/// `a * b + c + d` as `(low, high)`; the sum is at most `2^128 - 1`.
#[inline(always)]
pub(crate) fn mac(a: u64, b: u64, c: u64, d: u64) -> (u64, u64) {
    let wide = u128::from(a) * u128::from(b) + u128::from(c) + u128::from(d);
    (wide as u64, (wide >> 64) as u64)
}

/// `a` when `condition` holds, else `b`, by a conditional move.
#[inline(always)]
pub(crate) fn select<T>(condition: bool, a: T, b: T) -> T {
    core::hint::select_unpredictable(condition, a, b)
}
