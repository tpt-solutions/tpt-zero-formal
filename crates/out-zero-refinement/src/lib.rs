//! Refinement types for `no_std`: a value of type `T` that is known to
//! satisfy a compile-time-named predicate `P`. Part of the
//! [tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal)
//! ecosystem.
//!
//! A [`Refined<T, P>`] wraps a value together with a *type-level* promise
//! that the [`Predicate`] `P` holds for that value. The only way to build one
//! is through [`Refined::new`] (or the [`TryFrom`] impl), which runs
//! [`Predicate::check`]; construction fails with a [`RefinementError`] when
//! the predicate is not satisfied. Because `P` is zero-sized, a
//! `Refined<T, P>` has the same size and representation cost as `T` itself.
//!
//! This complements [`tpt_zero_witness::Witness`]: a `Witness` pairs a value
//! with an *externally established* proof, whereas a `Refined` establishes the
//! proof itself by running a decidable predicate at construction time. Use
//! [`Refined::into_witness`] to bridge from the former to the latter.
//!
//! ```
//! use out_zero_refinement::{Predicate, Refined};
//!
//! /// Predicate: a `u32` that is non-zero.
//! struct NonZeroU32;
//! impl Predicate<u32> for NonZeroU32 {
//!     fn check(value: &u32) -> bool {
//!         *value != 0
//!     }
//! }
//!
//! let refined = Refined::<u32, NonZeroU32>::new(7).unwrap();
//! assert_eq!(*refined.get(), 7);
//! assert_eq!(*refined, 7); // via `Deref`
//! assert_eq!(refined.into_inner(), 7);
//!
//! assert!(Refined::<u32, NonZeroU32>::new(0).is_err());
//! ```

#![no_std]
#![warn(missing_docs)]
#![forbid(unsafe_code)]

#[cfg(feature = "std")]
extern crate std;

use core::fmt;
use core::marker::PhantomData;
use core::ops::Deref;

use tpt_zero_witness::{Proof, Witness};

/// A decidable predicate over values of type `T`.
///
/// Implement this for a zero-sized marker type that *names* the property you
/// want to refine on. The check is an associated function, so no instance of
/// the predicate type is ever needed: the type itself is the predicate.
///
/// # Examples
///
/// ```
/// use out_zero_refinement::Predicate;
///
/// /// A predicate that a `i64` is strictly positive.
/// struct PositiveI64;
/// impl Predicate<i64> for PositiveI64 {
///     fn check(value: &i64) -> bool {
///         *value > 0
///     }
/// }
///
/// assert!(PositiveI64::check(&1));
/// assert!(!PositiveI64::check(&0));
/// ```
pub trait Predicate<T> {
    /// Returns `true` if and only if `value` satisfies the predicate.
    ///
    /// This must be a pure, deterministic function of `value`: the whole
    /// point of [`Refined`] is that a value which passed the check once will
    /// still pass it for the lifetime of the wrapper.
    fn check(value: &T) -> bool;
}

/// The error returned when a value fails a [`Predicate`] during construction
/// of a [`Refined`].
///
/// The original value is returned so the caller can recover it without a
/// clone.
///
/// # Examples
///
/// ```
/// use out_zero_refinement::{Predicate, Refined, RefinementError};
///
/// struct NonZeroU32;
/// impl Predicate<u32> for NonZeroU32 {
///     fn check(value: &u32) -> bool {
///         *value != 0
///     }
/// }
///
/// let err: RefinementError<u32> = Refined::<u32, NonZeroU32>::new(0).unwrap_err();
/// assert_eq!(err.into_value(), 0);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RefinementError<T> {
    /// The value that failed the predicate, returned to the caller.
    value: T,
}

impl<T> RefinementError<T> {
    /// Returns a shared reference to the rejected value.
    ///
    /// # Examples
    ///
    /// ```
    /// use out_zero_refinement::{Predicate, Refined};
    ///
    /// struct NonZeroU32;
    /// impl Predicate<u32> for NonZeroU32 {
    ///     fn check(value: &u32) -> bool {
    ///         *value != 0
    ///     }
    /// }
    ///
    /// let err = Refined::<u32, NonZeroU32>::new(0).unwrap_err();
    /// assert_eq!(err.value(), &0);
    /// ```
    #[must_use]
    pub const fn value(&self) -> &T {
        &self.value
    }

    /// Consumes the error, returning the rejected value.
    ///
    /// # Examples
    ///
    /// ```
    /// use out_zero_refinement::{Predicate, Refined};
    ///
    /// struct NonZeroU32;
    /// impl Predicate<u32> for NonZeroU32 {
    ///     fn check(value: &u32) -> bool {
    ///         *value != 0
    ///     }
    /// }
    ///
    /// let err = Refined::<u32, NonZeroU32>::new(0).unwrap_err();
    /// assert_eq!(err.into_value(), 0);
    /// ```
    #[must_use]
    pub fn into_value(self) -> T {
        self.value
    }
}

