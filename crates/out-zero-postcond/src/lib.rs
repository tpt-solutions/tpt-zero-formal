#![doc = include_str!("../README.md")]

#![no_std]
#![warn(missing_docs)]
#![forbid(unsafe_code)]

/// Error returned when a postcondition is violated.
///
/// Holds a `&'static str` message describing which postcondition failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PostconditionError {
    msg: &'static str,
}

impl PostconditionError {
    /// Creates a new postcondition error with the given message.
    ///
    /// The message should describe the postcondition that was violated.
    #[must_use]
    pub const fn new(msg: &'static str) -> Self {
        Self { msg }
    }

    /// Returns the message describing the violated postcondition.
    #[must_use]
    pub const fn message(&self) -> &'static str {
        self.msg
    }
}

impl core::fmt::Display for PostconditionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "postcondition violated: {}", self.msg)
    }
}

impl core::error::Error for PostconditionError {}

/// Checks a postcondition, returning [`Err`] if it does not hold.
///
/// Unlike [`postcondition!`], this check is always evaluated regardless of
/// build profile, so it is suitable for code paths that must be verified even
/// in release builds.
///
/// # Errors
///
/// Returns [`Err`] containing a [`PostconditionError`] with the provided
/// message when `cond` is `false`. The message must be `'static` because it
/// is stored directly inside the returned error.
///
/// # Examples
///
/// ```
/// use out_zero_postcond::{check, PostconditionError};
///
/// let r: Result<(), PostconditionError> = check(2 + 3 == 5, "arithmetic holds");
/// assert!(r.is_ok());
/// ```
pub fn check(cond: bool, msg: &'static str) -> Result<(), PostconditionError> {
    if cond {
        Ok(())
    } else {
        Err(PostconditionError::new(msg))
    }
}

/// Asserts a postcondition at the end of a function, early-returning the error
/// if it is violated.
///
/// This is the error-returning analogue of `ensures!` from
/// `out-zero-contract`: instead of panicking, it returns
/// `Err(PostconditionError)` from the enclosing function. The function must
/// return a `Result<_, PostconditionError>` (or another type whose `?`
/// operator accepts it).
///
/// # Examples
///
/// ```
/// use out_zero_postcond::{ensure_postcond, PostconditionError};
///
/// fn double(x: u32) -> Result<u32, PostconditionError> {
///     let y = x.wrapping_mul(2);
///     ensure_postcond!(y >= x, "doubling did not underflow");
///     Ok(y)
/// }
///
/// assert_eq!(double(21), Ok(42));
/// ```
///
/// [`Err`]: core::result::Result::Err
#[macro_export]
macro_rules! ensure_postcond {
    ($cond:expr $(,)?) => {{
        if !($cond) {
            return ::core::result::Result::Err($crate::PostconditionError::new(
                ::core::concat!("postcondition failed: ", ::core::stringify!($cond)),
            ));
        }
    }};
    ($cond:expr, $msg:expr $(,)?) => {{
        if !($cond) {
            return ::core::result::Result::Err($crate::PostconditionError::new($msg));
        }
    }};
}

/// Checks a postcondition, returning [`Err`] when violated.
///
/// In `debug_assertions` builds the condition is evaluated and, if it fails,
/// the macro expands to `Err(PostconditionError)`. In release builds the
/// condition is *not* evaluated and the macro expands to `Ok(())`, matching
/// the zero-cost behaviour of [`core::debug_assert!`].
///
/// This makes the macro ideal for postconditions you want enforced while
/// developing and testing, but which must not cost anything in production.
///
/// # Examples
///
/// ```
/// use out_zero_postcond::{postcondition, PostconditionError};
///
/// fn inc(x: u32) -> Result<u32, PostconditionError> {
///     let y = x + 1;
///     postcondition!(y > x, "increment increased the value")?;
///     Ok(y)
/// }
///
/// assert_eq!(inc(41), Ok(42));
/// ```
///
/// [`Err`]: core::result::Result::Err
#[macro_export]
macro_rules! postcondition {
    ($cond:expr $(,)?) => {{
        #[cfg(debug_assertions)]
        {
            if !($cond) {
                ::core::result::Result::Err($crate::PostconditionError::new(::core::concat!(
                    "postcondition failed: ",
                    ::core::stringify!($cond)
                )))
            } else {
                ::core::result::Result::Ok(())
            }
        }
        #[cfg(not(debug_assertions))]
        {
            let _ = $cond;
            ::core::result::Result::Ok(())
        }
    }};
    ($cond:expr, $msg:expr $(,)?) => {{
        #[cfg(debug_assertions)]
        {
            if !($cond) {
                ::core::result::Result::Err($crate::PostconditionError::new($msg))
            } else {
                ::core::result::Result::Ok(())
            }
        }
        #[cfg(not(debug_assertions))]
        {
            let _ = $cond;
            ::core::result::Result::Ok(())
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_holds_message() {
        let e = PostconditionError::new("boom");
        assert_eq!(e.message(), "boom");
    }

    #[test]
    fn check_passes_when_true() {
        assert_eq!(check(true, "ok"), Ok(()));
    }

    #[test]
    fn check_fails_when_false() {
        let r: Result<(), PostconditionError> = check(false, "nope");
        assert_eq!(r, Err(PostconditionError::new("nope")));
    }

    #[test]
    fn ensure_postcond_passes() {
        fn f(x: u32) -> Result<u32, PostconditionError> {
            let y = x + 1;
            ensure_postcond!(y > x, "y exceeds x");
            Ok(y)
        }
        assert_eq!(f(41), Ok(42));
    }

    #[test]
    fn ensure_postcond_fails() {
        fn f(x: u32) -> Result<(), PostconditionError> {
            let y = x.wrapping_sub(1);
            ensure_postcond!(y < x, "y is below x");
            Ok(())
        }
        assert_eq!(f(0), Err(PostconditionError::new("y is below x")));
    }

    #[test]
    fn ensure_postcond_generated_message() {
        fn f() -> Result<(), PostconditionError> {
            ensure_postcond!(false);
            Ok(())
        }
        let e = f().unwrap_err();
        assert!(e.message().contains("false"));
    }

    #[test]
    fn postcondition_passes() {
        fn f(x: u32) -> Result<u32, PostconditionError> {
            let y = x + 1;
            postcondition!(y > x, "y exceeds x")?;
            Ok(y)
        }
        assert_eq!(f(41), Ok(42));
    }

    #[test]
    fn postcondition_fails_in_debug() {
        fn f() -> Result<(), PostconditionError> {
            postcondition!(false, "always false")?;
            Ok(())
        }
        let r = f();
        if cfg!(debug_assertions) {
            assert_eq!(r, Err(PostconditionError::new("always false")));
        } else {
            assert_eq!(r, Ok(()));
        }
    }

    #[test]
    fn display_describes_violation() {
        let e = PostconditionError::new("msg");
        assert_eq!(e.message(), "msg");
    }
}
