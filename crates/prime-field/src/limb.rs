//! The single-limb primitives of the runtime kernels: carry chains on the `adc` and `sbb`
//! intrinsics on x86-64, and selections by `core::hint::select_unpredictable`.

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

/// `a + b + carry` as `(sum, carry_out)`, with carries in `{0, 1}`: one `u128` sum, which wide
/// arithmetic runs as `i64.add128`.
#[cfg(not(any(target_arch = "x86_64", all(target_arch = "wasm32", not(wide_arithmetic)))))]
#[inline(always)]
pub(crate) fn adc(carry: u8, a: u64, b: u64) -> (u64, u8) {
    let wide = u128::from(a) + u128::from(b) + u128::from(carry);
    (wide as u64, (wide >> 64) as u8)
}

/// `a + b + carry` as `(sum, carry_out)`, with carries in `{0, 1}`: two overflowing additions,
/// which WebAssembly without wide arithmetic runs as one comparison each.
#[cfg(all(target_arch = "wasm32", not(wide_arithmetic)))]
#[inline(always)]
pub(crate) fn adc(carry: u8, a: u64, b: u64) -> (u64, u8) {
    let (sum, first) = a.overflowing_add(b);
    let (sum, second) = sum.overflowing_add(u64::from(carry));
    (sum, u8::from(first | second))
}

/// `a - b - borrow` as `(difference, borrow_out)`, with borrows in `{0, 1}`: two overflowing
/// subtractions, which WebAssembly without wide arithmetic runs as one comparison each.
#[cfg(all(target_arch = "wasm32", not(wide_arithmetic)))]
#[inline(always)]
pub(crate) fn sbb(borrow: u8, a: u64, b: u64) -> (u64, u8) {
    let (difference, first) = a.overflowing_sub(b);
    let (difference, second) = difference.overflowing_sub(u64::from(borrow));
    (difference, u8::from(first | second))
}

/// `a - b - borrow` as `(difference, borrow_out)`, with borrows in `{0, 1}`.
#[cfg(not(any(target_arch = "x86_64", all(target_arch = "wasm32", not(wide_arithmetic)))))]
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
    crate::kernel::mul_add(a, b, c, d)
}

/// `a` when `condition` holds, else `b`, by a conditional move.
#[inline(always)]
pub(crate) fn select<T>(condition: bool, a: T, b: T) -> T {
    core::hint::select_unpredictable(condition, a, b)
}
