//! Loops unrolled in the source.
//!
//! In one codegen unit LLVM unrolls a constant-trip loop late, after the passes that keep a carry
//! in the flags, and the unrolled carry code saves its carries with `setb`. A loop written as one
//! copy per index reaches those passes as straight-line code.

/// The indices that [`unroll`] copies; a loop runs the indices above.
pub(crate) const UNROLLED: usize = 16;

/// Runs `$body` with `$index` bound to each of `0..$len` in order: one copy per index below
/// [`UNROLLED`], then a loop. With a constant `$len`, the copies at and above it fold away and the
/// loop runs no iteration.
macro_rules! unroll {
    ($index:ident in 0..$len:expr => $body:block) => {
        $crate::unroll::unroll!(@copies $index, $len, $body; 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15)
    };
    (@copies $index:ident, $len:expr, $body:block; $($copy:literal)*) => {{
        let len: usize = $len;
        $(
            if $copy < len {
                let $index: usize = $copy;
                $body
            }
        )*
        let mut $index: usize = $crate::unroll::UNROLLED;
        while $index < len {
            $body
            $index += 1;
        }
    }};
}

pub(crate) use unroll;
