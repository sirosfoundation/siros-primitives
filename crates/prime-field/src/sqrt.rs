//! Square roots: `x^((p + 1) / 4)` when `p = 3 mod 4`, Tonelli–Shanks otherwise.

use crate::field::{Fp, Modulus};
use crate::params::Shape;

impl<M: Modulus> Fp<M> {
    /// A square root of `self`, or `None` when `self` is a quadratic non-residue. Of the two
    /// roots of a residue, which one is returned is unspecified. The time depends on the value.
    #[inline]
    #[must_use]
    pub fn sqrt(self) -> Option<Self> {
        if Shape::<M>::THREE_MOD_FOUR {
            let root = self.pow_limbs(Shape::<M>::SQRT_EXPONENT.as_slice());
            return if root.square().equals(&self) {
                Some(root)
            } else {
                None
            };
        }
        if self.is_zero() {
            return Some(Self::ZERO);
        }
        // Tonelli–Shanks with `p - 1 = 2^s t`: from `w = x^((t - 1) / 2)`, the candidate root
        // `r = w x` and `b = w r = x^t`, which lies in the 2-Sylow subgroup. Each round finds the
        // order `2^i` of `b` and multiplies `r` by the power of the subgroup generator that takes
        // `b` one step closer to `1`.
        let w = self.pow_limbs(Shape::<M>::SQRT_EXPONENT.as_slice());
        let mut root = w.mul(self);
        let mut b = w.mul(root);
        let mut generator = Self::from_limbs(Shape::<M>::TWO_ADIC_ROOT);
        let mut order = Shape::<M>::TWO_ADICITY;
        loop {
            if b.equals(&Self::ONE) {
                return Some(root);
            }
            let mut power = b;
            let mut exponent = 0;
            while !power.equals(&Self::ONE) {
                power = power.square();
                exponent += 1;
                if exponent == order {
                    return None;
                }
            }
            let mut step = generator;
            for _ in 0..order - exponent - 1 {
                step = step.square();
            }
            root = root.mul(step);
            generator = step.square();
            b = b.mul(generator);
            order = exponent;
        }
    }
}
