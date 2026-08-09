#![doc = include_str!("../README.md")]

#![no_std]
#![warn(missing_docs)]
#![forbid(unsafe_code)]

/// The inclusive upper bound of the generated operation impls.
///
/// Values of `U<N>` with `N <= MAX_GENERATED` have pre-generated
/// [`Add`]/[`Sub`]/[`Mul`]/[`Min`]/[`Max`]/[`AssertLe`] impls. This bound
/// exists only because edition 2024 cannot compute `N + M` directly in a
/// const expression; it is not a semantic limit on the type itself.
pub const MAX_GENERATED: usize = 31;

/// A zero-sized type carrying an unsigned integer value `N` at the type
/// level, backed by a `const N: usize` generic parameter.
///
/// `U<N>` exists purely to let arithmetic be expressed in the type system:
/// the value `N` is recoverable via the `VALUE` of any operation trait.
///
/// # Examples
///
/// ```
/// use out_zero_type_level::U;
///
/// assert_eq!(U::<7>::VALUE, 7);
/// ```
pub struct U<const N: usize>;

impl<const N: usize> U<N> {
    /// The unsigned value `N` carried by this type, as a value-level `usize`.
    pub const VALUE: usize = N;
}

/// Adds two `usize` values at the value level (const context friendly).
///
/// # Examples
///
/// ```
/// use out_zero_type_level::add;
///
/// const R: usize = add(2, 3);
/// assert_eq!(R, 5);
/// ```
#[must_use]
pub const fn add(a: usize, b: usize) -> usize {
    a + b
}

/// Subtracts two `usize` values at the value level (const context friendly).
///
/// # Panics
///
/// Panics (at compile time in a `const` context, at runtime otherwise) on
/// underflow, i.e. when `a < b`.
///
/// # Examples
///
/// ```
/// use out_zero_type_level::sub;
///
/// const R: usize = sub(5, 3);
/// assert_eq!(R, 2);
/// ```
#[must_use]
pub const fn sub(a: usize, b: usize) -> usize {
    assert!(a >= b, "sub: underflow (a < b)");
    a - b
}

/// Multiplies two `usize` values at the value level (const context friendly).
///
/// # Examples
///
/// ```
/// use out_zero_type_level::mul;
///
/// const R: usize = mul(2, 3);
/// assert_eq!(R, 6);
/// ```
#[must_use]
pub const fn mul(a: usize, b: usize) -> usize {
    a * b
}

/// Returns the smaller of two `usize` values at the value level.
///
/// # Examples
///
/// ```
/// use out_zero_type_level::min;
///
/// const R: usize = min(2, 3);
/// assert_eq!(R, 2);
/// ```
#[must_use]
pub const fn min(a: usize, b: usize) -> usize {
    if a < b { a } else { b }
}

/// Returns the larger of two `usize` values at the value level.
///
/// # Examples
///
/// ```
/// use out_zero_type_level::max;
///
/// const R: usize = max(2, 3);
/// assert_eq!(R, 3);
/// ```
#[must_use]
pub const fn max(a: usize, b: usize) -> usize {
    if a > b { a } else { b }
}

/// Returns `true` when `a <= b`, evaluated at the value level (const context
/// friendly). This is the value-level counterpart used by [`AssertLe`].
///
/// # Examples
///
/// ```
/// use out_zero_type_level::is_le;
///
/// const OK: bool = is_le(2, 3);
/// assert!(OK);
/// assert!(!is_le(5, 3));
/// ```
#[must_use]
pub const fn is_le(a: usize, b: usize) -> bool {
    a <= b
}

/// Type-level addition: `Self + Rhs` evaluated at compile time.
///
/// The result is exposed both as the associated type [`Output`](Add::Output)
/// (another [`U`]) and as the associated const [`VALUE`](Add::VALUE). Impls
/// are generated for `0..=31`; see [`MAX_GENERATED`].
///
/// # Examples
///
/// ```
/// use out_zero_type_level::{U, Add};
///
/// type A = U<2>;
/// type B = U<3>;
/// type C = <A as Add<B>>::Output;
///
/// assert_eq!(<C as Add<U<0>>>::VALUE, 5);
/// ```
pub trait Add<Rhs = Self> {
    /// The [`U`] type carrying `N + M` at the type level.
    type Output;
    /// The numeric value `N + M` at the value level.
    const VALUE: usize;
}

