//! The error of the conversions into a field.

use biguint::OutOfRange;

/// The value has a term of degree `n` or above, so it is not an element of `GF(2^n)`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("the value has a term at or above the degree of the field")]
pub struct OutOfField;

impl From<OutOfRange> for OutOfField {
    #[inline]
    fn from(_: OutOfRange) -> Self {
        Self
    }
}
