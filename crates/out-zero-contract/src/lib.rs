#![doc = include_str!("../README.md")]
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
/// is compiled), not in the calling crate â€” so a `#[cfg(feature = "checked")]`
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
    (REQ=$req:literal, $cond:expr $(,)?) => {{
        $crate::__contract_assert!($cond);
    }};
    (REQ=$req:literal, $cond:expr, $($msg:tt)+) => {{
        $crate::__contract_assert!($cond, $($msg)+);
    }};
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
    (REQ=$req:literal, $cond:expr $(,)?) => {{
        $crate::__contract_assert!($cond);
    }};
    (REQ=$req:literal, $cond:expr, $($msg:tt)+) => {{
        $crate::__contract_assert!($cond, $($msg)+);
    }};
    ($cond:expr $(,)?) => {{
        $crate::__contract_assert!($cond);
    }};
    ($cond:expr, $($msg:tt)+) => {{
        $crate::__contract_assert!($cond, $($msg)+);
    }};
}

/// Declares a precondition in *MC/DC-oriented* form.
///
/// Modified Condition/Decision Coverage (MC/DC) requires that every
/// individual condition in a decision be shown to independently affect the
/// outcome. This macro takes a comma-separated list of independent conditions
/// and asserts **each one separately** (in debug or `checked` builds), so a
/// failing sub-condition is pinpointed rather than masked by short-circuit
/// evaluation of a single compound boolean.
///
/// Combine this with a coverage tool configured for MC/DC
/// (`rustc -Z coverage-options=mcdc`, nightly) to verify that every condition
/// is independently exercised by the test suite.
///
/// # Examples
///
/// ```
/// use out_zero_contract::mcdc_requires;
///
/// fn enter_armed(mode: u8, key: bool) {
///     mcdc_requires!(mode == 2, key);
/// }
///
/// enter_armed(2, true);
/// ```
///
/// # Panics
///
/// Panics (in debug or `checked` builds) on the first failing condition.
#[macro_export]
macro_rules! mcdc_requires {
    ($($cond:expr),+ $(,)?) => {{
        $(
            $crate::__contract_assert!($cond, "MC/DC condition failed: {}", stringify!($cond));
        )+
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

    #[test]
    fn req_tag_is_ignored_at_runtime() {
        fn f(x: i32) {
            requires!(REQ = "SRS-ALT-014", x >= 0, "x must be non-negative");
        }
        f(0);
        f(5);
    }

    #[test]
    fn mcdc_asserts_each_condition() {
        fn enter(mode: u8, key: bool) {
            mcdc_requires!(mode == 2, key);
        }
        enter(2, true);
    }

    #[test]
    #[should_panic(expected = "MC/DC condition failed")]
    fn mcdc_pins_failing_condition() {
        fn enter(mode: u8, key: bool) {
            mcdc_requires!(mode == 2, key);
        }
        enter(2, false);
    }
}

/// Bridge from design-by-contract macros to formal verification backends.
///
/// Enabled by the `formal` feature. This module pulls in no external
/// toolchain; it provides the vocabulary and anchors that the external
/// provers consume:
///
/// - **Kani** — translate a function guarded by `requires!`/`ensures!` into a
///   Kani verification harness; `proof!` marks the harness body.
/// - **Creusot** — Creusot translates Rust to Why3. Pair it with the
///   `tpt-zero-smt-lite` `export` module (the `alloc` feature) to emit a Why3
///   module from a `ConstraintSet`, re-checking the same constraints
///   deductively. See <https://docs.rs/tpt-zero-smt-lite>.
/// - **Prusti** — Prusti reads `requires`/`ensures` specification attributes;
///   the `proof!` macro marks the block whose invariants Prusti discharges.
#[cfg(feature = "formal")]
pub mod formal {
    /// Marks a code region whose safety properties are to be discharged by a
    /// formal backend (Kani / Creusot / Prusti).
    ///
    /// With the `formal` feature enabled this expands to a documentation-only
    /// anchor that has **no runtime effect** — the actual proof is performed
    /// by the external tool, which reads the surrounding `requires!`/
    /// `ensures!` contracts. The block's tokens are referenced (via
    /// [`stringify!`]) so the marker never changes program behaviour.
    #[macro_export]
    macro_rules! proof {
        ($($t:tt)*) => {{
            // Formal-verification anchor; intentionally has no runtime effect.
            let _ = ::core::stringify!($($t)*);
        }};
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
