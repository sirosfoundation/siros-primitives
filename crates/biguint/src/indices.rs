//! Loops over a length that a type fixes.

use typenum::{Bit, UInt, UTerm, Unsigned};

/// A `typenum` length whose [`each`](Indices::each) runs a body on every index below it: an
/// internal helper that `prime-field`, `clmul` and `binary-field` share, outside the public API.
///
/// Builds without debug assertions run one copy of the body per index, by recursion over the
/// binary digits of the length. Every index is then a constant: limbs stay in registers, and a
/// carry chain stays in the flags from one copy to the next, at every length. Builds with debug
/// assertions run a loop, which keeps their code and their frames small.
///
/// Every `typenum` unsigned integer implements it.
#[doc(hidden)]
pub trait Indices: Unsigned + sealed::Copies {
    /// Runs `body` on each index of `0..Self::USIZE`, in order. Mark a closure that is long or
    /// that captures much `#[inline(always)]`: its copies inline only then.
    #[inline(always)]
    fn each(mut body: impl FnMut(usize)) {
        if cfg!(debug_assertions) {
            for index in 0..Self::USIZE {
                body(index);
            }
        } else {
            Self::copies(0, &mut body);
        }
    }
}

impl<N: Unsigned + sealed::Copies> Indices for N {}

pub(crate) mod sealed {
    /// The copies of [`super::Indices::each`]: `Self::USIZE` calls of the body.
    pub trait Copies {
        /// Calls `body` on each index of `start..start + Self::USIZE`, in order.
        fn copies<F: FnMut(usize)>(start: usize, body: &mut F);
    }
}

impl sealed::Copies for UTerm {
    #[inline(always)]
    fn copies<F: FnMut(usize)>(_start: usize, _body: &mut F) {}
}

/// `2 U + B`: the indices of `U` twice, then the last index when `B` is one.
impl<U: Unsigned + sealed::Copies, B: Bit> sealed::Copies for UInt<U, B> {
    #[inline(always)]
    fn copies<F: FnMut(usize)>(start: usize, body: &mut F) {
        U::copies(start, body);
        U::copies(start + U::USIZE, body);
        if B::BOOL {
            body(start + 2 * U::USIZE);
        }
    }
}

#[cfg(test)]
mod tests;