impl<T> fmt::Debug for RefinementError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RefinementError").finish_non_exhaustive()
    }
}

impl<T> fmt::Display for RefinementError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("value did not satisfy its refinement predicate")
    }
}

#[cfg(feature = "std")]
impl<T> std::error::Error for RefinementError<T> {}

/// A value `T` refined by a [`Predicate`] `P`: it is guaranteed that
/// `P::check` returned `true` for the wrapped value at construction time.
///
/// `P` is stored as [`PhantomData`], so it occupies no space:
/// `Refined<T, P>` has the same size as `T`. The predicate type participates
/// only in the type signature, so a `Refined` can only be *named* — and thus
/// only be built or accepted — where the predicate type is in scope.
///
/// The wrapper deliberately exposes only shared access to its contents
/// ([`get`](Self::get), [`Deref`], [`AsRef`]) plus [`into_inner`](Self::into_inner) to move
/// the value back out. There is no `&mut` accessor, because mutation could
/// break the refinement invariant.
///
/// # Examples
///
/// ```
/// use out_zero_refinement::{Predicate, Refined};
///
/// struct EvenU8;
/// impl Predicate<u8> for EvenU8 {
///     fn check(value: &u8) -> bool {
///         value % 2 == 0
///     }
/// }
///
/// let refined = Refined::<u8, EvenU8>::new(4).unwrap();
/// assert_eq!(*refined, 4);
/// assert!(Refined::<u8, EvenU8>::new(3).is_err());
/// ```
pub struct Refined<T, P: Predicate<T>> {
    /// The wrapped value, proven to satisfy `P`.
    inner: T,
    /// Zero-sized marker pinning the predicate type `P`.
    _p: PhantomData<P>,
}

impl<T, P: Predicate<T>> Refined<T, P> {
    /// Constructs a `Refined` if `value` satisfies the predicate `P`.
    ///
    /// # Errors
    ///
    /// Returns a [`RefinementError`] carrying `value` unchanged when
    /// `P::check(&value)` returns `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// use out_zero_refinement::{Predicate, Refined};
    ///
    /// struct NonZeroU32;
    /// impl Predicate<u32> for NonZeroU32 {
    ///     fn check(value: &u32) -> bool {
    ///         *value != 0
    ///     }
    /// }
    ///
    /// assert!(Refined::<u32, NonZeroU32>::new(1).is_ok());
    /// assert!(Refined::<u32, NonZeroU32>::new(0).is_err());
    /// ```
    pub fn new(value: T) -> Result<Self, RefinementError<T>> {
        if P::check(&value) {
            Ok(Self {
                inner: value,
                _p: PhantomData,
            })
        } else {
            Err(RefinementError { value })
        }
    }

    /// Returns a shared reference to the refined value.
    ///
    /// # Examples
    ///
    /// ```
    /// use out_zero_refinement::{Predicate, Refined};
    ///
    /// struct NonZeroU32;
    /// impl Predicate<u32> for NonZeroU32 {
    ///     fn check(value: &u32) -> bool {
    ///         *value != 0
    ///     }
    /// }
    ///
    /// let refined = Refined::<u32, NonZeroU32>::new(9).unwrap();
    /// assert_eq!(refined.get(), &9);
    /// ```
    #[must_use]
    pub const fn get(&self) -> &T {
        &self.inner
    }

    /// Consumes the `Refined`, returning the wrapped value.
    ///
    /// # Examples
    ///
    /// ```
    /// use out_zero_refinement::{Predicate, Refined};
    ///
    /// struct NonZeroU32;
    /// impl Predicate<u32> for NonZeroU32 {
    ///     fn check(value: &u32) -> bool {
    ///         *value != 0
    ///     }
    /// }
    ///
    /// let refined = Refined::<u32, NonZeroU32>::new(9).unwrap();
    /// assert_eq!(refined.into_inner(), 9);
    /// ```
    #[must_use]
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Converts this refinement into a [`Witness`] carrying the same value,
    /// tagged with a proof that the predicate `P` held.
    ///
    /// This is the sound bridge from decidable refinements to the more
    /// general witness API. Because a `Refined<T, P>` can only exist if
    /// `P::check` returned `true`, it may hand out a
    /// `Witness<T, RefinedProof<P>>` — a proof type that is *only*
    /// constructible through this method, so no caller can forge it for an
    /// unvalidated value.
    ///
    /// # Examples
    ///
    /// ```
    /// use out_zero_refinement::{Predicate, Refined};
    ///
    /// struct NonZeroU32;
    /// impl Predicate<u32> for NonZeroU32 {
    ///     fn check(value: &u32) -> bool {
    ///         *value != 0
    ///     }
    /// }
    ///
    /// let refined = Refined::<u32, NonZeroU32>::new(9).unwrap();
    /// let witness = refined.into_witness();
    /// assert_eq!(*witness.value(), 9);
    /// ```
    #[must_use]
    pub fn into_witness(self) -> Witness<T, RefinedProof<P>> {
        Witness::from_proof(self.inner, RefinedProof(PhantomData))
    }
}

