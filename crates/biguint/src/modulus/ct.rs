//! The operations of [`Residue`] through which [`ConstantTimeResidue`] computes, each in time that
//! depends on no residue value.
//!
//! An odd modulus runs Montgomery multiplication with its final subtraction under a mask. An even
//! modulus reduces each product by the division of [`MaskedDivisor`], whose quotient limbs and
//! corrections run under masks. Sums correct under masks, and `pow` reads its table by a masked
//! scan of every entry.

use core::ops::{Add, Mul, Sub};

use constant_time::{ConstantTime, CtAdd, CtGet, CtMul, CtNeg, CtPow, CtSquare, CtSub, Public};
use generic_array::GenericArray;

use super::{Form, Residue, Ring, montgomery};
use crate::bits::Bits;
use crate::ct::bit;
use crate::div::DiscardQuotient;
use crate::kernel::{mask_equal, select_masked, split, split_mut};
use crate::modular::sum_mod_masked;
use crate::mul::{basecase, karatsuba};
use crate::recursion::{Depth, Halvings};
use crate::stream::TwosComplement;
use crate::uint::Uint;

/// A residue modulo the [`Modulus`](super::Modulus) of the scope `'id`, held in the form of its
/// modulus. Its operations run in constant time; the modulus is public.
///
/// [`Ring::reduce_constant_time`] makes one; [`ConstantTime::get`] gives its value below the
/// modulus.
pub type ConstantTimeResidue<'id, N> = ConstantTime<Residue<'id, N>>;

impl<'id, N: Bits> Ring<'id, N>
where
    Uint<N>: Copy,
{
    /// `value mod m`, as a residue whose operations run in constant time. `value` may have any
    /// width.
    #[inline(always)]
    #[must_use]
    pub fn reduce_constant_time<M: Bits>(
        self,
        value: ConstantTime<Uint<M>>,
    ) -> ConstantTimeResidue<'id, N> {
        let remainder = self
            .modulus
            .masked
            .remainder::<M::Window, N>(&value.reveal().limbs, &mut DiscardQuotient);
        let value = match &self.modulus.form {
            Form::Montgomery { r_squared, .. } => self.product_masked(&remainder, r_squared),
            Form::Plain | Form::Barrett(_) => remainder,
        };
        ConstantTime::new(Residue { value, ring: self })
    }

    /// The value of the residue `value`, in time that depends on no value.
    #[inline(always)]
    fn leave_masked(self, value: Uint<N>) -> Uint<N> {
        match &self.modulus.form {
            Form::Montgomery { inverse, .. } => {
                let mut t = GenericArray::<u64, N::ProductWindow>::default();
                for (slot, &limb) in t.iter_mut().zip(&value.limbs) {
                    *slot = limb;
                }
                let mut out = Uint::<N>::ZERO;
                montgomery::reduce_masked(
                    &mut out.limbs,
                    &mut t,
                    &self.modulus.value.limbs,
                    *inverse,
                );
                out
            }
            Form::Plain | Form::Barrett(_) => value,
        }
    }

    /// The product of the residues `a` and `b`, in time that depends on no value. A call at the
    /// widths where its copy is large.
    #[inline]
    fn product_masked(self, a: &Uint<N>, b: &Uint<N>) -> Uint<N> {
        let mut t = GenericArray::<u64, N::ProductWindow>::default();
        if N::LIMBS < karatsuba::THRESHOLD {
            basecase::product(&mut t, &a.limbs, &b.limbs);
        } else {
            let (product, _) = split_mut(&mut t, 2 * N::LIMBS);
            let mut scratch = GenericArray::<u64, N::ProductScratch>::default();
            Halvings::product_with::<true>(product, &a.limbs, &b.limbs, &mut scratch);
        }
        self.reduce_product_masked(&mut t)
    }

    /// The square of the residue `a`, in time that depends on no value. A call at
    /// the widths where its copy is large.
    #[inline]
    fn square_masked(self, a: &Uint<N>) -> Uint<N> {
        let mut t = GenericArray::<u64, N::ProductWindow>::default();
        {
            let (square, _) = split_mut(&mut t, 2 * N::LIMBS);
            if N::LIMBS < karatsuba::THRESHOLD {
                basecase::square(square, &a.limbs);
            } else {
                let mut scratch = GenericArray::<u64, N::ProductScratch>::default();
                Halvings::product_with::<true>(square, &a.limbs, &a.limbs, &mut scratch);
            }
        }
        self.reduce_product_masked(&mut t)
    }

    /// The residue of the product of two residues in the low `2 * N::LIMBS` limbs of `t`.
    #[inline(always)]
    fn reduce_product_masked(self, t: &mut GenericArray<u64, N::ProductWindow>) -> Uint<N> {
        match &self.modulus.form {
            Form::Montgomery { inverse, .. } => {
                let mut out = Uint::<N>::ZERO;
                montgomery::reduce_masked(&mut out.limbs, t, &self.modulus.value.limbs, *inverse);
                out
            }
            Form::Plain | Form::Barrett(_) => {
                let (product, _) = split(t, 2 * N::LIMBS);
                self.modulus
                    .masked
                    .remainder::<N::ProductWindow, N>(product, &mut DiscardQuotient)
            }
        }
    }
}

impl<N: Bits> Public for Residue<'_, N> where Uint<N>: Copy {}

