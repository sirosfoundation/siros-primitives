//! Square roots: `x^((p + 1) / 4)` when `p = 3 mod 4`, Tonelli–Shanks otherwise.

use crate::field::{Fp, Modulus};
use crate::limb::select;
use crate::params::{ROOT_POWERS, Shape};
use constant_time::declassify;

impl<M: Modulus> Fp<M> {
    /// A square root of `self`, or `None` when `self` is a quadratic non-residue. Of the two
    /// roots of a residue, which one is returned is unspecified.
    // As a call, the P-256 scalar root measured 5.5% more cycles.
    #[inline(always)]
    #[must_use]
    pub fn sqrt(self) -> Option<Self> {
        if Shape::<M>::THREE_MOD_FOUR {
            let root = self.pow_program::<false>(&Shape::<M>::SQRT_CHAIN);
            return if root.square_with::<false>().equals(&self) {
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
        let w = self.pow_program::<false>(&Shape::<M>::SQRT_CHAIN);
        let mut root = w.mul_with::<false>(self);
        let mut b = w.mul_with::<false>(root);
        let mut offset = 0;
        while !b.equals(&Self::ONE) {
            let mut power = b;
            let mut index = Shape::<M>::TWO_ADICITY;
            loop {
                power = power.square_with::<false>();
                index -= 1;
                if index == offset {
                    return None;
                }
                if power.equals(&Self::ONE) {
                    break;
                }
            }
            root = root.mul_with::<false>(Self::root_power(index - 1));
            b = b.mul_with::<false>(Self::root_power(index));
            offset = index;
        }
        Some(root)
    }

    /// A square root of `self`, or `None` when `self` is a quadratic non-residue, in time that
    /// depends on the modulus alone; the `Option` is built in [`declassify`].
    ///
    /// For `p = 1 mod 4` this is the constant-time Tonelli–Shanks of the `ff` crate (algorithm
    /// 5 of Adj and Rodríguez-Henríquez, *Square root computation over even extension fields*,
    /// 2012, without the non-residue test): every round of the `s` rounds runs every squaring
    /// that a round may need and keeps the right values by conditional moves.
    #[inline(always)]
    pub(crate) fn sqrt_ct(self) -> Option<Self> {
        let root = if Shape::<M>::THREE_MOD_FOUR {
            self.pow_program::<true>(&Shape::<M>::SQRT_CHAIN)
        } else {
            let w = self.pow_program::<true>(&Shape::<M>::SQRT_CHAIN);
            let mut root = w.mul_with::<true>(self);
            let mut b = root.mul_with::<true>(w);
            let mut z = Self::from_limbs(Shape::<M>::TWO_ADIC_ROOT);
            let mut v = Shape::<M>::TWO_ADICITY;
            let mut max_v = Shape::<M>::TWO_ADICITY;
            while max_v >= 1 {
                // `k` ends as the order exponent of `b`; up to it the squarings compute
                // `b^(2^k)`, beyond it they square `z`, and past `v` they change nothing.
                let mut k = 1;
                let mut power = b.square_with::<true>();
                let mut below_v = true;
                let mut j = 2;
                while j < max_v {
                    let one = power.equals(&Self::ONE);
                    let squared = Self::select(one, z, power).square_with::<true>();
                    power = Self::select(one, power, squared);
                    let next_z = Self::select(one, squared, z);
                    below_v &= j != v;
                    k = select(one, k as u64, j as u64) as usize;
                    z = Self::select(below_v, next_z, z);
                    j += 1;
                }
                let moved = root.mul_with::<true>(z);
                root = Self::select(b.equals(&Self::ONE), root, moved);
                z = z.square_with::<true>();
                b = b.mul_with::<true>(z);
                v = k;
                max_v -= 1;
            }
            root
        };
        declassify(root.square_with::<true>().equals(&self), root)
    }

    /// `a` when `condition` holds, else `b`, by conditional moves.
    #[inline(always)]
    fn select(condition: bool, a: Self, b: Self) -> Self {
        Self::from_limbs(Shape::<M>::select_limbs(
            condition,
            a.limb_array(),
            b.limb_array(),
        ))
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
            power = power.square_with::<false>();
        }
        power
    }
}