impl<T, P: Predicate<T>> Deref for Refined<T, P> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T, P: Predicate<T>> AsRef<T> for Refined<T, P> {
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

impl<T, P: Predicate<T>> From<Refined<T, P>> for RefinedInner<T> {
    fn from(refined: Refined<T, P>) -> Self {
        RefinedInner(refined.inner)
    }
}

/// A [`Proof`](tpt_zero_witness::Proof) that a value satisfied the predicate
/// `P`.
///
/// This is constructed *only* by [`Refined::into_witness`], which is itself
/// reachable only from a `Refined` that already passed `P::check`. Because
/// the type is local to this crate and has no public constructor other than
/// through a validated `Refined`, no caller can mint a
/// `Witness<T, RefinedProof<P>>` for an unvalidated value.
pub struct RefinedProof<P>(PhantomData<P>);

impl<P> Proof for RefinedProof<P> {}

/// A transparent newtype wrapping the inner value moved out of a [`Refined`].
///
/// This exists so a blanket `From<Refined<T, P>>` conversion can be offered
/// without conflicting with the reflexive `From<T> for T` blanket impl in
/// `core`. Call [`RefinedInner::into_inner`] to unwrap it.
///
/// # Examples
///
/// ```
/// use out_zero_refinement::{Predicate, Refined, RefinedInner};
///
/// struct NonZeroU32;
/// impl Predicate<u32> for NonZeroU32 {
///     fn check(value: &u32) -> bool {
///         *value != 0
///     }
/// }
///
/// let refined = Refined::<u32, NonZeroU32>::new(5).unwrap();
/// let raw: RefinedInner<u32> = refined.into();
/// assert_eq!(raw.into_inner(), 5);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RefinedInner<T>(T);

impl<T> RefinedInner<T> {
    /// Consumes the wrapper, returning the inner value.
    #[must_use]
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: Clone, P: Predicate<T>> Clone for Refined<T, P> {
    fn clone(&self) -> Self {
        // Cloning preserves the value, so the predicate still holds; no need
        // to re-check.
        Self {
            inner: self.inner.clone(),
            _p: PhantomData,
        }
    }
}

impl<T: Copy, P: Predicate<T>> Copy for Refined<T, P> {}

impl<T: fmt::Debug, P: Predicate<T>> fmt::Debug for Refined<T, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Refined").field(&self.inner).finish()
    }
}

impl<T: PartialEq, P: Predicate<T>> PartialEq for Refined<T, P> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T: Eq, P: Predicate<T>> Eq for Refined<T, P> {}

impl<T: PartialOrd, P: Predicate<T>> PartialOrd for Refined<T, P> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}

impl<T: Ord, P: Predicate<T>> Ord for Refined<T, P> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl<T: core::hash::Hash, P: Predicate<T>> core::hash::Hash for Refined<T, P> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

/// Example predicate: a `u32` that is non-zero.
///
/// # Examples
///
/// ```
/// use out_zero_refinement::{NonZeroU32, Refined};
///
/// let refined = Refined::<u32, NonZeroU32>::new(42).unwrap();
/// assert_eq!(*refined, 42);
/// assert!(Refined::<u32, NonZeroU32>::new(0).is_err());
/// ```
#[derive(Clone, Copy, Debug)]
pub struct NonZeroU32;

impl Predicate<u32> for NonZeroU32 {
    fn check(value: &u32) -> bool {
        *value != 0
    }
}

impl TryFrom<u32> for Refined<u32, NonZeroU32> {
    type Error = RefinementError<u32>;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// Example predicate: an `i64` that is strictly positive.
///
/// # Examples
///
/// ```
/// use out_zero_refinement::{PositiveI64, Refined};
///
/// let refined = Refined::<i64, PositiveI64>::new(1).unwrap();
/// assert_eq!(*refined, 1);
/// assert!(Refined::<i64, PositiveI64>::new(0).is_err());
/// assert!(Refined::<i64, PositiveI64>::new(-5).is_err());
/// ```
#[derive(Clone, Copy, Debug)]
pub struct PositiveI64;

impl Predicate<i64> for PositiveI64 {
    fn check(value: &i64) -> bool {
        *value > 0
    }
}

impl TryFrom<i64> for Refined<i64, PositiveI64> {
    type Error = RefinementError<i64>;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EvenU8;
    impl Predicate<u8> for EvenU8 {
        fn check(value: &u8) -> bool {
            value % 2 == 0
        }
    }

