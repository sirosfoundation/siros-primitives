//! Fixed-width unsigned integers with a type-level bit width.
//!
//! [`Uint<N>`] holds an integer of exactly `N` bits for any `typenum` width `N`, stored in
//! `ceil(N / 64)` little-endian `u64` limbs. Arithmetic wraps modulo `2^N`.
//!
//! Every multi-limb operation is a lazy, zero-extended stream of limbs. The compiler unrolls
//! each stream into straight-line code over the limbs.
//!
//! ```
//! use biguint::typenum::U521;
//! use biguint::{U256, U512, Uint, WideningMul};
//!
//! let top = U256::ONE << 255usize;
//! assert_eq!(U256::from_u64(3) * top, top);
//! assert_eq!(top.widening_mul(top), U512::ONE << 510usize);
//! assert_eq!(
//!     U256::from_u64(7).mul_mod(U256::from_u64(9), U256::from_u64(10)),
//!     Some(U256::from_u64(3))
//! );
//!
//! let max521: Uint<U521> = !Uint::ZERO;
//! assert_eq!(max521.bit_len(), 521);
//! ```

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
mod limb;
mod mul;
mod num;
mod permute;
mod pow;
mod shift;
mod stream;
mod uint;

/// The array type of limbs and encodings.
pub use generic_array;
/// The type-level integers that name widths.
pub use typenum;

pub use bits::{Bits, Bits8, Bits16, Bits32, Bits64, Bits128, Bits256, Bits384, Bits512};
pub use convert::OutOfRange;
pub use mul::WideningMul;
pub use num::{Exponent, One, Pow, Square};
pub use uint::{U8, U16, U32, U64, U128, U256, U384, U512, Uint};
