//! A modulus prepared once, and the residues of the scopes it opens.
//!
//! [`Modulus::scope`] hands its closure a [`Ring`], whose lifetime `'id` is fresh for each call:
//! the generative-lifetime brand of GhostCell's `GhostToken::new`. A residue carries the brand, so
//! the compiler refuses an operation on residues of two scopes and the operations check nothing.
//!
//! An odd modulus keeps residues in Montgomery form, `a * R mod m` for `R = 2^(64 * k)`, and a
//! product is a Montgomery multiplication. An even modulus keeps residues as their values. A
//! product divides by the modulus, whose normalization and reciprocal the context computes once;
//! from 4 to 8 limbs, a modulus that fills its limbs reduces it by Barrett's reduction instead.

mod barrett;
mod ct;
mod montgomery;

use core::fmt;
use core::marker::PhantomData;
use core::ops::{Add, Mul, Neg, Sub};

use generic_array::GenericArray;

use crate::bits::Bits;
use crate::div::ct::MaskedDivisor;
use crate::div::{DiscardQuotient, Invariant, Top};
use crate::kernel::split_mut;
use crate::modular::{sum_mod, sum_mod_masked};
use crate::mul::{basecase, karatsuba};
use crate::nonzero::{NonZeroUint, significant_len};
use crate::recursion::{Depth, Halvings, Width};
use crate::stream::TwosComplement;
use crate::uint::Uint;
use barrett::Barrett;
pub use ct::ConstantTimeResidue;

/// A nonzero modulus with its reduction prepared: [`Modulus::scope`] computes with its residues.
pub struct Modulus<N: Bits> {
    /// `m`.
    value: Uint<N>,
    /// `m` prepared for division.
    divisor: Invariant<N::Limbs>,
    /// `m` prepared for divisions in constant time.
    masked: MaskedDivisor<N::Limbs>,
    /// The form of the residues.
    form: Form<N>,
    /// The residue `1` in that form.
    one: Uint<N>,
}

/// The form of the residues of a modulus.
enum Form<N: Bits> {
    /// An odd modulus: residues hold `a * R mod m`.
    Montgomery {
        /// `-m^-1 mod 2^64`.
        inverse: u64,
        /// `R^2 mod m`: the Montgomery product with it brings a value into the form.
        r_squared: Uint<N>,
    },
    /// An even modulus: residues hold their values, and products divide by the modulus.
    Plain,
    /// An even modulus that fills its limbs, from 4 to 8 limbs: residues hold their values, and
    /// products reduce by Barrett's reduction.
    Barrett(Barrett<N>),
}

/// The residue refused by [`Ring::residue`]: it is not below the modulus.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("the value is not below the modulus")]
pub struct NotReduced;

impl<N: Bits> Modulus<N> {
    /// `value`, or `None` when it is zero.
    #[inline(always)]
    #[must_use]
    pub fn new(value: Uint<N>) -> Option<Self> {
        NonZeroUint::new(value).map(Self::from)
    }

    /// Runs `f` on a ring of residues modulo `self`, branded with a lifetime of its own.
    ///
    /// Residues of different scopes do not mix:
    ///
    /// ```compile_fail,E0521
    /// use biguint::{Modulus, U256};
    ///
    /// let modulus = Modulus::new(U256::from_u64(97)).expect("a nonzero modulus");
    /// modulus.scope(|first| {
    ///     let a = first.reduce(U256::ONE);
    ///     modulus.scope(|second| {
    ///         let b = second.reduce(U256::ONE);
    ///         (a + b).get()
    ///     })
    /// });
    /// ```
    #[inline(always)]
    pub fn scope<R>(&self, f: impl for<'id> FnOnce(Ring<'id, N>) -> R) -> R {
        f(Ring {
            modulus: self,
            brand: PhantomData,
        })
    }
}

impl<N: Bits> From<NonZeroUint<N>> for Modulus<N> {
    /// Prepares the reduction: the division by `m`, and for an odd `m` the Montgomery constants.
    #[inline(always)]
    fn from(modulus: NonZeroUint<N>) -> Self {
        let value = modulus.get();
        let divisor = Invariant::new(&value.limbs, significant_len(&value.limbs));
        let masked = MaskedDivisor::new(&value.limbs);
        // `2^(64 * limb) mod m`.
        let power = |limb: usize| {
            let mut dividend = GenericArray::<u64, N::ProductWindow>::default();
            if let Some(slot) = dividend.get_mut(limb) {
                *slot = 1;
            }
            let (dividend, _) = split_mut(&mut dividend, limb + 1);
            divisor.remainder::<N::ProductScratch, N>(dividend, Top::Any, &mut DiscardQuotient)
        };
        let low = value.limbs.first().copied().unwrap_or(0);
        if low & 1 == 1 {
            Self {
                form: Form::Montgomery {
                    inverse: montgomery::negated_inverse(low),
                    r_squared: power(2 * N::LIMBS),
                },
                one: power(N::LIMBS),
                value,
                divisor,
                masked,
            }
        } else {
            let form = if barrett::applies(N::LIMBS) && significant_len(&value.limbs) == N::LIMBS {
                Form::Barrett(Barrett::new(&value, &divisor))
            } else {
                Form::Plain
            };
            Self {
                form,
                one: power(0),
                value,
                divisor,
                masked,
            }
        }
    }
}

