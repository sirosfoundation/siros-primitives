//! Loops over a length that a type fixes.

use core::marker::PhantomData;

use typenum::{Bit, UInt, UTerm, Unsigned};

/// A `typenum` length whose [`each`](Indices::each) runs a body on every index below it: an
/// internal helper that `prime-field`, `clmul` and `binary-field` share, outside the public API.
///
/// Builds without debug assertions run one copy of the body per index, by recursion over the
/// binary digits of the length. Each copy passes its index as a constant of its type, so the body
/// sees a constant even while LLVM simplifies it inside the recursion: limbs stay in registers,
/// and a carry chain stays in the flags from one copy to the next, at every length. Builds with
/// debug assertions run a loop, which keeps their code and their frames small.
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
            Self::copies::<sealed::Origin, _>(&mut body);
        }
    }
}

impl<N: Unsigned + sealed::Copies> Indices for N {}

pub(crate) mod sealed {
    use super::{PhantomData, Unsigned};

    /// A first index, as a type.
    pub trait At {
        /// The index.
        const AT: usize;
    }

    /// Index 0.
    pub struct Origin;

    impl At for Origin {
        const AT: usize = 0;
    }

    /// The index `N` past `O`.
    pub struct Past<O, N>(PhantomData<(O, N)>);

    impl<O: At, N: Unsigned> At for Past<O, N> {
        const AT: usize = O::AT + N::USIZE;
    }

    /// The copies of [`super::Indices::each`]: `Self::USIZE` calls of the body.
    pub trait Copies {
        /// Calls `body` on each index of `O::AT..O::AT + Self::USIZE`, in order, each a constant.
        fn copies<O: At, F: FnMut(usize)>(body: &mut F);
    }
}

impl sealed::Copies for UTerm {
    #[inline(always)]
    fn copies<O: sealed::At, F: FnMut(usize)>(_body: &mut F) {}
}

/// `2 U + B`: the indices of `U` twice, then the last index when `B` is one.
impl<U: Unsigned + sealed::Copies, B: Bit> sealed::Copies for UInt<U, B> {
    #[inline(always)]
    fn copies<O: sealed::At, F: FnMut(usize)>(body: &mut F) {
        U::copies::<O, F>(body);
        U::copies::<sealed::Past<O, U>, F>(body);
        if B::BOOL {
            body(<sealed::Past<sealed::Past<O, U>, U> as sealed::At>::AT);
        }
    }
}

#[cfg(test)]
mod tests;
