//! [`unroll!`], the limb loops of the product kernels written out in the source.
//!
//! It copies a loop body once per index below 9, the limbs of the widest standard field, each copy
//! behind a test of its index against the length; a loop carries the indices from 9 on. After
//! monomorphization every length is a constant, so the copies past the length fold away. The
//! products, the Montgomery reductions and the folds use it: written with the `each` runs of
//! `arith.rs`, their operands reached LLVM in another order, and the products measured up to 2
//! cycles more on BN254, Tom-256 and P-521. Builds with debug assertions run the loop alone.

/// The indices that [`unroll!`] copies; a loop runs the rest.
pub(crate) const UNROLLED: usize = 9;

/// Runs `$body` with `$i` bound to each index of `$start..$len`, in order.
macro_rules! unroll {
    ($i:ident in $start:expr, $len:expr => $body:block) => {
        $crate::unrolled::unroll!(@copies $i, $start, $len, $body; 0 1 2 3 4 5 6 7 8)
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
