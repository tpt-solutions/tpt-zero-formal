//! Precondition checking helpers built on design-by-contract for `no_std`.
//! Part of the
//! [tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal)
//! ecosystem.
//!
//! A *precondition* is an invariant that must hold when a function is entered.
//! The companion [`out-zero-contract`](https://docs.rs/out-zero-contract)
//! crate encodes preconditions with the [`requires!`](https://docs.rs/out-zero-contract)
//! macro, which *asserts* the condition (panicking in debug, or always with
//! its `checked` feature). This crate instead *checks* preconditions: the
//! condition is turned into a recoverable [`PreconditionError`] that the
//! caller's function can propagate with `?`.
//!
//! ```
//! use out_zero_precond::{check, PreconditionError};
//!
//! fn sqrt(x: f64) -> Result<f64, PreconditionError> {
//!     check(x >= 0.0, "x must be non-negative")?;
//!     Ok(x.sqrt())
//! }
//!
//! assert_eq!(sqrt(4.0), Ok(2.0));
//! assert!(sqrt(-1.0).is_err());
//! ```
//!
//! For call-site ergonomics the [`requires_ok!`] macro performs the same check
//! and returns `Err(PreconditionError)` early from the enclosing function, so a
//! validating function reads almost like a contract:
//!
//! ```
//! use out_zero_precond::{requires_ok, PreconditionError};
//!
//! fn push_at(slice: &[i32], index: usize) -> Result<i32, PreconditionError> {
//!     requires_ok!(index < slice.len(), "index out of bounds");
//!     Ok(slice[index])
//! }
//!
//! assert_eq!(push_at(&[10, 20], 1), Ok(20));
//! assert!(push_at(&[10, 20], 5).is_err());
//! ```

#![no_std]
#![warn(missing_docs)]
#![forbid(unsafe_code)]

use core::fmt;

#[cfg(feature = "std")]
extern crate std;

/// Error returned when a precondition is violated.
///
/// The error holds a `&'static str` message describing the failed condition,
/// so it is usable in a `no_std` context without any allocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreconditionError {
    /// Human-readable description of the violated precondition.
    msg: &'static str,
}

impl PreconditionError {
    /// Creates a new precondition error with the given message.
    ///
    /// # Examples
    ///
    /// ```
    /// use out_zero_precond::PreconditionError;
    ///
    /// let err = PreconditionError::new("value must be positive");
    /// assert_eq!(err.msg(), "value must be positive");
    /// ```
    #[must_use]
    pub const fn new(msg: &'static str) -> Self {
        Self { msg }
    }

    /// Returns the message describing the violated precondition.
    ///
    /// # Examples
    ///
    /// ```
    /// use out_zero_precond::PreconditionError;
    ///
    /// let err = PreconditionError::new("non-zero divisor");
    /// assert_eq!(err.msg(), "non-zero divisor");
    /// ```
    #[must_use]
    pub const fn msg(&self) -> &'static str {
        self.msg
    }
}

impl fmt::Display for PreconditionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "precondition violated: {}", self.msg)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for PreconditionError {}

/// Checks a precondition, returning [`PreconditionError`] if it fails.
///
/// This is the explicit, non-macro form of the check. It evaluates `cond` and,
/// when `false`, builds a [`PreconditionError`] carrying `msg`. The caller
/// typically propagates the error with `?`:
///
/// ```
/// use out_zero_precond::{check, PreconditionError};
///
/// fn divide(a: i32, b: i32) -> Result<i32, PreconditionError> {
///     check(b != 0, "divisor must be non-zero")?;
///     Ok(a / b)
/// }
///
/// assert_eq!(divide(10, 2), Ok(5));
/// assert!(divide(10, 0).is_err());
/// ```
///
/// The check is always evaluated, even in release builds, so it is suitable for
/// external-input validation where relying on a `debug_assert!` would be unsound.
///
/// # Errors
///
/// Returns [`PreconditionError`] when `cond` is `false`.
pub fn check(cond: bool, msg: &'static str) -> Result<(), PreconditionError> {
    if cond {
        Ok(())
    } else {
        Err(PreconditionError::new(msg))
    }
}

