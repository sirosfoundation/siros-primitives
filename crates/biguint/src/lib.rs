#![doc = include_str!("../README.md")]
#![no_std]
#![forbid(unsafe_code)]
#![doc(test(attr(forbid(unsafe_code))))]
#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable,
        clippy::todo,
        clippy::unimplemented,
        clippy::indexing_slicing
    )
)]

mod add_sub;
mod bits;
mod bitwise;
mod bytes;
mod cmp;
mod convert;
mod div;
mod fmt;
mod kernel;
mod limb;
mod modular;
mod mul;
mod nonzero;
mod num;
mod permute;
mod pow;
mod recursion;
mod shift;
mod stream;
mod uint;
mod unroll;

/// The array type of limbs and encodings.
pub use generic_array;
/// The type-level integers that name widths.
pub use typenum;

pub use bits::{Bits, Bits8, Bits16, Bits32, Bits64, Bits128, Bits256, Bits384, Bits512};
pub use convert::OutOfRange;
pub use mul::WideningMul;
pub use nonzero::NonZeroUint;
pub use num::{Exponent, One, Pow, Square};
pub use uint::{U8, U16, U32, U64, U128, U256, U384, U512, Uint};