impl<N: Bits> fmt::Debug for Modulus<N> {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Modulus({:?})", self.value)
    }
}

/// The residues modulo one [`Modulus`] within one [`Modulus::scope`]. The lifetime `'id` is the
/// scope's brand.
pub struct Ring<'id, N: Bits> {
    modulus: &'id Modulus<N>,
    /// Invariant in `'id`, so no other scope's brand converts to it.
    brand: PhantomData<fn(&'id ()) -> &'id ()>,
}

impl<N: Bits> Clone for Ring<'_, N> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<N: Bits> Copy for Ring<'_, N> {}

impl<N: Bits> fmt::Debug for Ring<'_, N> {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Ring({:?})", self.modulus.value)
    }
}

impl<'id, N: Bits> Ring<'id, N> {
    /// `value mod m`, for a value of any width.
    #[inline(always)]
    #[must_use]
    pub fn reduce<M: Bits>(self, value: Uint<M>) -> Residue<'id, N> {
        let remainder = self.modulus.divisor.remainder::<M::Window, N>(
            &value.limbs,
            Top::Any,
            &mut DiscardQuotient,
        );
        self.enter(remainder)
    }

    /// `value` as a residue.
    ///
    /// # Errors
    ///
    /// [`NotReduced`] when `value` is not below the modulus.
    #[inline(always)]
    pub fn residue(self, value: Uint<N>) -> Result<Residue<'id, N>, NotReduced> {
        if value < self.modulus.value {
            Ok(self.enter(value))
        } else {
            Err(NotReduced)
        }
    }

    /// The residue of `value`, below the modulus.
    #[inline(always)]
    fn enter(self, value: Uint<N>) -> Residue<'id, N> {
        let value = match &self.modulus.form {
            Form::Montgomery { r_squared, .. } => self.product(&value, r_squared),
            Form::Plain | Form::Barrett(_) => value,
        };
        Residue { value, ring: self }
    }

    /// The value of the residue `value`. A call at the widths where its copy is large.
    #[inline]
    fn leave(self, value: Uint<N>) -> Uint<N> {
        match &self.modulus.form {
            Form::Montgomery { inverse, .. } => {
                let mut t = GenericArray::<u64, N::ProductWindow>::default();
                for (slot, &limb) in t.iter_mut().zip(&value.limbs) {
                    *slot = limb;
                }
                Uint {
                    limbs: montgomery::reduce(&mut t, &self.modulus.value.limbs, *inverse),
                }
            }
            Form::Plain | Form::Barrett(_) => value,
        }
    }

    /// The product of the residues `a` and `b`. A call at the widths where its copy is
    /// large.
    #[inline]
    fn product(self, a: &Uint<N>, b: &Uint<N>) -> Uint<N> {
        let mut t = GenericArray::<u64, N::ProductWindow>::default();
        if N::LIMBS < karatsuba::THRESHOLD {
            basecase::product(&mut t, &a.limbs, &b.limbs);
        } else {
            let (product, _) = split_mut(&mut t, 2 * N::LIMBS);
            let mut scratch = GenericArray::<u64, N::ProductScratch>::default();
            Halvings::entry_product::<Width<N::Limbs>, false>(
                product,
                &a.limbs,
                &b.limbs,
                &mut scratch,
            );
        }
        self.reduce_product(&mut t)
    }

    /// The square of the residue `a`. A call at the widths where its copy is large.
    #[inline]
    fn square(self, a: &Uint<N>) -> Uint<N> {
        let mut t = GenericArray::<u64, N::ProductWindow>::default();
        {
            let (square, _) = split_mut(&mut t, 2 * N::LIMBS);
            if N::LIMBS < karatsuba::THRESHOLD {
                basecase::square(square, &a.limbs);
            } else {
                let mut scratch = GenericArray::<u64, N::ProductScratch>::default();
                Halvings::entry_product::<Width<N::Limbs>, false>(
                    square,
                    &a.limbs,
                    &a.limbs,
                    &mut scratch,
                );
            }
        }
        self.reduce_product(&mut t)
    }

    /// The residue of the product of two residues in the low `2 * N::LIMBS` limbs of `t`.
    #[inline(always)]
    fn reduce_product(self, t: &mut GenericArray<u64, N::ProductWindow>) -> Uint<N> {
        match &self.modulus.form {
            Form::Montgomery { inverse, .. } => Uint {
                limbs: montgomery::reduce(t, &self.modulus.value.limbs, *inverse),
            },
            Form::Plain => {
                let (product, _) = split_mut(t, 2 * N::LIMBS);
                self.modulus.divisor.remainder::<N::ProductWindow, N>(
                    product,
                    Top::Below,
                    &mut DiscardQuotient,
                )
            }
            Form::Barrett(barrett) => {
                let (product, _) = split_mut(t, 2 * N::LIMBS);
                barrett.reduce(product)
            }
        }
    }
}

