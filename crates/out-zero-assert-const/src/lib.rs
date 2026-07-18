//! Compile-time assertion macros and `const fn` helpers for `no_std` const
//! contexts. Part of the
//! [tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal)
//! ecosystem.
//!
//! [`const_assert!`] fails the *build*, not a runtime call, when the
//! condition does not hold, so invariants like `MIN <= MAX` on a
//! `BoundedInt<const MIN: i64, const MAX: i64>` are caught at compile time
//! for every concrete instantiation that is actually used.
//!
//! ```
//! use out_zero_assert_const::const_assert;
//!
//! const_assert!(core::mem::size_of::<u32>() == 4);
//! ```
//!
//! A failing assertion is a compile error, not a runtime panic:
//!
//! ```compile_fail
//! use out_zero_assert_const::const_assert;
//!
//! const_assert!(core::mem::size_of::<u32>() == 8);
//! ```

#![no_std]
#![warn(missing_docs)]
#![forbid(unsafe_code)]

/// Asserts that a `const`-evaluable boolean expression is `true`, failing
/// the build if it is not.
///
/// Unlike [`core::assert!`], this is checked once at compile time rather
/// than on every call at runtime, and costs nothing in the compiled binary.
///
/// # Examples
///
/// ```
/// use out_zero_assert_const::const_assert;
///
/// const N: usize = 4;
/// const_assert!(N > 0);
/// const_assert!(N > 0, "N must be positive");
/// ```
#[macro_export]
macro_rules! const_assert {
    ($cond:expr $(,)?) => {
        const _: () = ::core::assert!($cond);
    };
    ($cond:expr, $($msg:tt)+) => {
        const _: () = ::core::assert!($cond, $($msg)+);
    };
}

/// Asserts that two `const`-evaluable expressions are equal, failing the
/// build if they are not.
///
/// # Examples
///
/// ```
/// use out_zero_assert_const::const_assert_eq;
///
/// const_assert_eq!(core::mem::size_of::<u8>(), 1);
/// ```
#[macro_export]
macro_rules! const_assert_eq {
    ($left:expr, $right:expr $(,)?) => {
        const _: () = ::core::assert!($left == $right);
    };
}

/// Asserts that two `const`-evaluable expressions are not equal, failing
/// the build if they are.
///
/// # Examples
///
/// ```
/// use out_zero_assert_const::const_assert_ne;
///
/// const_assert_ne!(core::mem::size_of::<u8>(), core::mem::size_of::<u16>());
/// ```
#[macro_export]
macro_rules! const_assert_ne {
    ($left:expr, $right:expr $(,)?) => {
        const _: () = ::core::assert!($left != $right);
    };
}

/// Asserts, in a `const fn` body, that `min <= max`, returning `max` for
/// use directly in a `const` binding.
///
/// This is the pattern `out-zero-bounded` uses to reject an inverted
/// `BoundedInt<MIN, MAX>` range at the point it is instantiated, rather
/// than deferring the failure to first use.
///
/// # Examples
///
/// ```
/// use out_zero_assert_const::assert_i64_le;
///
/// const MAX: i64 = assert_i64_le(0, 100);
/// assert_eq!(MAX, 100);
/// ```
///
/// # Panics
///
/// Panics (at compile time in a `const` context, at runtime otherwise) if
/// `min > max`.
#[must_use]
pub const fn assert_i64_le(min: i64, max: i64) -> i64 {
    assert!(min <= max, "assert_i64_le: min must be <= max");
    max
}

/// `usize` counterpart of [`assert_i64_le`].
///
/// # Panics
///
/// Panics (at compile time in a `const` context, at runtime otherwise) if
/// `min > max`.
#[must_use]
pub const fn assert_usize_le(min: usize, max: usize) -> usize {
    assert!(min <= max, "assert_usize_le: min must be <= max");
    max
}

#[cfg(test)]
mod tests {
    use super::*;

    const_assert!(core::mem::size_of::<u32>() == 4);
    const_assert_eq!(core::mem::size_of::<u8>(), 1);
    const_assert_ne!(core::mem::size_of::<u8>(), core::mem::size_of::<u16>());

    #[test]
    fn assert_i64_le_accepts_valid_range() {
        assert_eq!(assert_i64_le(0, 100), 100);
        assert_eq!(assert_i64_le(-5, -5), -5);
    }

    #[test]
    #[should_panic(expected = "assert_i64_le")]
    fn assert_i64_le_rejects_inverted_range() {
        let _ = assert_i64_le(10, 5);
    }

    #[test]
    fn assert_usize_le_accepts_valid_range() {
        assert_eq!(assert_usize_le(0, 100), 100);
    }

    #[test]
    #[should_panic(expected = "assert_usize_le")]
    fn assert_usize_le_rejects_inverted_range() {
        let _ = assert_usize_le(10, 5);
    }
}
