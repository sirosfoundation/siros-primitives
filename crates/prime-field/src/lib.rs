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

mod arith;
mod chain;
mod ct;
mod divide;
mod field;
mod invert;
mod kernel;
mod limb;
mod params;
mod scale;
mod sqrt;
pub mod standard;
mod unrolled;
mod width;

/// The integers that elements convert from and to.
pub use biguint;

pub use biguint::{ConstantTime, NonZero};
pub use field::{Fp, Modulus, NotCanonical};
pub use width::Width;
