//! State-machine invariant traits and zero-cost assertion macros for `no_std`.
//! Part of the
//! [tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal)
//! ecosystem.
//!
//! A *state machine* exposes transitions that must keep the object in a
//! well-formed state. An *invariant* is a property that holds in every
//! reachable state. This crate gives you a tiny [`Invariant`] trait to encode
//! that property, plus [`invariant!`] and [`check_invariant!`] macros that
//! assert it holds after each transition — compiled to a `debug_assert!`, so
//! they cost nothing in release builds.
//!
//! ```
//! use tpt_zero_invariant::{Invariant, invariant};
//!
//! struct Counter { count: u32, max: u32 }
//!
//! impl Invariant for Counter {
//!     fn check(&self) -> bool {
//!         self.count <= self.max
//!     }
//! }
//!
//! let mut c = Counter { count: 0, max: 10 };
//! c.count += 1;
//! invariant!(c.count <= c.max);
//! ```
//!
//! The same invariant can be checked through the trait:
//!
//! ```
//! use tpt_zero_invariant::{Invariant, check_invariant};
//!
//! struct Counter { count: u32, max: u32 }
//! impl Invariant for Counter {
//!     fn check(&self) -> bool { self.count <= self.max }
//! }
//!
//! let c = Counter { count: 0, max: 10 };
//! check_invariant!(c);
//! ```

#![no_std]
#![warn(missing_docs)]
#![forbid(unsafe_code)]

/// A type with a boolean well-formedness property that must hold in every
/// reachable state.
///
/// Implement this for state-machine types so the property can be checked
/// uniformly through [`check_invariant`] and [`assert_invariant`], and so
/// callers can require "a type whose invariants have been verified".
///
/// # Examples
///
/// ```
/// use tpt_zero_invariant::Invariant;
///
/// struct Bounded { value: i32, min: i32, max: i32 }
///
/// impl Invariant for Bounded {
///     fn check(&self) -> bool {
///         self.value >= self.min && self.value <= self.max
///     }
/// }
/// ```
pub trait Invariant {
    /// Returns `true` when `self` satisfies the type's invariant.
    ///
    /// This must be a pure, side-effect-free predicate: given the same
    /// state it always returns the same answer, and it must never panic.
    fn check(&self) -> bool;
}

/// Asserts a boolean invariant expression holds, using [`core::debug_assert!`]
/// so the check is compiled away in release builds.
///
/// Use it after state-machine transitions to document and guard the
/// well-formedness of `self` without paying a runtime cost in optimized
/// builds.
///
/// # Examples
///
/// ```
/// use tpt_zero_invariant::invariant;
///
/// let mut v = Vec::new();
/// v.push(1);
/// invariant!(v.len() == 1);
/// ```
///
/// [`core::debug_assert!`]: https://doc.rust-lang.org/core/macro.debug_assert.html
#[macro_export]
macro_rules! invariant {
    ($cond:expr $(,)?) => {
        ::core::debug_assert!($cond);
    };
    ($cond:expr, $($msg:tt)+) => {
        ::core::debug_assert!($cond, $($msg)+);
    };
}

/// Checks that `value` satisfies [`Invariant::check`], using
/// [`core::debug_assert!`] so the check is compiled away in release builds.
///
/// Returns `value` unchanged so it can be threaded through a transition
/// while documenting the invariant at the call site.
///
/// # Examples
///
/// ```
/// use tpt_zero_invariant::{Invariant, check_invariant};
///
/// struct Counter { count: u32, max: u32 }
/// impl Invariant for Counter {
///     fn check(&self) -> bool { self.count <= self.max }
/// }
///
/// let c = Counter { count: 3, max: 10 };
/// let c = check_invariant!(c);
/// assert_eq!(c.count, 3);
/// ```
///
/// [`core::debug_assert!`]: https://doc.rust-lang.org/core/macro.debug_assert.html
#[macro_export]
macro_rules! check_invariant {
    ($value:expr $(,)?) => {{
        ::core::debug_assert!(
            $crate::Invariant::check(&$value),
            "invariant violated: {}",
            ::core::stringify!($value)
        );
        $value
    }};
    ($value:expr, $($msg:tt)+) => {{
        ::core::debug_assert!($crate::Invariant::check(&$value), $($msg)+);
        $value
    }};
}

/// Debug-asserts that `value` satisfies its [`Invariant`] and returns a
/// shared reference to it.
///
/// This is the borrowed counterpart to the [`check_invariant!`] macro: use
/// it when you already hold a reference and want to verify, but not move,
/// the value.
///
/// # Examples
///
/// ```
/// use tpt_zero_invariant::{Invariant, assert_invariant};
///
/// struct Counter { count: u32, max: u32 }
/// impl Invariant for Counter {
///     fn check(&self) -> bool { self.count <= self.max }
/// }
///
/// let c = Counter { count: 3, max: 10 };
/// let c = assert_invariant(&c);
/// assert_eq!(c.count, 3);
/// ```
pub fn assert_invariant<T: Invariant>(value: &T) -> &T {
    debug_assert!(
        Invariant::check(value),
        "invariant violated for {}",
        core::any::type_name::<T>()
    );
    value
}

/// Debug-asserts that `value` satisfies its [`Invariant`] and returns a
/// mutable reference to it.
///
/// Use this after mutating a state machine in place to confirm the
/// transition did not break well-formedness.
///
/// # Examples
///
/// ```
/// use tpt_zero_invariant::{Invariant, assert_invariant_mut};
///
/// struct Counter { count: u32, max: u32 }
/// impl Invariant for Counter {
///     fn check(&self) -> bool { self.count <= self.max }
/// }
///
/// let mut c = Counter { count: 3, max: 10 };
/// c.count += 1;
/// assert_invariant_mut(&mut c);
/// ```
pub fn assert_invariant_mut<T: Invariant>(value: &mut T) -> &mut T {
    debug_assert!(
        Invariant::check(value),
        "invariant violated for {}",
        core::any::type_name::<T>()
    );
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Bounded {
        value: i32,
        min: i32,
        max: i32,
    }

    impl Invariant for Bounded {
        fn check(&self) -> bool {
            self.value >= self.min && self.value <= self.max
        }
    }

    #[test]
    fn invariant_holds_in_valid_state() {
        let b = Bounded {
            value: 5,
            min: 0,
            max: 10,
        };
        assert!(b.check());
        let _ = assert_invariant(&b);
        let _ = assert_invariant_mut(&mut { b });
    }

    #[test]
    fn invariant_detects_violation() {
        let mut b = Bounded {
            value: 5,
            min: 0,
            max: 10,
        };
        // Manually break the invariant without going through check();
        // this documents that check returns false for an out-of-range value.
        b.value = 20;
        assert!(!b.check());
    }

    #[test]
    fn macro_returns_value_unchanged() {
        let b = Bounded {
            value: 1,
            min: 0,
            max: 10,
        };
        let returned = check_invariant!(b);
        assert_eq!(returned.value, 1);
    }

    #[test]
    fn invariant_macro_accepts_true_condition() {
        let n: u32 = 3;
        invariant!(n < 10);
        invariant!(n < 10, "n should be small");
    }
}
