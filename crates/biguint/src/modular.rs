//! Modular addition, subtraction and negation, with one kernel: `a - b` is `a + TwosComplement(b)`,
//! and `-a` is `0 - a`. The addend's type selects the carry chain and the correction that brings
//! the sum below the modulus.

use crate::bits::Bits;
use crate::div::{DiscardQuotient, Top, divide, divide_preparing};
#[cfg(target_arch = "wasm32")]
use crate::kernel::{add_assign, compare, fits, sub_assign};
use crate::kernel::{
    add_unless_carry, add_unless_carry_masked, reduced, subtract_when_above_in_place,
    subtract_when_above_masked,
};
use crate::limb::rare_call;
use crate::nonzero::NonZeroUint;
use generic_array::GenericArray;

use crate::stream::{
    AddOp, LimbSource, Op2CarryIter, SubOp, TwosComplement, ZeroExtended, pull_limbs,
};
use crate::uint::Uint;

impl<N: Bits> Uint<N> {
    /// `(self + rhs) mod modulus`.
    #[inline(always)]
    #[must_use]
    pub fn add_mod(self, rhs: Self, modulus: NonZeroUint<N>) -> Self {
        below_modulus(
            self,
            rhs,
            modulus,
            #[inline(always)]
            |a, b, m| sum_mod(a, b.source(), m),
        )
    }

    /// `(self - rhs) mod modulus`.
    #[inline(always)]
    #[must_use]
    pub fn sub_mod(self, rhs: Self, modulus: NonZeroUint<N>) -> Self {
        below_modulus(
            self,
            rhs,
            modulus,
            #[inline(always)]
            |a, b, m| sum_mod(a, TwosComplement(b.source()), m),
        )
    }

    /// `-self mod modulus`.
    #[inline(always)]
    #[must_use]
    pub fn neg_mod(self, modulus: NonZeroUint<N>) -> Self {
        below_modulus(
            Self::ZERO,
            self,
            modulus,
            #[inline(always)]
            |zero, a, m| sum_mod(zero, TwosComplement(a.source()), m),
        )
    }
}

/// The most limbs whose operands [`below_modulus`] selects into registers.
const SELECTED_LIMBS: usize = 4;

/// `sum(a mod modulus, b mod modulus, modulus)`. An operand whose top limb is below the modulus's
/// is below the modulus; any other reduces by a division.
///
/// Up to [`SELECTED_LIMBS`] limbs, each operand is a select between itself and its remainder,
/// which stays in registers. Longer operands would go through copies: there, `sum` inlines twice,
/// once on the operands where they lie and once on their remainders. On Zen 3 the two copies of
/// `sum` take 18% to 25% fewer cycles than the selects from 384 bits up, and the selects take as
/// many or fewer up to 256 bits. On WebAssembly up to [`SELECTED_LIMBS`] limbs, one call reduces
/// both operands ([`remainders`]), or at one limb computes the whole sum ([`reduced_sum`]).
#[inline(always)]
fn below_modulus<N: Bits>(
    a: Uint<N>,
    b: Uint<N>,
    modulus: NonZeroUint<N>,
    sum: impl Fn(&Uint<N>, &Uint<N>, &Uint<N>) -> Uint<N>,
) -> Uint<N> {
    let m = &modulus.get();
    if N::LIMBS == 1 && cfg!(target_arch = "wasm32") {
        if below(&a, m) && below(&b, m) {
            return sum(&a, &b, m);
        }
        return reduced_sum(a, b, modulus, sum);
    }
    if N::LIMBS <= SELECTED_LIMBS && cfg!(target_arch = "wasm32") {
        let (a, b) = if below(&a, m) && below(&b, m) {
            (a, b)
        } else {
            remainders(a, b, modulus)
        };
        return sum(&a, &b, m);
    }
    if N::LIMBS <= SELECTED_LIMBS {
        let a = if below(&a, m) {
            a
        } else {
            remainder(a, modulus)
        };
        let b = if below(&b, m) {
            b
        } else {
            remainder(b, modulus)
        };
        sum(&a, &b, m)
    } else if below(&a, m) && below(&b, m) {
        sum(&a, &b, m)
    } else {
        sum(&remainder(a, modulus), &remainder(b, modulus), m)
    }
}