/// Type-level subtraction: `Self - Rhs` evaluated at compile time.
///
/// Impls are generated only for `A >= B`; see [`MAX_GENERATED`].
///
/// # Examples
///
/// ```
/// use out_zero_type_level::{U, Sub};
///
/// type A = U<5>;
/// type B = U<3>;
/// type C = <A as Sub<B>>::Output;
///
/// assert_eq!(<C as Sub<U<0>>>::VALUE, 2);
/// ```
pub trait Sub<Rhs = Self> {
    /// The [`U`] type carrying `N - M` at the type level.
    type Output;
    /// The numeric value `N - M` at the value level.
    const VALUE: usize;
}

/// Type-level multiplication: `Self * Rhs` evaluated at compile time.
///
/// Impls are generated for `0..=31`; see [`MAX_GENERATED`].
///
/// # Examples
///
/// ```
/// use out_zero_type_level::{U, Mul};
///
/// type A = U<2>;
/// type B = U<3>;
/// type C = <A as Mul<B>>::Output;
///
/// assert_eq!(<C as Mul<U<1>>>::VALUE, 6);
/// ```
pub trait Mul<Rhs = Self> {
    /// The [`U`] type carrying `N * M` at the type level.
    type Output;
    /// The numeric value `N * M` at the value level.
    const VALUE: usize;
}

/// Type-level minimum: `min(Self, Rhs)` evaluated at compile time.
///
/// Impls are generated for `0..=31`; see [`MAX_GENERATED`].
///
/// # Examples
///
/// ```
/// use out_zero_type_level::{U, Min};
///
/// type A = U<2>;
/// type B = U<3>;
/// type C = <A as Min<B>>::Output;
///
/// assert_eq!(<C as Min<U<9>>>::VALUE, 2);
/// ```
pub trait Min<Rhs = Self> {
    /// The [`U`] type carrying `min(N, M)` at the type level.
    type Output;
    /// The numeric value `min(N, M)` at the value level.
    const VALUE: usize;
}

/// Type-level maximum: `max(Self, Rhs)` evaluated at compile time.
///
/// Impls are generated for `0..=31`; see [`MAX_GENERATED`].
///
/// # Examples
///
/// ```
/// use out_zero_type_level::{U, Max};
///
/// type A = U<2>;
/// type B = U<3>;
/// type C = <A as Max<B>>::Output;
///
/// assert_eq!(<C as Max<U<1>>>::VALUE, 3);
/// ```
pub trait Max<Rhs = Self> {
    /// The [`U`] type carrying `max(N, M)` at the type level.
    type Output;
    /// The numeric value `max(N, M)` at the value level.
    const VALUE: usize;
}

/// Convenience alias for the type-level sum `A + B`.
///
/// # Examples
///
/// ```
/// use out_zero_type_level::{U, Add, Sum};
///
/// type Seven = Sum<U<3>, U<4>>;
/// assert_eq!(<Seven as Add<U<0>>>::VALUE, 7);
/// ```
pub type Sum<A, B> = <A as Add<B>>::Output;

/// Convenience alias for the type-level difference `A - B`.
pub type Difference<A, B> = <A as Sub<B>>::Output;

/// Convenience alias for the type-level product `A * B`.
pub type Product<A, B> = <A as Mul<B>>::Output;

/// Convenience alias for the type-level minimum `min(A, B)`.
pub type Minimum<A, B> = <A as Min<B>>::Output;

/// Convenience alias for the type-level maximum `max(A, B)`.
pub type Maximum<A, B> = <A as Max<B>>::Output;

