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

mod element;
mod error;
mod invert;
mod kernel;
mod mul;
mod plan;
mod reduce;
mod standard;

/// The integers that hold the polynomials of the elements, and the traits of the exponents.
pub use biguint;
/// The array type of limbs and encodings.
pub use generic_array;
/// The type-level integers that name degrees.
pub use typenum;

pub use biguint::{Exponent, One, Pow, Square, Uint};
pub use element::{Bytes, Gf2, Limbs};
pub use error::OutOfField;
pub use plan::Polynomial;
pub use standard::{
    Aes, B233, B571, Gcm, Gf2p8, Gf2p16, Gf2p64, Gf2p128, Gf2p233, Gf2p256, Gf2p571, Poly16,
    Poly64, Poly256,
};