/// `(self + rhs) mod m`.
impl<'id, N: Bits> CtAdd for Residue<'id, N>
where
    Uint<N>: Copy,
{
    type Output = Self;

    #[inline(always)]
    fn ct_add(self, rhs: Self) -> Self {
        Self {
            value: sum_mod_masked(&self.value, rhs.value.source(), &self.ring.modulus.value),
            ring: self.ring,
        }
    }
}

/// `(self - rhs) mod m`.
impl<'id, N: Bits> CtSub for Residue<'id, N>
where
    Uint<N>: Copy,
{
    type Output = Self;

    #[inline(always)]
    fn ct_sub(self, rhs: Self) -> Self {
        Self {
            value: sum_mod_masked(
                &self.value,
                TwosComplement(rhs.value.source()),
                &self.ring.modulus.value,
            ),
            ring: self.ring,
        }
    }
}

/// `-self mod m`.
impl<'id, N: Bits> CtNeg for Residue<'id, N>
where
    Uint<N>: Copy,
{
    #[inline(always)]
    fn ct_neg(self) -> Self {
        Self {
            value: sum_mod_masked(
                &Uint::ZERO,
                TwosComplement(self.value.source()),
                &self.ring.modulus.value,
            ),
            ring: self.ring,
        }
    }
}

/// `(self * rhs) mod m`.
impl<'id, N: Bits> CtMul for Residue<'id, N>
where
    Uint<N>: Copy,
{
    type Output = Self;

    #[inline(always)]
    fn ct_mul(self, rhs: Self) -> Self {
        Self {
            value: self.ring.product_masked(&self.value, &rhs.value),
            ring: self.ring,
        }
    }
}

/// `self * self`.
impl<'id, N: Bits> CtSquare for Residue<'id, N>
where
    Uint<N>: Copy,
{
    #[inline(always)]
    fn ct_square(self) -> Self {
        Self {
            value: self.ring.square_masked(&self.value),
            ring: self.ring,
        }
    }
}

/// The widest window of `pow` by a wrapped exponent: its table holds `2^MAX_WINDOW` powers.
const MAX_WINDOW: usize = 5;

/// The window width that minimizes the products of a `bits`-bit exponent: `2^w - 1` table
/// products and `ceil(bits / w)` window products. Every window takes `w` squarings whatever `w`.
#[inline(always)]
const fn fixed_window(bits: usize) -> usize {
    let mut best = 1;
    let mut window = 2;
    while window <= MAX_WINDOW {
        if (1 << window) + bits.div_ceil(window) < (1 << best) + bits.div_ceil(best) {
            best = window;
        }
        window += 1;
    }
    best
}

/// `self^exponent` for a wrapped exponent, by a fixed window over all `E` bits: every window
/// squares `w` times and multiplies by the table entry of its digit, which a masked scan of the
/// whole table reads.
impl<'id, N: Bits, E: Bits> CtPow<ConstantTime<Uint<E>>> for Residue<'id, N>
where
    Uint<N>: Copy,
{
    #[inline]
    fn ct_pow(self, exponent: ConstantTime<Uint<E>>) -> Self {
        let exponent = exponent.reveal();
        let ring = self.ring;
        let window = fixed_window(E::BITS);
        let entries = 1 << window;
        let one = ring.modulus.one;
        // `table[i] = self^i`.
        let mut table = [one; 1 << MAX_WINDOW];
        let mut previous = one;
        for entry in table.iter_mut().take(entries).skip(1) {
            previous = ring.product_masked(&previous, &self.value);
            *entry = previous;
        }
        let mut power = one;
        for start in (0..E::BITS.div_ceil(window))
            .rev()
            .map(|index| index * window)
        {
            for _ in 0..window {
                power = ring.square_masked(&power);
            }
            let digit = (0..window).fold(0, |digit, offset| {
                digit | bit(&exponent, start + offset) << offset
            });
            let mut selected = Uint::<N>::ZERO;
            for (index, entry) in (0u64..).zip(table.iter().take(entries)) {
                select_masked(&mut selected.limbs, &entry.limbs, mask_equal(index, digit));
            }
            power = ring.product_masked(&power, &selected);
        }
        Self { value: power, ring }
    }
}

/// The value below the modulus.
impl<N: Bits> CtGet for Residue<'_, N>
where
    Uint<N>: Copy,
{
    type Value = Uint<N>;

    #[inline(always)]
    fn ct_get(self) -> Uint<N> {
        self.ring.leave_masked(self.value)
    }
}

/// Implements an operator with a public `Residue` on the left and a [`ConstantTimeResidue`] on the
/// right.
macro_rules! public_left {
    ($($op:ident, $method:ident, $ct:ident, $ct_method:ident;)*) => {
        $(
            impl<'id, N: Bits> $op<ConstantTimeResidue<'id, N>> for Residue<'id, N>
            where
                Uint<N>: Copy,
            {
                type Output = ConstantTimeResidue<'id, N>;

                #[inline(always)]
                fn $method(self, rhs: ConstantTimeResidue<'id, N>) -> ConstantTimeResidue<'id, N> {
                    ConstantTime::new($ct::$ct_method(self, rhs.reveal()))
                }
            }
        )*
    };
}

public_left!(
    Add, add, CtAdd, ct_add;
    Sub, sub, CtSub, ct_sub;
    Mul, mul, CtMul, ct_mul;
);