    #[test]
    fn new_accepts_satisfying_value() {
        let r = Refined::<u32, NonZeroU32>::new(3).unwrap();
        assert_eq!(*r.get(), 3);
    }

    #[test]
    fn new_rejects_and_returns_value() {
        let err = Refined::<u32, NonZeroU32>::new(0).unwrap_err();
        assert_eq!(*err.value(), 0);
        assert_eq!(err.into_value(), 0);
    }

    #[test]
    fn positive_i64_predicate() {
        assert!(Refined::<i64, PositiveI64>::new(10).is_ok());
        assert!(Refined::<i64, PositiveI64>::new(0).is_err());
        assert!(Refined::<i64, PositiveI64>::new(-1).is_err());
    }

    #[test]
    fn deref_and_as_ref() {
        let r = Refined::<u8, EvenU8>::new(8).unwrap();
        assert_eq!(*r, 8);
        assert_eq!(r.as_ref(), &8);
        // Deref lets us call `T` methods directly.
        assert_eq!(r.count_ones(), 1);
    }

    #[test]
    fn into_inner_moves_value() {
        let r = Refined::<u8, EvenU8>::new(6).unwrap();
        assert_eq!(r.into_inner(), 6);
    }

    #[test]
    fn try_from_matches_new() {
        let r: Result<Refined<u32, NonZeroU32>, _> = 5u32.try_into();
        assert!(r.is_ok());
        let bad: Result<Refined<u32, NonZeroU32>, _> = 0u32.try_into();
        assert!(bad.is_err());
    }

    #[test]
    fn from_refined_into_inner_wrapper() {
        let r = Refined::<u32, NonZeroU32>::new(11).unwrap();
        let raw: RefinedInner<u32> = r.into();
        assert_eq!(raw.into_inner(), 11);
    }

    #[test]
    fn into_witness_bridges_to_witness() {
        let r = Refined::<u32, NonZeroU32>::new(4).unwrap();
        let w = r.into_witness();
        assert_eq!(*w.value(), 4);
    }

    #[test]
    fn derived_traits() {
        let a = Refined::<u8, EvenU8>::new(2).unwrap();
        let b = a; // Copy
        assert_eq!(a, b);
        assert!(a <= b);
        // Debug does not panic.
        let _ = alloc_debug(&a);
    }

    /// A `Clone`-but-not-`Copy` value type usable without `alloc`.
    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Pair([u8; 2]);

    struct SumIsEven;
    impl Predicate<Pair> for SumIsEven {
        fn check(value: &Pair) -> bool {
            (value.0[0].wrapping_add(value.0[1])) % 2 == 0
        }
    }

    #[test]
    fn clone_preserves_refinement() {
        let a = Refined::<Pair, SumIsEven>::new(Pair([2, 4])).unwrap();
        let b = a.clone();
        assert_eq!(a, b);
        assert_eq!(b.into_inner(), Pair([2, 4]));
    }

    fn alloc_debug<T: fmt::Debug>(value: &T) -> bool {
        // Use the formatter in a `no_std`/no-alloc friendly way via a
        // counting writer.
        use core::fmt::Write;

        struct Counter(usize);
        impl Write for Counter {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                self.0 += s.len();
                Ok(())
            }
        }
        let mut c = Counter(0);
        write!(c, "{value:?}").is_ok() && c.0 > 0
    }

    #[test]
    fn error_display_and_debug_hide_value() {
        use core::fmt::Write;

        struct Buf {
            data: [u8; 128],
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

        let err = Refined::<u32, NonZeroU32>::new(0).unwrap_err();
        let mut buf = Buf {
            data: [0; 128],
            len: 0,
        };
        write!(buf, "{err}").unwrap();
        let shown = core::str::from_utf8(&buf.data[..buf.len]).unwrap();
        assert_eq!(shown, "value did not satisfy its refinement predicate");

        let mut dbg_buf = Buf {
            data: [0; 128],
            len: 0,
        };
        write!(dbg_buf, "{err:?}").unwrap();
        let dbg = core::str::from_utf8(&dbg_buf.data[..dbg_buf.len]).unwrap();
        assert!(dbg.contains("RefinementError"));
    }
}
