#![doc = include_str!("../README.md")]

#![no_std]
#![warn(missing_docs)]
#![forbid(unsafe_code)]

extern crate tpt_zero_invariant;

/// A property that holds on entry to a loop and is preserved by every
/// iteration.
///
/// Splitting the invariant into [`LoopInvariant::establish`] (true before the
/// first iteration) and [`LoopInvariant::maintain`] (true after every
/// iteration) lets callers verify the two halves independently â€” the entry
/// condition can be checked once, and the per-iteration condition checked on
/// every pass through the loop body.
///
/// # Examples
///
/// ```
/// use out_zero_loop_inv::LoopInvariant;
///
/// struct SumLoop { i: u64, sum: u64 }
///
/// impl LoopInvariant for SumLoop {
///     fn establish(&self) -> bool { self.sum == self.i * (self.i + 1) / 2 }
///     fn maintain(&self) -> bool { self.sum == self.i * (self.i + 1) / 2 }
/// }
/// ```
pub trait LoopInvariant {
    /// Returns `true` when the invariant holds on entry to the loop (before
    /// the first iteration runs).
    ///
    /// This must be a pure, side-effect-free predicate: given the same state
    /// it always returns the same answer, and it must never panic.
    fn establish(&self) -> bool;

    /// Returns `true` when the invariant holds at the top of every iteration,
    /// including the first.
    ///
    /// This must be a pure, side-effect-free predicate: given the same state
    /// it always returns the same answer, and it must never panic.
    fn maintain(&self) -> bool;
}

/// Asserts a boolean loop-invariant expression holds on the current
/// iteration, using [`core::debug_assert!`] so the check is compiled away in
/// release builds.
///
/// Place it at the top of the loop body (after the loop variable is
/// introduced) to document and guard the invariant without paying a runtime
/// cost in optimized builds.
///
/// # Examples
///
/// ```
/// use out_zero_loop_inv::loop_invariant;
///
/// let mut sum: u64 = 0;
/// for i in 0..10u64 {
///     loop_invariant!(sum == i * (i + 1) / 2);
///     sum += i + 1;
/// }
/// assert_eq!(sum, 10 * 11 / 2);
/// ```
///
/// [`core::debug_assert!`]: https://doc.rust-lang.org/core/macro.debug_assert.html
#[macro_export]
macro_rules! loop_invariant {
    ($cond:expr $(,)?) => {
        ::core::debug_assert!($cond);
    };
    ($cond:expr, $($msg:tt)+) => {
        ::core::debug_assert!($cond, $($msg)+);
    };
}

/// Checks that `value` satisfies [`LoopInvariant::maintain`], using
/// [`core::debug_assert!`] so the check is compiled away in release builds.
///
/// Returns `value` unchanged so it can be threaded through a loop body while
/// documenting the invariant at the call site. Use [`check_loop_entry!`] once
/// before the loop to verify [`LoopInvariant::establish`].
///
/// # Examples
///
/// ```
/// use out_zero_loop_inv::{LoopInvariant, check_loop_invariant};
///
/// struct SumLoop { i: u64, sum: u64 }
/// impl LoopInvariant for SumLoop {
///     fn establish(&self) -> bool { self.sum == self.i * (self.i + 1) / 2 }
///     fn maintain(&self) -> bool { self.sum == self.i * (self.i + 1) / 2 }
/// }
///
/// let s = SumLoop { i: 0, sum: 0 };
/// let s = check_loop_invariant!(s);
/// assert_eq!(s.sum, 0);
/// ```
///
/// [`core::debug_assert!`]: https://doc.rust-lang.org/core/macro.debug_assert.html
#[macro_export]
macro_rules! check_loop_invariant {
    ($value:expr $(,)?) => {{
        let __inv_value = $value;
        ::core::debug_assert!(
            $crate::LoopInvariant::maintain(&__inv_value),
            "loop invariant violated: {}",
            ::core::stringify!($value)
        );
        __inv_value
    }};
    ($value:expr, $($msg:tt)+) => {{
        let __inv_value = $value;
        ::core::debug_assert!($crate::LoopInvariant::maintain(&__inv_value), $($msg)+);
        __inv_value
    }};
}

/// Checks that `value` satisfies [`LoopInvariant::establish`], using
/// [`core::debug_assert!`] so the check is compiled away in release builds.
///
/// Call this once, before entering the loop, to verify the invariant holds on
/// entry; use [`check_loop_invariant!`] inside the body to verify it is
/// maintained on every iteration.
///
/// # Examples
///
/// ```
/// use out_zero_loop_inv::{LoopInvariant, check_loop_entry};
///
/// struct SumLoop { i: u64, sum: u64 }
/// impl LoopInvariant for SumLoop {
///     fn establish(&self) -> bool { self.sum == self.i * (self.i + 1) / 2 }
///     fn maintain(&self) -> bool { self.sum == self.i * (self.i + 1) / 2 }
/// }
///
/// let s = SumLoop { i: 0, sum: 0 };
/// let s = check_loop_entry!(s);
/// assert_eq!(s.sum, 0);
/// ```
///
/// [`core::debug_assert!`]: https://doc.rust-lang.org/core/macro.debug_assert.html
#[macro_export]
macro_rules! check_loop_entry {
    ($value:expr $(,)?) => {{
        let __inv_value = $value;
        ::core::debug_assert!(
            $crate::LoopInvariant::establish(&__inv_value),
            "loop invariant not established on entry: {}",
            ::core::stringify!($value)
        );
        __inv_value
    }};
    ($value:expr, $($msg:tt)+) => {{
        let __inv_value = $value;
        ::core::debug_assert!($crate::LoopInvariant::establish(&__inv_value), $($msg)+);
        __inv_value
    }};
}

