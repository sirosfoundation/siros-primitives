//! Carry-less products of little-endian `u64` limb slices: the schoolbook over word products,
//! accumulated by XOR. The slice lengths set the loop counts, so lengths that come from types fold
//! the loops into straight-line code.

use crate::to_limbs;

/// Defines the slice product `$name` over the word products of `$leaf`.
macro_rules! schoolbook {
    ($(#[$attribute:meta])* [$($qualifier:tt)*] fn $name:ident uses $leaf:path) => {
        $(#[$attribute])*
        #[inline]
        pub $($qualifier)* fn $name(out: &mut [u64], a: &[u64], b: &[u64]) {
            let mut slots = &mut *out;
            while let [slot, slots_rest @ ..] = slots {
                *slot = 0;
                slots = slots_rest;
            }
            // `row` starts at the limb where the products of the current multiplier land.
            let mut row = out;
            let mut multipliers = a;
            while let [multiplier, multipliers_rest @ ..] = multipliers {
                let mut slots = &mut *row;
                let mut limbs = b;
                // The high half of the previous word product, which lands on the current slot.
                let mut high = 0;
                while let [slot, slots_rest @ ..] = slots {
                    let [low, next_high] = match limbs {
                        [limb, limbs_rest @ ..] => {
                            limbs = limbs_rest;
                            to_limbs($leaf(*multiplier, *limb))
                        }
                        [] => {
                            *slot ^= high;
                            break;
                        }
                    };
                    *slot ^= low ^ high;
                    high = next_high;
                    slots = slots_rest;
                }
                row = match row {
                    [_, row_rest @ ..] => row_rest,
                    [] => break,
                };
                multipliers = multipliers_rest;
            }
        }
    };
}

schoolbook! {
    /// Writes the carry-less product of the little-endian limb slices `a` and `b` to `out`, modulo
    /// `x^(64 * out.len())`. The full product takes `a.len() + b.len()` limbs; the limbs of `out`
    /// above it are zero.
    ///
    /// [`clmul64`](crate::clmul64) computes the word products.
    [] fn clmul_into uses crate::backend::clmul64
}

schoolbook! {
    /// [`clmul_into`], evaluable in const contexts: the word products come from
    /// [`const_clmul64`](crate::const_clmul64).
    [const] fn const_clmul_into uses crate::comb::clmul64
}