/// `sum(a mod modulus, b mod modulus, modulus)` as a call: the rare path of [`below_modulus`] on
/// WebAssembly at one limb, whose result returns in a register. Inlined, the division's code joins
/// the sum's and the browsers allocate the registers of the whole: the 64-bit modular sum took
/// Chrome 11 cycles inlined against 6.1 as a call. Two calls that each return a remainder put both
/// operands of the common path in memory: 5.0 cycles against 4.0 with this call.
#[inline(never)]
fn reduced_sum<N: Bits>(
    a: Uint<N>,
    b: Uint<N>,
    modulus: NonZeroUint<N>,
    sum: impl Fn(&Uint<N>, &Uint<N>, &Uint<N>) -> Uint<N>,
) -> Uint<N> {
    sum(&(a % modulus), &(b % modulus), &modulus.get())
}

/// `(a mod modulus, b mod modulus)` as one call: the rare path of [`below_modulus`] on WebAssembly
/// from two limbs. The common path keeps its operands in locals: the 128-bit modular sum took
/// Chrome 8.8 cycles against 12.2 with a call per operand. A call that returns the sum, as
/// [`reduced_sum`] does, merges the results of both paths in memory.
#[inline(never)]
fn remainders<N: Bits>(a: Uint<N>, b: Uint<N>, modulus: NonZeroUint<N>) -> (Uint<N>, Uint<N>) {
    (a % modulus, b % modulus)
}

/// Whether the top limb of `value` is below the top limb of `modulus`, which puts `value` below
/// `modulus`.
#[inline(always)]
fn below<N: Bits>(value: &Uint<N>, modulus: &Uint<N>) -> bool {
    top(value) < top(modulus)
}

/// The top limb.
#[inline(always)]
pub(crate) fn top<N: Bits>(value: &Uint<N>) -> u64 {
    value.limbs.last().copied().unwrap_or(0)
}

/// Whether the variable-time sums of `limbs` limbs branch on their correction. On WebAssembly the
/// difference and the selects of every limb cost more than a branch: the 512-bit sum took Chrome 48
/// cycles with the branch against 65 with the selects. Without wide arithmetic at 32 limbs, the
/// branch on the whole comparison mispredicts on half the sums of reduced operands, and the
/// correction of the top limbs ([`subtract_when_above_in_place`]) took Firefox 268 cycles against
/// 430 and Chrome 252 against 283.
#[cfg(target_arch = "wasm32")]
const fn branches(limbs: usize) -> bool {
    !(cfg!(not(wide_arithmetic)) && limbs == 32)
}

/// The second operand of [`sum_mod`], with the correction that brings the sum below the modulus.
pub(crate) trait Addend: LimbSource {
    /// The limbs of `a + addend` in `N` bits, and the carry out of the top limb.
    fn sum<N: Bits>(a: &Uint<N>, addend: Self) -> (GenericArray<u64, N::Limbs>, u64);

    /// Brings `sum`, whose carry out of the top limb is `carry`, below `modulus`.
    fn correct(sum: &mut [u64], carry: u64, modulus: &[u64]);

    /// [`Self::correct`] by one branch on whether the whole correction applies.
    #[cfg(target_arch = "wasm32")]
    fn correct_by_branch(sum: &mut [u64], carry: u64, modulus: &[u64]);

    /// [`Self::correct`] under masks at every length: writes the corrected `sum` to `out`.
    fn correct_masked(out: &mut [u64], sum: &[u64], carry: u64, modulus: &[u64]);
}

/// The limbs of `b`: `a + b` is below twice the modulus, which a carry out or a sum at or above
/// the modulus subtracts once.
impl Addend for ZeroExtended<'_> {
    #[inline(always)]
    fn sum<N: Bits>(a: &Uint<N>, addend: Self) -> (GenericArray<u64, N::Limbs>, u64) {
        let mut sum = Op2CarryIter::<_, _, AddOp>::new(a.source().pull(), addend.pull());
        (pull_limbs::<N::Limbs>(&mut sum), sum.carry())
    }

    #[inline(always)]
    fn correct(sum: &mut [u64], carry: u64, modulus: &[u64]) {
        reduced(subtract_when_above_in_place(sum, modulus, carry));
    }

    #[cfg(target_arch = "wasm32")]
    #[inline(always)]
    fn correct_by_branch(sum: &mut [u64], carry: u64, modulus: &[u64]) {
        if carry != 0 || compare(sum, modulus).is_ge() {
            fits(sub_assign(sum, modulus));
        }
    }

    #[inline(always)]
    fn correct_masked(out: &mut [u64], sum: &[u64], carry: u64, modulus: &[u64]) {
        subtract_when_above_masked(out, sum, modulus, carry);
    }
}