/// Checks a precondition at the top of a function, returning early when it
/// fails.
///
/// This macro evaluates `$cond`; when it is `false` it immediately returns
/// `Err(PreconditionError::new($msg))` from the enclosing function, so the
/// function must return a `Result<_, PreconditionError>`. It is a thin wrapper
/// over [`check`] that provides the early-return `?`-free ergonomics:
///
/// ```
/// use out_zero_precond::{requires_ok, PreconditionError};
///
/// fn first_three(slice: &[i32]) -> Result<i32, PreconditionError> {
///     requires_ok!(slice.len() >= 3, "slice must have at least 3 elements");
///     Ok(slice[0] + slice[1] + slice[2])
/// }
///
/// assert_eq!(first_three(&[1, 2, 3]), Ok(6));
/// assert!(first_three(&[1, 2]).is_err());
/// ```
///
/// The check is always evaluated, even in release builds.
///
/// # Errors
///
/// Returns early with [`PreconditionError`] when `$cond` is `false`.
#[macro_export]
macro_rules! requires_ok {
    ($cond:expr, $msg:expr $(,)?) => {{
        if !($cond) {
            return ::core::result::Result::Err($crate::PreconditionError::new($msg));
        }
    }};
    ($cond:expr $(,)?) => {{
        $crate::requires_ok!($cond, "precondition violated");
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_passes_on_true() {
        assert_eq!(check(true, "always"), Ok(()));
    }

    #[test]
    fn check_fails_on_false() {
        let err = check(false, "must be even").unwrap_err();
        assert_eq!(err.msg(), "must be even");
    }

    #[test]
    fn display_describes_condition() {
        use core::fmt::{self, Write};
        struct Buf {
            data: [u8; 64],
            len: usize,
        }
        impl Write for Buf {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                let bytes = s.as_bytes();
                if self.len + bytes.len() > self.data.len() {
                    return Err(fmt::Error);
                }
                self.data[self.len..self.len + bytes.len()].copy_from_slice(bytes);
                self.len += bytes.len();
                Ok(())
            }
        }
        let err = PreconditionError::new("x > 0");
        let mut buf = Buf {
            data: [0u8; 64],
            len: 0,
        };
        write!(buf, "{err}").unwrap();
        let text = core::str::from_utf8(&buf.data[..buf.len]).unwrap();
        assert_eq!(text, "precondition violated: x > 0");
    }

    #[test]
    fn requires_ok_returns_ok_when_satisfied() {
        fn mid(a: i32, b: i32) -> Result<i32, PreconditionError> {
            requires_ok!(a <= b, "a must be <= b");
            Ok((a + b) / 2)
        }
        assert_eq!(mid(2, 8), Ok(5));
    }

    #[test]
    fn requires_ok_returns_err_when_violated() {
        fn mid(a: i32, b: i32) -> Result<i32, PreconditionError> {
            requires_ok!(a <= b, "a must be <= b");
            Ok((a + b) / 2)
        }
        assert!(mid(8, 2).is_err());
    }

    #[test]
    fn requires_ok_default_message() {
        fn f(x: i32) -> Result<i32, PreconditionError> {
            requires_ok!(x >= 0);
            Ok(x)
        }
        assert!(f(-1).unwrap_err().msg().starts_with("precondition"));
    }

    #[test]
    fn validating_function_early_returns() {
        fn sqrt(x: f64) -> Result<f64, PreconditionError> {
            check(x >= 0.0, "x must be non-negative")?;
            Ok(x.sqrt())
        }
        assert_eq!(sqrt(9.0), Ok(3.0));
        assert!(sqrt(-4.0).is_err());
    }
}
