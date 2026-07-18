//! A value paired with a proof that some property holds, for `no_std`
//! dependently-typed-style APIs. Part of the
//! [tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal)
//! ecosystem.
//!
//! [`Witness<T, P>`] carries a value `T` together with a (zero-sized) proof
//! `P` that some property of `T` holds. The proof is a *type-level* witness:
//! because it appears in the type, the compiler only lets you construct a
//! `Witness` when you can name the proof type `P`, which in practice means
//! you have run the construction logic that establishes the property.
//!
//! ```
//! use tpt_zero_witness::{Proof, Witness};
//!
//! /// Proof that a `u32` is non-zero.
//! #[derive(Clone, Copy, Debug)]
//! struct NonZero;
//! impl Proof for NonZero {}
//!
//! fn checked_nonzero(value: u32) -> Option<Witness<u32, NonZero>> {
//!     if value == 0 {
//!         None
//!     } else {
//!         Some(Witness::new(value))
//!     }
//! }
//!
//! let w = checked_nonzero(7).unwrap();
//! assert_eq!(*w.value(), 7);
//! ```
//!
//! Because `P` is zero-sized, a `Witness<T, P>` has the same size and
//! representation cost as `T` itself — the proof is erased at runtime.

#![no_std]
#![warn(missing_docs)]
#![forbid(unsafe_code)]

use core::marker::PhantomData;

/// A zero-sized marker trait implemented by proof types.
///
/// A type implementing `Proof` carries no data; it exists only as a
/// compile-time witness that some property holds. Implement it for the unit
/// struct (or any zero-sized type) that names the property you want to
/// track.
///
/// # Examples
///
/// ```
/// use tpt_zero_witness::Proof;
///
/// /// Witness that a value is even.
/// #[derive(Clone, Copy, Debug)]
/// struct IsEven;
/// impl Proof for IsEven {}
/// ```
pub trait Proof {}

/// A value `T` paired with a zero-sized proof `P` that some property holds.
///
/// `P` is stored as [`PhantomData`], so it occupies no space: `Witness<T, P>`
/// has the same size as `T`. The proof type participates only in the type
/// signature, guaranteeing that a `Witness` can only be constructed where
/// the proof type is in scope and has been meaningfully provided.
///
/// `T` is not required to be `Copy`; the `Witness` is `Copy` *only when* `T`
/// is, via the derived `Copy` impl's bounds.
///
/// # Examples
///
/// ```
/// use tpt_zero_witness::{Proof, Witness};
///
/// /// Proof that a `u32` is non-zero.
/// #[derive(Clone, Copy, Debug)]
/// struct NonZero;
/// impl Proof for NonZero {}
///
/// fn checked_nonzero(value: u32) -> Option<Witness<u32, NonZero>> {
///     if value == 0 {
///         None
///     } else {
///         Some(Witness::new(value))
///     }
/// }
///
/// let w = checked_nonzero(42).unwrap();
/// assert_eq!(*w.value(), 42);
/// let raw = w.into_inner();
/// assert_eq!(raw, 42);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Witness<T, P: Proof> {
    /// The carried value.
    value: T,
    /// Zero-sized marker pinning the proof type `P`.
    _proof: PhantomData<P>,
}

impl<T, P: Proof> Witness<T, P> {
    /// Constructs a `Witness` carrying `value` and the proof type `P`.
    ///
    /// Call this only after the property witnessed by `P` has been
    /// established for `value` (typically inside a constructor that checks
    /// the invariant and returns `None` or panics otherwise).
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_witness::{Proof, Witness};
    ///
    /// #[derive(Clone, Copy, Debug)]
    /// struct Always;
    /// impl Proof for Always {}
    ///
    /// let w: Witness<i32, Always> = Witness::new(3);
    /// assert_eq!(*w.value(), 3);
    /// ```
    #[must_use]
    pub const fn new(value: T) -> Self {
        Self {
            value,
            _proof: PhantomData,
        }
    }

    /// Returns a shared reference to the carried value.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_witness::{Proof, Witness};
    ///
    /// #[derive(Clone, Copy, Debug)]
    /// struct P;
    /// impl Proof for P {}
    ///
    /// let w = Witness::<&str, P>::new("hello");
    /// assert_eq!(w.value(), &"hello");
    /// ```
    #[must_use]
    pub const fn value(&self) -> &T {
        &self.value
    }

    /// Consumes the `Witness`, returning the carried value.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_witness::{Proof, Witness};
    ///
    /// #[derive(Clone, Copy, Debug)]
    /// struct P;
    /// impl Proof for P {}
    ///
    /// let w = Witness::<String, P>::new(String::from("x"));
    /// let s = w.into_inner();
    /// assert_eq!(s, "x");
    /// ```
    #[must_use]
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T, P: Proof> Witness<T, P> {
    /// Constructs a `Witness` from a value that already carries an explicit
    /// proof marker value of type `P`.
    ///
    /// Unlike [`Witness::new`], which relies only on the type `P`, this
    /// accepts a *value* of the proof type (which must itself be zero-sized)
    /// for APIs that thread a witness value through. The value is consumed
    /// and discarded; only its type matters.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_witness::{Proof, Witness};
    ///
    /// #[derive(Clone, Copy, Debug)]
    /// struct Positive;
    /// impl Proof for Positive {}
    ///
    /// let w: Witness<u64, Positive> = Witness::from_proof(10u64, Positive);
    /// assert_eq!(*w.value(), 10);
    /// ```
    #[must_use]
    pub fn from_proof(value: T, _proof: P) -> Self {
        Self {
            value,
            _proof: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug)]
    struct NonZero;
    impl Proof for NonZero {}

    #[derive(Clone, Copy, Debug)]
    struct Even;
    impl Proof for Even {}

    /// A non-`Copy` value type usable in `no_std` (no `Drop` impl, so it
    /// stays trivially droppable without `alloc`).
    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Bag {
        items: [u32; 2],
    }

    #[test]
    fn new_and_value() {
        let w = Witness::<u32, NonZero>::new(5);
        assert_eq!(*w.value(), 5);
    }

    #[test]
    fn into_inner_returns_value() {
        let w = Witness::<&str, NonZero>::new("abc");
        assert_eq!(w.into_inner(), "abc");
    }

    #[test]
    fn from_proof_consumes_proof_value() {
        let w = Witness::<i64, Even>::from_proof(4, Even);
        assert_eq!(*w.value(), 4);
    }

    #[test]
    fn copy_when_t_is_copy() {
        let w = Witness::<u8, NonZero>::new(1);
        let w2 = w;
        assert_eq!(*w.value(), 1);
        assert_eq!(*w2.value(), 1);
    }

    #[test]
    fn non_copy_t_works() {
        let w = Witness::<Bag, NonZero>::new(Bag { items: [1, 2] });
        let w2 = w.clone();
        assert_eq!(w.into_inner(), Bag { items: [1, 2] });
        assert_eq!(w2.into_inner(), Bag { items: [1, 2] });
    }
}
