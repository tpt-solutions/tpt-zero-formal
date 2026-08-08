#![doc = include_str!("../README.md")]

#![no_std]
#![warn(missing_docs)]
#![forbid(unsafe_code)]
// This crate is fundamentally about `as` casts; the pedantic cast lints are
// noise here because every conversion is checked for exactness by `SafeCast`.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::float_cmp
)]

mod error;
mod float;
mod int;

pub use error::CastError;

/// Converts `Self` into `T`, failing if the value cannot be represented in
/// `T` exactly.
///
/// Implemented for every pair of primitive integer types (backed by
/// [`core::convert::TryFrom`]), and manually for every integer-to-float,
/// float-to-integer, and `f64 <-> f32` pair, none of which `core` provides
/// a checked conversion for on its own.
///
/// A single blanket `impl<T, U> SafeCast<U> for T where U: TryFrom<T>`
/// would be more concise, but Rust's coherence checker rejects concrete
/// float impls alongside it: a *foreign* trait bound like `TryFrom` is
/// treated as something upstream `core` could satisfy for new type pairs
/// in a future release, so every pair is implemented explicitly instead.
pub trait SafeCast<T> {
    /// Attempts the cast, returning [`CastError`] if `T` cannot represent
    /// `self` exactly.
    ///
    /// # Errors
    ///
    /// Returns [`CastError`] if the value is out of range for `T`, has a
    /// fractional part that `T` cannot hold, or is not finite.
    fn safe_cast(self) -> Result<T, CastError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_narrowing_succeeds_in_range() {
        let v: Result<u8, _> = 42i32.safe_cast();
        assert_eq!(v, Ok(42u8));
    }

    #[test]
    fn integer_narrowing_fails_out_of_range() {
        let v: Result<u8, _> = 256i32.safe_cast();
        assert!(v.is_err());
    }

    #[test]
    fn integer_narrowing_fails_negative_to_unsigned() {
        let v: Result<u32, _> = (-1i32).safe_cast();
        assert!(v.is_err());
    }

    #[test]
    fn identity_cast_always_succeeds() {
        let v: Result<i32, _> = 7i32.safe_cast();
        assert_eq!(v, Ok(7));
    }

    #[test]
    fn error_reports_type_names() {
        let err = <i32 as SafeCast<u8>>::safe_cast(999).unwrap_err();
        assert_eq!(err.from_type(), "i32");
        assert_eq!(err.to_type(), "u8");
    }
}
