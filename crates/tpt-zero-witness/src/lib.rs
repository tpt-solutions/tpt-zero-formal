//! A value paired with a proof that some property holds, for `no_std`
//! dependently-typed-style APIs. Part of the
//! [tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal)
//! ecosystem.
//!
//! [`Witness<T, P>`] carries a value `T` together with a (zero-sized) proof
//! `P` that some property of `T` holds. The proof is a *type-level* witness:
//! because it appears in the type, the compiler only lets you construct a
//! `Witness` when you can supply a *value* of the proof type `P` — and the
//! only way to obtain such a value is through a constructor that establishes
//! the property. Build a `Witness` with [`Witness::from_proof`].
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
//!         Some(Witness::from_proof(value, NonZero))
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
#![cfg_attr(docsrs, feature(doc_cfg))]

use core::marker::PhantomData;

/// A zero-sized marker trait implemented by proof types.
///
/// A type implementing `Proof` carries no data; it exists only as a
/// compile-time witness that some property holds. Implement it for the unit
/// struct (or any zero-sized type) that names the property you want to
/// track.
///
/// # Soundness
///
/// The only public constructor for a `Witness<T, P>` is
/// [`Witness::from_proof`], which consumes a *value* of `P`. Because `P` is
/// a zero-sized type you define, you should construct that value only inside
/// a function that has just established the property (e.g. a checked
/// constructor that returns `None` otherwise). If the proof type has no
/// public constructor outside its defining module, then no caller elsewhere
/// can ever mint a `Witness` for it.
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
/// has the same size as `T`. A `Witness` can only be constructed by supplying
/// a value of the proof type `P` (see [`Witness::from_proof`]), so the
/// property is guaranteed to have been established wherever the proof value
/// originated.
///
/// `T` is not required to be `Copy`; the `Witness` is `Copy` *only when* `T`
/// is, via the hand-written `Copy` impl.
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
///         Some(Witness::from_proof(value, NonZero))
///     }
/// }
///
/// let w = checked_nonzero(42).unwrap();
/// assert_eq!(*w.value(), 42);
/// let raw = w.into_inner();
/// assert_eq!(raw, 42);
/// ```
pub struct Witness<T, P: Proof> {
    /// The carried value.
    value: T,
    /// Zero-sized marker pinning the proof type `P`.
    _proof: PhantomData<P>,
}

impl<T: Clone, P: Proof> Clone for Witness<T, P> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            _proof: PhantomData,
        }
    }
}

impl<T: Copy, P: Proof> Copy for Witness<T, P> {}

impl<T: core::fmt::Debug, P: Proof> core::fmt::Debug for Witness<T, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Witness").field("value", &self.value).finish()
    }
}

impl<T, P: Proof> Witness<T, P> {
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
    /// let w = Witness::from_proof("hello", P);
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
    /// let w = Witness::from_proof(String::from("x"), P);
    /// let s = w.into_inner();
    /// assert_eq!(s, "x");
    /// ```
    #[must_use]
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T, P: Proof> Witness<T, P> {
    /// Constructs a `Witness` from a value together with a *value* of its
    /// proof type `P`.
    ///
    /// This is the only public constructor. Because it consumes a value of
    /// `P`, a `Witness` can only be created where such a value is available —
    /// i.e. where the property has just been established. The proof value is
    /// zero-sized and discarded; only its type matters.
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
    fn from_proof_and_value() {
        let w = Witness::from_proof(5u32, NonZero);
        assert_eq!(*w.value(), 5);
    }

    #[test]
    fn into_inner_returns_value() {
        let w = Witness::from_proof("abc", NonZero);
        assert_eq!(w.into_inner(), "abc");
    }

    #[test]
    fn from_proof_consumes_proof_value() {
        let w = Witness::from_proof(4i64, Even);
        assert_eq!(*w.value(), 4);
    }

    #[test]
    fn copy_when_t_is_copy() {
        let w = Witness::from_proof(1u8, NonZero);
        let w2 = w;
        assert_eq!(*w.value(), 1);
        assert_eq!(*w2.value(), 1);
    }

    #[test]
    fn non_copy_t_works() {
        let w = Witness::from_proof(Bag { items: [1, 2] }, NonZero);
        let w2 = w.clone();
        assert_eq!(w.into_inner(), Bag { items: [1, 2] });
        assert_eq!(w2.into_inner(), Bag { items: [1, 2] });
    }
}