/// A residue modulo the [`Modulus`] of the scope `'id`, held in the form of its modulus.
pub struct Residue<'id, N: Bits> {
    value: Uint<N>,
    ring: Ring<'id, N>,
}

impl<N: Bits> Clone for Residue<'_, N>
where
    Uint<N>: Copy,
{
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<N: Bits> Copy for Residue<'_, N> where Uint<N>: Copy {}

impl<N: Bits> fmt::Debug for Residue<'_, N>
where
    Uint<N>: Copy,
{
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Residue({:?})", self.get())
    }
}

impl<'id, N: Bits> Residue<'id, N>
where
    Uint<N>: Copy,
{
    /// The value, below the modulus.
    #[inline(always)]
    #[must_use]
    pub fn get(self) -> Uint<N> {
        self.ring.leave(self.value)
    }

    /// `self * self`.
    #[inline(always)]
    #[must_use]
    pub fn square(self) -> Self {
        Self {
            value: self.ring.square(&self.value),
            ring: self.ring,
        }
    }

    /// `self^exponent`, by sliding windows of odd powers over the exponent bits from the most
    /// significant.
    #[must_use]
    #[inline]
    pub fn pow<E: Bits>(self, exponent: Uint<E>) -> Self {
        let bits = exponent.bit_len();
        let window = pow_window(bits);
        // `table[i] = self^(2 i + 1)`.
        let mut table = [self; 1 << (POW_WINDOW - 1)];
        let square = self.square();
        let mut previous = self;
        for entry in table.iter_mut().take(1 << (window - 1)).skip(1) {
            previous = previous * square;
            *entry = previous;
        }
        let mut result = Self {
            value: self.ring.modulus.one,
            ring: self.ring,
        };
        let mut index = bits;
        while index > 0 {
            if !exponent.bit_at(index - 1) {
                result = result.square();
                index -= 1;
                continue;
            }
            let width = window.min(index);
            let mut value = 0;
            for offset in 0..width {
                value = (value << 1) | usize::from(exponent.bit_at(index - 1 - offset));
            }
            let trailing = value.trailing_zeros() as usize;
            let used = width - trailing;
            for _ in 0..used {
                result = result.square();
            }
            let power = table.get((value >> trailing) / 2).copied().unwrap_or(self);
            result = result * power;
            index -= used;
        }
        result
    }
}

/// The widest window of [`Residue::pow`]: its table holds `2^(POW_WINDOW - 1)` odd powers.
const POW_WINDOW: usize = 6;

/// The window width that minimizes the table and the multiplications of a `bits`-bit exponent:
/// `2^(w - 1) - 1` table products and about `bits / (w + 1)` window products.
#[inline(always)]
const fn pow_window(bits: usize) -> usize {
    let mut best = 1;
    let mut window = 2;
    while window <= POW_WINDOW {
        let cost = (1 << (window - 1)) - 1 + bits / (window + 1);
        let best_cost = (1 << (best - 1)) - 1 + bits / (best + 1);
        if cost < best_cost {
            best = window;
        }
        window += 1;
    }
    best
}

/// `(self + rhs) mod m`.
impl<'id, N: Bits> Add for Residue<'id, N> {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        let (a, b, m) = (&self.value, rhs.value.source(), &self.ring.modulus.value);
        let value = if const { masked_residue_sum::<N>() } {
            sum_mod_masked(a, b, m)
        } else {
            sum_mod(a, b, m)
        };
        Self {
            value,
            ring: self.ring,
        }
    }
}

/// The lengths whose residue sums take their correction under a mask. There LLVM turns the selects
/// of [`sum_mod`] into a branch, which mispredicts on half the sums of uniform residues.
const MASKED_RESIDUE_SUM_LIMBS: core::ops::RangeInclusive<usize> = 4..=8;

/// Whether a residue sum of `N` bits runs [`sum_mod_masked`]; the WebAssembly builds keep their
/// measured form, the branch of [`sum_mod`]. Tested in a `const` block, it leaves the other arm out
/// of the code of each length.
const fn masked_residue_sum<N: Bits>() -> bool {
    cfg!(not(target_arch = "wasm32"))
        && *MASKED_RESIDUE_SUM_LIMBS.start() <= N::LIMBS
        && N::LIMBS <= *MASKED_RESIDUE_SUM_LIMBS.end()
}

/// `(self - rhs) mod m`.
impl<'id, N: Bits> Sub for Residue<'id, N> {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            value: sum_mod(
                &self.value,
                TwosComplement(rhs.value.source()),
                &self.ring.modulus.value,
            ),
            ring: self.ring,
        }
    }
}

/// `-self mod m`.
impl<'id, N: Bits> Neg for Residue<'id, N> {
    type Output = Self;

    #[inline(always)]
    fn neg(self) -> Self::Output {
        Self {
            value: sum_mod(
                &Uint::ZERO,
                TwosComplement(self.value.source()),
                &self.ring.modulus.value,
            ),
            ring: self.ring,
        }
    }
}

/// `(self * rhs) mod m`.
impl<'id, N: Bits> Mul for Residue<'id, N> {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            value: self.ring.product(&self.value, &rhs.value),
            ring: self.ring,
        }
    }
}
