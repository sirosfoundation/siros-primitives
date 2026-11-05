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
mod ct;
mod div;
mod each;
mod field;
mod fmt;
mod kernel;
mod limb;
mod modular;
mod modulus;
mod mul;
mod nonzero;
mod num;
mod num_traits_impls;
mod permute;
mod pow;
mod recursion;
mod scan;
mod shift;
mod stream;
mod uint;

/// The constant-time wrapper and the traits that `Uint` and `Residue` implement.
pub use constant_time;
pub use constant_time::ConstantTime;
/// The array type of limbs and encodings.
pub use generic_array;
/// The type-level integers that name widths.
pub use typenum;

pub use bits::{Bits, Bits8, Bits16, Bits32, Bits64, Bits128, Bits256, Bits384, Bits512};
pub use convert::OutOfRange;
pub use field::Field;
pub use modulus::{ConstantTimeResidue, Modulus, NotReduced, Residue, Ring};
pub use mul::WideningMul;
#[doc(hidden)]
pub use nonzero::seal;
pub use nonzero::{Divisor, IsZero, NonZero, NonZeroUint};
pub use num::{Exponent, One, Pow, Square};
pub use shift::ShiftAmount;
pub use uint::{U8, U16, U32, U64, U128, U256, U384, U512, Uint};
