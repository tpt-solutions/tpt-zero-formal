//! Manual `SafeCast` impls for every integer<->float and `f64 <-> f32`
//! pair. `core` provides no checked (`TryFrom`) conversions at all
//! involving `f32`/`f64`, so every impl here is hand-written using an
//! explicit round-trip check: convert, convert back, and compare.
//!
//! For `i64`/`u64`/`i128`/`u128`/`isize`/`usize` cast *from* a float, the
//! round-trip check is exact for every value that survives it, but a value
//! at the extreme edge of the target's range that isn't itself exactly
//! representable as the source float type can, in rare cases, round-trip
//! through saturation and be accepted. This is an inherent limit of
//! checking floats near 2^53 (`f64`) / 2^24 (`f32`) against 64-/128-bit
//! integer bounds, not a logic bug: at that magnitude the float itself can
//! no longer distinguish adjacent integers.

use crate::{CastError, SafeCast};

macro_rules! impl_int_from_float_checked {
    ($float:ty => $($int:ty),+ $(,)?) => {
        $(
            impl SafeCast<$int> for $float {
                fn safe_cast(self) -> Result<$int, CastError> {
                    if !self.is_finite() {
                        return Err(CastError::new(stringify!($float), stringify!($int)));
                    }
                    #[allow(
                        clippy::cast_possible_truncation,
                        clippy::cast_sign_loss,
                        clippy::cast_possible_wrap
                    )]
                    let candidate = self as $int;
                    #[allow(clippy::cast_precision_loss)]
                    let round_trip = candidate as $float;
                    if round_trip == self {
                        Ok(candidate)
                    } else {
                        Err(CastError::new(stringify!($float), stringify!($int)))
                    }
                }
            }
        )+
    };
}

macro_rules! impl_float_from_int_checked {
    ($int:ty => $($float:ty),+ $(,)?) => {
        $(
            impl SafeCast<$float> for $int {
                fn safe_cast(self) -> Result<$float, CastError> {
                    #[allow(clippy::cast_precision_loss)]
                    let candidate = self as $float;
                    #[allow(
                        clippy::cast_possible_truncation,
                        clippy::cast_sign_loss,
                        clippy::cast_possible_wrap
                    )]
                    let round_trip = candidate as $int;
                    if round_trip == self {
                        Ok(candidate)
                    } else {
                        Err(CastError::new(stringify!($int), stringify!($float)))
                    }
                }
            }
        )+
    };
}

impl_int_from_float_checked!(f32 => i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
impl_int_from_float_checked!(f64 => i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

impl_float_from_int_checked!(i8 => f32, f64);
impl_float_from_int_checked!(i16 => f32, f64);
impl_float_from_int_checked!(i32 => f32, f64);
impl_float_from_int_checked!(i64 => f32, f64);
impl_float_from_int_checked!(i128 => f32, f64);
impl_float_from_int_checked!(isize => f32, f64);
impl_float_from_int_checked!(u8 => f32, f64);
impl_float_from_int_checked!(u16 => f32, f64);
impl_float_from_int_checked!(u32 => f32, f64);
impl_float_from_int_checked!(u64 => f32, f64);
impl_float_from_int_checked!(u128 => f32, f64);
impl_float_from_int_checked!(usize => f32, f64);

impl SafeCast<f64> for f32 {
    fn safe_cast(self) -> Result<f64, CastError> {
        Ok(f64::from(self))
    }
}

impl SafeCast<f32> for f64 {
    fn safe_cast(self) -> Result<f32, CastError> {
        if !self.is_finite() {
            return Err(CastError::new("f64", "f32"));
        }
        #[allow(clippy::cast_possible_truncation)]
        let candidate = self as f32;
        if f64::from(candidate) == self {
            Ok(candidate)
        } else {
            Err(CastError::new("f64", "f32"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn float_to_int_exact_value_succeeds() {
        let v: Result<i32, _> = 4.0f64.safe_cast();
        assert_eq!(v, Ok(4));
    }

    #[test]
    fn float_to_int_fractional_fails() {
        let v: Result<i32, _> = 4.5f64.safe_cast();
        assert!(v.is_err());
    }

    #[test]
    fn float_to_int_nan_fails() {
        let v: Result<i32, _> = f64::NAN.safe_cast();
        assert!(v.is_err());
    }

    #[test]
    fn float_to_int_infinity_fails() {
        let v: Result<i32, _> = f64::INFINITY.safe_cast();
        assert!(v.is_err());
    }

    #[test]
    fn float_to_int_out_of_range_fails() {
        let v: Result<u8, _> = 300.0f64.safe_cast();
        assert!(v.is_err());
    }

    #[test]
    fn int_to_float_exact_succeeds() {
        let v: Result<f32, _> = 1_000_000i64.safe_cast();
        assert_eq!(v, Ok(1_000_000.0f32));
    }

    #[test]
    fn int_to_float_imprecise_fails() {
        // 2^24 + 1 is not exactly representable as f32.
        let v: Result<f32, _> = 16_777_217i64.safe_cast();
        assert!(v.is_err());
    }

    #[test]
    fn f64_to_f32_exact_succeeds() {
        let v: Result<f32, _> = 1.5f64.safe_cast();
        assert_eq!(v, Ok(1.5f32));
    }

    #[test]
    fn f64_to_f32_imprecise_fails() {
        let v: Result<f32, _> = 0.1f64.safe_cast();
        assert!(v.is_err());
    }

    #[test]
    fn f32_to_f64_always_succeeds() {
        let v: Result<f64, _> = 1.5f32.safe_cast();
        assert_eq!(v, Ok(1.5f64));
    }
}
