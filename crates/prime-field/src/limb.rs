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
#[cfg(not(any(
    target_arch = "x86_64",
    all(target_arch = "wasm32", not(wide_arithmetic))
)))]
#[inline(always)]
pub(crate) fn adc(carry: u8, a: u64, b: u64) -> (u64, u8) {
    let wide = u128::from(a) + u128::from(b) + u128::from(carry);
    (wide as u64, (wide >> 64) as u8)
}

/// `a + b + carry` as `(sum, carry_out)`, with carries in `{0, 1}`: two overflowing additions,
/// which WebAssembly without wide arithmetic runs as one comparison each, and the sum of their
/// carries. Their `|` took Firefox 56 cycles for a 381-bit difference, against 35.
#[cfg(all(target_arch = "wasm32", not(wide_arithmetic)))]
#[inline(always)]
pub(crate) fn adc(carry: u8, a: u64, b: u64) -> (u64, u8) {
    let (sum, first) = a.overflowing_add(b);
    let (sum, second) = sum.overflowing_add(u64::from(carry));
    (sum, u8::from(first) + u8::from(second))
}

/// `a - b - borrow` as `(difference, borrow_out)`, with borrows in `{0, 1}`: two overflowing
/// subtractions and the sum of their borrows, as [`adc`] adds.
#[cfg(all(target_arch = "wasm32", not(wide_arithmetic)))]
#[inline(always)]
pub(crate) fn sbb(borrow: u8, a: u64, b: u64) -> (u64, u8) {
    let (difference, first) = a.overflowing_sub(b);
    let (difference, second) = difference.overflowing_sub(u64::from(borrow));
    (difference, u8::from(first) + u8::from(second))
}

/// `a - b - borrow` as `(difference, borrow_out)`, with borrows in `{0, 1}`.
#[cfg(not(any(
    target_arch = "x86_64",
    all(target_arch = "wasm32", not(wide_arithmetic))
)))]
#[inline(always)]
pub(crate) fn sbb(borrow: u8, a: u64, b: u64) -> (u64, u8) {
    let wide = u128::from(a)
        .wrapping_sub(u128::from(b))
        .wrapping_sub(u128::from(borrow));
    (wide as u64, (wide >> 127) as u8)
}

/// [`adc`] for a constant `b`: on WebAssembly without wide arithmetic and `b < 2^64 - 1`, `a` plus
/// `b + carry`, a selection between two constants, and one comparison for the carry out; elsewhere
/// [`adc`]. The variable-time BN254 sum, whose subtraction of `p` runs [`sbb_constant`], took
/// Chrome 13.9 cycles and Firefox 14.7 against 16.8 with [`sbb`].
#[inline(always)]
pub(crate) fn adc_constant(carry: u8, a: u64, b: u64) -> (u64, u8) {
    if cfg!(all(target_arch = "wasm32", not(wide_arithmetic))) && b != u64::MAX {
        let addend = b + u64::from(carry);
        let (sum, out) = a.overflowing_add(addend);
        (sum, u8::from(out))
    } else {
        adc(carry, a, b)
    }
}

/// [`adc`] as `a` plus the sum `b + carry`, on WebAssembly without wide arithmetic: that sum wraps
/// to zero only for `b = 2^64 - 1` and a carry, which the second comparison adds; elsewhere
/// [`adc`]. MEASURE.
#[inline(always)]
pub(crate) fn adc_sum(carry: u8, a: u64, b: u64) -> (u64, u8) {
    if cfg!(all(target_arch = "wasm32", not(wide_arithmetic))) {
        let addend = b.wrapping_add(u64::from(carry));
        let (sum, out) = a.overflowing_add(addend);
        (sum, u8::from(out) + u8::from(addend < b))
    } else {
        adc(carry, a, b)
    }
}

/// [`sbb`] for a constant `b`: `a` less `b + borrow`, as [`adc_constant`] adds.
#[inline(always)]
pub(crate) fn sbb_constant(borrow: u8, a: u64, b: u64) -> (u64, u8) {
    if cfg!(all(target_arch = "wasm32", not(wide_arithmetic))) && b != u64::MAX {
        let subtrahend = b + u64::from(borrow);
        let (difference, out) = a.overflowing_sub(subtrahend);
        (difference, u8::from(out))
    } else {
        sbb(borrow, a, b)
    }
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

/// A word as the sum of at most three signed powers of two `±2^e`, `e <= 64`: its non-adjacent
/// form, the signed-binary form of fewest terms (Reitwiesner, 1960).
#[derive(Clone, Copy)]
pub(crate) struct SignedPowers {
    /// The terms from the lowest power up; the first `len` hold.
    terms: [SignedPower; 3],
    /// The terms that hold, at most three.
    len: usize,
}

/// One term `±2^exponent` of [`SignedPowers`].
#[derive(Clone, Copy)]
struct SignedPower {
    exponent: u32,
    negative: bool,
}

impl SignedPowers {
    /// The form of `value`, or `None` where it takes more than three terms.
    #[expect(
        clippy::indexing_slicing,
        reason = "const evaluation, with the index checked against the length"
    )]
    pub(crate) const fn of(value: u64) -> Option<Self> {
        let unused = SignedPower {
            exponent: 0,
            negative: false,
        };
        let mut form = Self {
            terms: [unused; 3],
            len: 0,
        };
        let mut rest = value as u128;
        let mut exponent = 0;
        while rest != 0 {
            if rest & 1 == 1 {
                // `rest = 3 mod 4` takes `-1`, which clears the next bit as well.
                let negative = rest & 2 == 2;
                if form.len == form.terms.len() {
                    return None;
                }
                form.terms[form.len] = SignedPower { exponent, negative };
                form.len += 1;
                rest = if negative { rest + 1 } else { rest - 1 };
            }
            rest >>= 1;
            exponent += 1;
        }
        Some(form)
    }

    /// The number of terms.
    pub(crate) const fn terms(self) -> usize {
        self.len
    }

    /// `m` times the word, as `(low, high)`, by shifts, additions and subtractions.
    #[inline(always)]
    pub(crate) fn mul(self, m: u64) -> (u64, u64) {
        let (mut low, mut high) = (0, 0);
        for term in self.terms.iter().take(self.len) {
            let (term_low, term_high) = match term.exponent {
                0 => (m, 0),
                64 => (0, m),
                exponent => (m << exponent, m >> (64 - exponent)),
            };
            if term.negative {
                let (difference, borrow) = sbb(0, low, term_low);
                (low, high) = (difference, sbb(borrow, high, term_high).0);
            } else {
                let (sum, carry) = adc(0, low, term_low);
                (low, high) = (sum, adc(carry, high, term_high).0);
            }
        }
        (low, high)
    }
}
