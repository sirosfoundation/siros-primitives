//! Square roots: `x^((p + 1) / 4)` when `p = 3 mod 4`, Tonelli–Shanks otherwise.

use crate::field::{Fp, Modulus};
use crate::params::{ROOT_POWERS, Shape};

impl<M: Modulus> Fp<M> {
    /// A square root of `self`, or `None` when `self` is a quadratic non-residue. Of the two
    /// roots of a residue, which one is returned is unspecified. The time depends on the value.
    #[inline]
    #[must_use]
    pub fn sqrt(self) -> Option<Self> {
        if Shape::<M>::THREE_MOD_FOUR {
            let root = self.pow_program(&Shape::<M>::SQRT_CHAIN);
            return if root.square().equals(&self) {
                Some(root)
            } else {
                None
            };
        }
        if self.is_zero() {
            return Some(Self::ZERO);
        }
        // Tonelli–Shanks with `p - 1 = 2^s t` and the two-adic root `c = z^t`: from
        // `w = x^((t - 1) / 2)`, the candidate root `r = w x` and `b = w r = x^t`, which lies in
        // the 2-Sylow subgroup. Each round finds the order `2^i` of `b` and multiplies `r` by
        // the power of `c` that takes `b` one step closer to `1`. The subgroup generator of a
        // round is `c^(2^offset)` with `offset + order = s`, so every power it needs is a table
        // entry: `index` counts down from `s` with the squarings that find `i`, and lands on the
        // entry of the next generator.
        let w = self.pow_program(&Shape::<M>::SQRT_CHAIN);
        let mut root = w.mul(self);
        let mut b = w.mul(root);
        let mut offset = 0;
        while !b.equals(&Self::ONE) {
            let mut power = b;
            let mut index = Shape::<M>::TWO_ADICITY;
            loop {
                power = power.square();
                index -= 1;
                if index == offset {
                    return None;
                }
                if power.equals(&Self::ONE) {
                    break;
                }
            }
            root = root.mul(Self::root_power(index - 1));
            b = b.mul(Self::root_power(index));
            offset = index;
        }
        Some(root)
    }

    /// `c^(2^index)` for the two-adic root `c`, with `index < TWO_ADICITY`: a table entry, or
    /// the last entry squared up to the index.
    #[inline(always)]
    #[expect(
        clippy::indexing_slicing,
        reason = "the index is checked against the table"
    )]
    fn root_power(index: usize) -> Self {
        if index < ROOT_POWERS {
            return Self::from_limbs(Shape::<M>::TWO_ADIC_ROOT_POWERS[index]);
        }
        let mut power = Self::from_limbs(Shape::<M>::TWO_ADIC_ROOT_POWERS[ROOT_POWERS - 1]);
        for _ in ROOT_POWERS - 1..index {
            power = power.square();
        }
        power
    }
}
