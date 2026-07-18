//! `SafeCast` impls between every pair of primitive integer types, backed
//! by [`core::convert::TryFrom`] (which `core` implements for every such
//! pair, including a type paired with itself).

use crate::{CastError, SafeCast};

macro_rules! impl_int_cast {
    ($from:ty => $($to:ty),+ $(,)?) => {
        $(
            impl SafeCast<$to> for $from {
                fn safe_cast(self) -> Result<$to, CastError> {
                    <$to>::try_from(self)
                        .map_err(|_| CastError::new(stringify!($from), stringify!($to)))
                }
            }
        )+
    };
}

macro_rules! impl_int_cast_all {
    ($($from:ty),+ $(,)?) => {
        $(
            impl_int_cast!(
                $from => i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
            );
        )+
    };
}

impl_int_cast_all!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_type_casts_to_itself() {
        let v: Result<i8, _> = 5i8.safe_cast();
        assert_eq!(v, Ok(5));
        let v: Result<u128, _> = 5u128.safe_cast();
        assert_eq!(v, Ok(5));
    }

    #[test]
    fn widening_always_succeeds() {
        let v: Result<i128, _> = i64::MIN.safe_cast();
        assert_eq!(v, Ok(i64::MIN as i128));
    }

    #[test]
    fn narrowing_out_of_range_fails() {
        let v: Result<i8, _> = 200i32.safe_cast();
        assert!(v.is_err());
    }

    #[test]
    fn signed_to_unsigned_negative_fails() {
        let v: Result<usize, _> = (-1isize).safe_cast();
        assert!(v.is_err());
    }
}