/// Debug-asserts that `value` satisfies [`LoopInvariant::establish`] and
/// returns a shared reference to it.
///
/// This is the borrowed counterpart to the [`check_loop_entry!`] macro: use it
/// once, before the loop, when you already hold a reference and want to verify
/// the entry condition without moving the value.
///
/// # Examples
///
/// ```
/// use out_zero_loop_inv::{LoopInvariant, assert_loop_entry};
///
/// struct SumLoop { i: u64, sum: u64 }
/// impl LoopInvariant for SumLoop {
///     fn establish(&self) -> bool { self.sum == self.i * (self.i + 1) / 2 }
///     fn maintain(&self) -> bool { self.sum == self.i * (self.i + 1) / 2 }
/// }
///
/// let s = SumLoop { i: 0, sum: 0 };
/// let s = assert_loop_entry(&s);
/// assert_eq!(s.sum, 0);
/// ```
pub fn assert_loop_entry<T: LoopInvariant>(value: &T) -> &T {
    debug_assert!(
        LoopInvariant::establish(value),
        "loop invariant not established on entry for {}",
        core::any::type_name::<T>()
    );
    value
}

/// Debug-asserts that `value` satisfies [`LoopInvariant::maintain`] and
/// returns a shared reference to it.
///
/// This is the borrowed counterpart to the [`check_loop_invariant!`] macro:
/// use it inside the loop body when you already hold a reference and want to
/// verify the invariant is preserved without moving the value.
///
/// # Examples
///
/// ```
/// use out_zero_loop_inv::{LoopInvariant, assert_loop_invariant};
///
/// struct SumLoop { i: u64, sum: u64 }
/// impl LoopInvariant for SumLoop {
///     fn establish(&self) -> bool { self.sum == self.i * (self.i + 1) / 2 }
///     fn maintain(&self) -> bool { self.sum == self.i * (self.i + 1) / 2 }
/// }
///
/// let s = SumLoop { i: 0, sum: 0 };
/// let s = assert_loop_invariant(&s);
/// assert_eq!(s.sum, 0);
/// ```
pub fn assert_loop_invariant<T: LoopInvariant>(value: &T) -> &T {
    debug_assert!(
        LoopInvariant::maintain(value),
        "loop invariant violated for {}",
        core::any::type_name::<T>()
    );
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SumLoop {
        i: u64,
        sum: u64,
    }

    impl LoopInvariant for SumLoop {
        fn establish(&self) -> bool {
            self.sum == self.i * (self.i + 1) / 2
        }
        fn maintain(&self) -> bool {
            self.sum == self.i * (self.i + 1) / 2
        }
    }

    #[test]
    fn sum_loop_macro_maintains_invariant() {
        let mut sum: u64 = 0;
        for i in 0..100u64 {
            loop_invariant!(sum == i * (i + 1) / 2);
            sum += i + 1;
        }
        assert_eq!(sum, 100 * 101 / 2);
    }

    #[test]
    fn sum_loop_maintains_invariant_with_message() {
        let mut sum: u64 = 0;
        for i in 0..100u64 {
            loop_invariant!(sum == i * (i + 1) / 2, "broken at i = {}", i);
            sum += i + 1;
        }
        assert_eq!(sum, 100 * 101 / 2);
    }

    #[test]
    fn trait_entry_and_maintain_checked_each_iteration() {
        let mut s = SumLoop { i: 0, sum: 0 };
        assert_loop_entry(&s);
        while s.i < 100 {
            assert_loop_invariant(&s);
            s.sum += s.i + 1;
            s.i += 1;
        }
        assert_eq!(s.sum, 100 * 101 / 2);
    }

    #[test]
    fn macro_returns_value_unchanged() {
        let s = SumLoop { i: 0, sum: 0 };
        let entry = check_loop_entry!(s);
        let maintained = check_loop_invariant!(entry);
        assert_eq!(maintained.sum, 0);
    }

    #[test]
    fn fn_helpers_return_reference_unchanged() {
        let s = SumLoop { i: 0, sum: 0 };
        let entry = assert_loop_entry(&s);
        let maintained = assert_loop_invariant(entry);
        assert_eq!(maintained.sum, 0);
    }

    #[test]
    fn establish_detects_broken_entry() {
        let s = SumLoop { i: 5, sum: 0 };
        assert!(!s.establish());
    }
}

#[cfg(test)]
mod proptest_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn sum_loop_invariant_holds_for_any_n(n in 0u64..10_000) {
            let mut sum: u64 = 0;
            for i in 0..n {
                prop_assert!(sum == i * (i + 1) / 2);
                sum += i + 1;
            }
            prop_assert_eq!(sum, n * (n + 1) / 2);
        }
    }
}
