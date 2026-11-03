//! Loops over a length that a type fixes, for the kernels of this crate.

use typenum::Unsigned;

/// The indices that [`each`] calls its body for one by one, each a literal; a loop runs the rest.
const COPIES: usize = 16;

/// Runs `body` on each index of `0..N::USIZE`, in order. Mark a closure that is long or that
/// captures much `#[inline(always)]`: its copies inline only then.
///
/// Builds without debug assertions call `body` once per index below [`COPIES`], each call with
/// its index as a literal, and loop beyond. The index is a constant wherever LLVM simplifies the
/// body, so limbs stay in registers and a carry chain stays in the flags from one call to the
/// next. The calls at and past `N` sit behind conditions on a constant, which fold before the
/// body inlines. Builds with debug assertions loop, which keeps their code and frames small.
#[inline(always)]
pub(crate) fn each<N: Unsigned>(mut body: impl FnMut(usize)) {
    if cfg!(debug_assertions) {
        for index in 0..N::USIZE {
            body(index);
        }
    } else {
        copies::<N>(&mut body);
    }
}

/// The calls of [`each`] in builds without debug assertions.
#[inline(always)]
fn copies<N: Unsigned>(body: &mut impl FnMut(usize)) {
    if N::USIZE > 0 {
        body(0);
    }
    if N::USIZE > 1 {
        body(1);
    }
    if N::USIZE > 2 {
        body(2);
    }
    if N::USIZE > 3 {
        body(3);
    }
    if N::USIZE > 4 {
        body(4);
    }
    if N::USIZE > 5 {
        body(5);
    }
    if N::USIZE > 6 {
        body(6);
    }
    if N::USIZE > 7 {
        body(7);
    }
    if N::USIZE > 8 {
        body(8);
    }
    if N::USIZE > 9 {
        body(9);
    }
    if N::USIZE > 10 {
        body(10);
    }
    if N::USIZE > 11 {
        body(11);
    }
    if N::USIZE > 12 {
        body(12);
    }
    if N::USIZE > 13 {
        body(13);
    }
    if N::USIZE > 14 {
        body(14);
    }
    if N::USIZE > 15 {
        body(15);
    }
    for index in COPIES..N::USIZE {
        body(index);
    }
}

#[cfg(test)]
mod tests;
