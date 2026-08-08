//! Design-by-contract precondition and postcondition macros for `no_std`.
//! Part of the
//! [tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal)
//! ecosystem.
//!
//! [`requires!`] and [`ensures!`] encode function invariants directly at
//! the call site: [`requires!`] checks a *precondition* at the top of a
//! function, and [`ensures!`] checks a *postcondition* before it returns.
//!
//! By default both macros use [`core::debug_assert!`], so they cost nothing
//! in release builds. Enable the `checked` feature to make them use
//! [`core::assert!`] instead, so the contracts are enforced even in release.
//!
//! ```
//! use out_zero_contract::{requires, ensures};
//!
//! fn div(a: i32, b: i32) -> i32 {
//!     requires!(b != 0, "divisor must be non-zero");
//!     let q = a / b;
//!     ensures!(b * q == a - (a % b), "division identity holds");
//!     q
//! }
//!
//! assert_eq!(div(10, 2), 5);
//! ```
//!
//! In a `release` build the above checks are compiled away; with
//! `--features checked` they are always evaluated.

#![no_std]
#![warn(missing_docs)]
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]

/// Internal helper that expands to [`core::debug_assert!`] or
/// [`core::assert!`] depending on whether this crate was built with the
/// `checked` feature. Not part of the public API.
///
/// The `checked` flag is resolved in *this* crate (the two definitions below
/// are gated by `#[cfg]` at the item level, which is resolved when this crate
/// is compiled), not in the calling crate — so a `#[cfg(feature = "checked")]`
/// written *inside* an exported macro body would silently read the caller's
/// features and do nothing.
#[cfg(not(feature = "checked"))]
#[doc(hidden)]
#[macro_export]
macro_rules! __contract_assert {
    ($($t:tt)*) => {{
        ::core::debug_assert!($($t)*);
    }};
}

/// Internal helper that expands to [`core::debug_assert!`] or
/// [`core::assert!`] depending on whether this crate was built with the
/// `checked` feature. Not part of the public API.
#[cfg(feature = "checked")]
#[doc(hidden)]
#[macro_export]
macro_rules! __contract_assert {
    ($($t:tt)*) => {{
        ::core::assert!($($t)*);
    }};
}

/// Declares a function *precondition*: an invariant that must hold when the
/// function is entered.
///
/// By default this expands to [`core::debug_assert!`], so the check is a
/// no-op in release builds (zero-cost). Enable the `checked` feature to make
/// it always assert via [`core::assert!`].
///
/// # Examples
///
/// ```
/// use out_zero_contract::requires;
///
/// fn push_index(len: usize, capacity: usize, index: usize) {
///     requires!(index < capacity, "index out of capacity");
///     requires!(index <= len, "index must not exceed length");
/// }
///
/// push_index(1, 4, 0);
/// ```
///
/// # Panics
///
/// Panics in debug builds (or always, with the `checked` feature) if
/// `$cond` is `false`.
///
/// # Note on feature resolution
///
/// The `checked` flag is resolved in *this* crate, not in the calling crate.
/// A `#[cfg(feature = "checked")]` written *inside* an exported macro body is
/// resolved where the macro is expanded (the caller), so it silently reads
/// the wrong features. These macros therefore delegate to the
/// `__contract_assert` helper, whose two definitions are selected by
/// `#[cfg]` at this crate's compile time.
#[macro_export]
macro_rules! requires {
    ($cond:expr $(,)?) => {{
        $crate::__contract_assert!($cond);
    }};
    ($cond:expr, $($msg:tt)+) => {{
        $crate::__contract_assert!($cond, $($msg)+);
    }};
}

/// Declares a function *postcondition*: an invariant that must hold when the
/// function exits.
///
/// By default this expands to [`core::debug_assert!`], so the check is a
/// no-op in release builds (zero-cost). Enable the `checked` feature to make
/// it always assert via [`core::assert!`].
///
/// # Examples
///
/// ```
/// use out_zero_contract::ensures;
///
/// fn checked_add(a: u32, b: u32) -> u32 {
///     let sum = a + b;
///     ensures!(sum >= a, "addition did not wrap");
///     sum
/// }
///
/// let _ = checked_add(2, 3);
/// ```
///
/// # Panics
///
/// Panics in debug builds (or always, with the `checked` feature) if
/// `$cond` is `false`.
#[macro_export]
macro_rules! ensures {
    ($cond:expr $(,)?) => {{
        $crate::__contract_assert!($cond);
    }};
    ($cond:expr, $($msg:tt)+) => {{
        $crate::__contract_assert!($cond, $($msg)+);
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requires_passes_on_valid_input() {
        fn f(x: i32) {
            requires!(x >= 0, "x must be non-negative");
        }
        f(0);
        f(5);
    }

    #[test]
    #[should_panic(expected = "non-negative")]
    fn requires_panics_on_invalid_input() {
        fn f(x: i32) {
            requires!(x >= 0, "x must be non-negative");
        }
        f(-1);
    }

    #[test]
    fn ensures_passes_on_valid_output() {
        fn add_one(x: i32) -> i32 {
            let y = x + 1;
            ensures!(y > x, "result must exceed input");
            y
        }
        assert_eq!(add_one(41), 42);
    }

    #[test]
    #[should_panic(expected = "positive")]
    fn ensures_panics_on_invalid_output() {
        fn bad(x: i32) -> i32 {
            let y = x;
            ensures!(y > 0, "result must be positive");
            y
        }
        let _ = bad(-1);
    }

    #[test]
    fn both_macros_coexist() {
        fn mid(a: i32, b: i32) -> i32 {
            requires!(a <= b, "a <= b");
            let m = (a + b) / 2;
            ensures!(m >= a && m <= b, "midpoint in range");
            m
        }
        assert_eq!(mid(2, 8), 5);
    }
}

#[cfg(all(test, feature = "checked"))]
mod checked_tests {
    use super::*;

    #[test]
    #[should_panic(expected = "checked panics in release too")]
    fn checked_requires_panics_even_in_release() {
        fn f(x: i32) {
            requires!(x >= 0, "checked panics in release too");
        }
        f(-1);
    }
}