/// A boolean type constructor used to drive compile-time constraints.
///
/// `AssertConst<true>` implements [`IsTrue`]; `AssertConst<false>` does not.
/// Combined with the [`IsTrue`] marker trait, a bound of the form
/// `AssertConst<{ COND }>: IsTrue` fails to compile when `COND` is `false`,
/// turning a `const` condition into a build-time error.
pub struct AssertConst<const C: bool>;

/// Marker trait implemented only for [`AssertConst<true>`].
///
/// See [`AssertConst`] for how this drives compile-time constraints.
pub trait IsTrue {}

impl IsTrue for AssertConst<true> {}

/// A compile-time constraint guaranteeing `A <= B` for two `usize` values.
///
/// This trait is implemented *only* for pairs where `A <= B` (generated for
/// `0..=31`; see [`MAX_GENERATED`]). Requiring it for `A > B` is therefore a
/// compile error, not a runtime panic.
///
/// # Examples
///
/// ```
/// use out_zero_type_level::{U, AssertLe};
///
/// fn needs_le<const A: usize, const B: usize>() where U<A>: AssertLe<B> {}
///
/// needs_le::<2, 3>();
/// needs_le::<5, 5>();
/// ```
///
/// ```compile_fail
/// use out_zero_type_level::{U, AssertLe};
///
/// fn needs_le<const A: usize, const B: usize>() where U<A>: AssertLe<B> {}
///
/// // 5 is not <= 3.
/// needs_le::<5, 3>();
/// ```
pub trait AssertLe<const B: usize> {}

/// Generated operation impls for the range `0..=MAX_GENERATED`.
///
/// These impls exist because edition 2024 cannot compute `N + M` (etc.)
/// directly inside a `const` expression; the code in `generated.rs` is
/// produced by a small generator and is not intended to be hand-edited.
#[path = "generated.rs"]
pub mod generated;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_helpers() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(sub(5, 3), 2);
        assert_eq!(mul(2, 3), 6);
        assert_eq!(min(2, 3), 2);
        assert_eq!(max(2, 3), 3);
        assert!(is_le(2, 3));
        assert!(!is_le(5, 3));
    }

    #[test]
    fn type_level_add() {
        assert_eq!(<U<2> as Add<U<3>>>::VALUE, 5);
        assert_eq!(<<U<2> as Add<U<3>>>::Output as Add<U<10>>>::VALUE, 15);
    }

    #[test]
    fn type_level_sub() {
        assert_eq!(<U<5> as Sub<U<3>>>::VALUE, 2);
    }

    #[test]
    fn type_level_mul() {
        assert_eq!(<U<2> as Mul<U<3>>>::VALUE, 6);
    }

    #[test]
    fn type_level_min_max() {
        assert_eq!(<U<2> as Min<U<3>>>::VALUE, 2);
        assert_eq!(<U<2> as Max<U<3>>>::VALUE, 3);
        assert_eq!(<U<7> as Min<U<7>>>::VALUE, 7);
        assert_eq!(<U<7> as Max<U<7>>>::VALUE, 7);
    }

    #[test]
    fn aliases() {
        assert_eq!(<Sum<U<3>, U<4>> as Add<U<0>>>::VALUE, 7);
        assert_eq!(<Difference<U<9>, U<4>> as Sub<U<0>>>::VALUE, 5);
        assert_eq!(<Product<U<3>, U<4>> as Mul<U<1>>>::VALUE, 12);
        assert_eq!(<Minimum<U<3>, U<4>> as Min<U<9>>>::VALUE, 3);
        assert_eq!(<Maximum<U<3>, U<4>> as Max<U<1>>>::VALUE, 4);
    }

    #[test]
    fn assert_le_accepts_valid() {
        fn needs_le<const A: usize, const B: usize>() where U<A>: AssertLe<B> {}
        needs_le::<0, 1>();
        needs_le::<5, 5>();
    }

    #[test]
    #[should_panic(expected = "underflow")]
    fn sub_panics_on_underflow() {
        let _ = sub(3, 5);
    }
}