/// `2^(64 * k) - b`: the sum is `a - b + 2^(64 * k)`, which carries out exactly when `a >= b`. It
/// runs as the subtraction `a - b`, whose borrow is the opposite of that carry. Without the carry,
/// `a - b` borrowed, and the modulus is added back; the carry out of that addition cancels the
/// borrow.
impl<S: LimbSource> Addend for TwosComplement<S> {
    #[inline(always)]
    fn sum<N: Bits>(a: &Uint<N>, addend: Self) -> (GenericArray<u64, N::Limbs>, u64) {
        let mut difference = Op2CarryIter::<_, _, SubOp>::new(a.source().pull(), addend.0.pull());
        (
            pull_limbs::<N::Limbs>(&mut difference),
            difference.carry() ^ 1,
        )
    }

    #[inline(always)]
    fn correct(sum: &mut [u64], carry: u64, modulus: &[u64]) {
        add_unless_carry(sum, modulus, carry);
    }

    #[cfg(target_arch = "wasm32")]
    #[inline(always)]
    fn correct_by_branch(sum: &mut [u64], carry: u64, modulus: &[u64]) {
        if carry == 0 {
            fits(add_assign(sum, modulus));
        }
    }

    #[inline(always)]
    fn correct_masked(out: &mut [u64], sum: &[u64], carry: u64, modulus: &[u64]) {
        add_unless_carry_masked(out, sum, modulus, carry);
    }
}

/// `(a + addend) mod modulus` for `a` and the addend's value below `modulus`. The sum takes its
/// correction in place.
#[inline(always)]
pub(crate) fn sum_mod<N: Bits, B: Addend>(a: &Uint<N>, addend: B, modulus: &Uint<N>) -> Uint<N> {
    let (limbs, carry) = B::sum(a, addend);
    let mut out = Uint::<N> { limbs };
    #[cfg(target_arch = "wasm32")]
    correct::<N, B>(&mut out.limbs, carry, &modulus.limbs);
    #[cfg(not(target_arch = "wasm32"))]
    B::correct(&mut out.limbs, carry, &modulus.limbs);
    // Below the modulus, the bits above `N` are zero.
    out
}

/// The correction of a sum of `N` bits: by one branch where [`branches`] says so.
#[cfg(target_arch = "wasm32")]
#[inline(always)]
fn correct<N: Bits, B: Addend>(sum: &mut [u64], carry: u64, modulus: &[u64]) {
    if const { branches(N::LIMBS) } {
        B::correct_by_branch(sum, carry, modulus);
    } else {
        B::correct(sum, carry, modulus);
    }
}

/// [`sum_mod`] with the correction under masks: no branch and no memory address depends on `a` or
/// the addend. The sum runs into registers or a copy, and the correction writes the result.
#[inline(always)]
pub(crate) fn sum_mod_masked<N: Bits, B: Addend>(
    a: &Uint<N>,
    addend: B,
    modulus: &Uint<N>,
) -> Uint<N> {
    let (limbs, carry) = B::sum(a, addend);
    let mut out = Uint::<N>::ZERO;
    B::correct_masked(&mut out.limbs, &limbs, carry, &modulus.limbs);
    out
}

/// `value mod modulus`, for an operand whose top limb reaches the modulus's.
///
/// Natively, [`rare_call`] lays this path out of the loop, so that the operands below the modulus
/// run without a taken branch. On WebAssembly it is a call, as [`reduced_sum`] is.
#[cfg_attr(target_arch = "wasm32", inline(never))]
#[cfg_attr(not(target_arch = "wasm32"), inline(always))]
fn remainder<N: Bits>(value: Uint<N>, modulus: NonZeroUint<N>) -> Uint<N> {
    if const { apart::<N>() } {
        return remainder_apart(&value, &modulus);
    }
    if cfg!(not(target_arch = "wasm32")) {
        rare_call();
    }
    divide_preparing::<N::Window, N>(&value.limbs, modulus, Top::Any, &mut DiscardQuotient)
}

/// Whether the [`remainder`] of `N` bits is a call of its own, natively: at 6 limbs and from 32.
/// Inlined, the division shares the frame of the sum, and the sum's path spills to the slots that
/// the division leaves it. At 384 and 2048 bits the call took the sums 2 to 12 fewer cycles; at 512
/// and 1024 bits the sums took 1 to 3 more, though they ran fewer instructions.
/// Tested in a `const` block, it leaves the other arm out of the code of each length.
const fn apart<N: Bits>() -> bool {
    cfg!(not(target_arch = "wasm32")) && (N::LIMBS == 6 || N::LIMBS >= 32)
}

/// [`remainder`] in a frame of its own.
#[cold]
#[inline(never)]
fn remainder_apart<N: Bits>(value: &Uint<N>, modulus: &NonZeroUint<N>) -> Uint<N> {
    divide::<N::Window, N>(&value.limbs, *modulus, Top::Any, &mut DiscardQuotient)
}
