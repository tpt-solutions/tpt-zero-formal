//! Separation-logic-style ghost-state markers for `no_std`. Part of the
//! [tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal)
//! ecosystem.
//!
//! A `Ghost<T, P>` pairs a value `T` with a zero-sized *provenance marker*
//! `P` — either [`Proven`] or [`Unproven`] — that records, at the type
//! level, whether some property of `T` has been established. The marker
//! occupies no space: `Ghost<T, P>` has exactly the same size, alignment,
//! and layout as `T` itself, because `P` is stored as [`PhantomData`].
//!
//! This mirrors the *ghost state* of separation logic: a piece of
//! information that the program carries around to reason about a property
//! but that is erased at runtime, leaving zero cost behind.
//!
//! ```
//! use tpt_zero_ghost::{Ghost, GhostProven, Proven, Unproven};
//!
//! // A value whose property has not yet been established.
//! let raw: Ghost<u32, Unproven> = Ghost::new(42);
//!
//! // The caller asserts the invariant; this is a ghost operation and is
//! // unsound unless the caller actually maintains it.
//! let proven: GhostProven<u32> = raw.assume_proven();
//!
//! // The value is carried unchanged, at zero runtime cost.
//! assert_eq!(*proven.value(), 42);
//! assert_eq!(core::mem::size_of::<GhostProven<u32>>(), 4);
//! ```
//!
//! The optional [`Ghost::prove`] operation ties the transition into the
//! [`tpt-zero-witness`] crate: it consumes a [`Proof`](tpt_zero_witness::Proof)
//! witness value, producing a [`Proven`] ghost only when that proof type is
//! in scope.

#![no_std]
#![warn(missing_docs)]
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]

use core::marker::PhantomData;

/// Zero-sized marker carried by a [`Ghost`] whose property is **proven** to
/// hold.
///
/// It carries no data; only its type participates in the signature, so a
/// `Ghost<T, Proven>` is distinguishable from a `Ghost<T, Unproven>` by the
/// type checker but has identical runtime representation.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Proven;

/// Zero-sized marker carried by a [`Ghost`] whose property is **not yet
/// proven**.
///
/// It carries no data; only its type participates in the signature. A value
/// tagged `Unproven` must be promoted to [`Proven`] (via `Ghost::prove` or
/// the unsound-but-available [`Ghost::assume_proven`]) before its property
/// may be relied upon.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Unproven;

/// A value `T` tagged with a zero-sized provenance marker `P`, either
/// [`Proven`] or [`Unproven`].
///
/// `P` is stored as [`PhantomData`], so it occupies no space:
/// `Ghost<T, P>` has the same size, alignment, and layout as `T`. The marker
/// exists only to track, at the type level, whether some property of `T` has
/// been established — a piece of *ghost state* that is erased at runtime.
///
/// `Ghost::new` constructs only the [`Unproven`] end of the lattice. Promote
/// to [`Proven`] through [`Ghost::prove`] (sound, given a proof) or the
/// unsound [`Ghost::assume_proven`].
///
/// # Examples
///
/// ```
/// use tpt_zero_ghost::{Ghost, Unproven};
///
/// let unproven: Ghost<u8, Unproven> = Ghost::new(7);
/// assert_eq!(*unproven.value(), 7);
/// assert_eq!(core::mem::size_of::<Ghost<u8, Unproven>>(), 1);
/// ```
pub struct Ghost<T, P> {
    /// The carried value.
    value: T,
    /// Zero-sized marker pinning the provenance type `P`.
    _marker: PhantomData<P>,
}

impl<T: Clone, P> Clone for Ghost<T, P> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T: Copy, P> Copy for Ghost<T, P> {}

impl<T: core::fmt::Debug, P> core::fmt::Debug for Ghost<T, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Ghost").field("value", &self.value).finish()
    }
}

impl<T: PartialEq, P> PartialEq for Ghost<T, P> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: Eq, P> Eq for Ghost<T, P> {}

impl<T: core::hash::Hash, P> core::hash::Hash for Ghost<T, P> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

/// A [`Ghost`] whose property has been established as [`Proven`].
///
/// This is the type you hand to downstream APIs that may rely on the
/// property. Construct it via `Ghost::prove` (sound, given a proof) or
/// [`Ghost::assume_proven`] (unsound without caller-maintained invariants).
pub type GhostProven<T> = Ghost<T, Proven>;

impl<T> Ghost<T, Unproven> {
    /// Constructs a `Ghost` carrying `value` tagged as [`Unproven`].
    ///
    /// Prefer the type alias [`GhostProven`] (for `P = Proven`) only after the
    /// value has actually been proven via [`Ghost::prove`] or
    /// [`Ghost::assume_proven`].
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_ghost::{Ghost, Unproven};
    ///
    /// let g: Ghost<u32, Unproven> = Ghost::new(1);
    /// assert_eq!(*g.value(), 1);
    /// ```
    #[must_use]
    pub const fn new(value: T) -> Self {
        Self {
            value,
            _marker: PhantomData,
        }
    }
}

impl<T, P> Ghost<T, P> {
    /// Returns a shared reference to the carried value.
    ///
    /// The ghost provenance never affects the value being carried; it is
    /// always available regardless of `P`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_ghost::{Ghost, Proven, Unproven};
    ///
    /// let g: Ghost<u32, Proven> = Ghost::<u32, Unproven>::new(5).assume_proven();
    /// assert_eq!(g.value(), &5);
    /// ```
    #[must_use]
    pub const fn value(&self) -> &T {
        &self.value
    }

    /// Consumes the `Ghost`, returning the carried value and discarding the
    /// provenance marker.
    ///
    /// Because the marker is zero-sized, this is a no-op at runtime — the
    /// proof/state distinction is erased.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_ghost::{Ghost, Unproven};
    ///
    /// let g: Ghost<u32, Unproven> = Ghost::new(3);
    /// assert_eq!(g.into_inner(), 3);
    /// ```
    #[must_use]
    pub fn into_inner(self) -> T {
        self.value
    }

    /// Maps the carried value, **resetting** the provenance marker to
    /// [`Unproven`].
    ///
    /// The property was established for the *previous* value; an arbitrary
    /// function `f` may not preserve it, so the result can no longer be
    /// trusted as proven.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_ghost::{Ghost, Unproven};
    ///
    /// let g: Ghost<u32, Unproven> = Ghost::new(2);
    /// let mapped: Ghost<u32, Unproven> = g.map(|x| x * 21);
    /// assert_eq!(*mapped.value(), 42);
    /// ```
    #[must_use]
    pub fn map<U, F>(self, f: F) -> Ghost<U, Unproven>
    where
        F: FnOnce(T) -> U,
    {
        Ghost {
            value: f(self.value),
            _marker: PhantomData,
        }
    }
}

impl<T> Ghost<T, Unproven> {
    /// Promotes an [`Unproven`] ghost to a [`Proven`] one **without checking
    /// the property**.
    ///
    /// This is a *ghost* operation: it changes only the type-level marker,
    /// leaving the value untouched and costing nothing at runtime. It is
    /// **unsound** unless the caller actually maintains the invariant that
    /// the property holds for `self.value`. Use `Ghost::prove` instead when
    /// a [`Proof`](tpt_zero_witness::Proof) witness is available.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_ghost::{Ghost, GhostProven, Unproven};
    ///
    /// let raw: Ghost<u32, Unproven> = Ghost::new(42);
    /// let proven: GhostProven<u32> = raw.assume_proven();
    /// assert_eq!(*proven.value(), 42);
    /// ```
    #[must_use]
    pub fn assume_proven(self) -> Ghost<T, Proven> {
        Ghost {
            value: self.value,
            _marker: PhantomData,
        }
    }
}

#[cfg(feature = "witness")]
impl<T> Ghost<T, Unproven> {
    /// Promotes an [`Unproven`] ghost to a [`Proven`] one by consuming a
    /// [`Proof`](tpt_zero_witness::Proof) witness value.
    ///
    /// This is the sound counterpart to [`Ghost::assume_proven`]: because the
    /// proof type `W` is only in scope where its property has been
    /// established, naming it is what justifies the promotion. The witness
    /// value itself is zero-sized and discarded; only its type matters.
    ///
    /// This method is available because the [`tpt-zero-witness`] crate (and its
    /// [`Proof`] trait) is always pulled in.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_ghost::{Ghost, GhostProven, Unproven};
    /// use tpt_zero_witness::Proof;
    ///
    /// /// Proof that a `u32` is even.
    /// #[derive(Clone, Copy, Debug)]
    /// struct IsEven;
    /// impl Proof for IsEven {}
    ///
    /// fn checked_even(value: u32) -> Option<GhostProven<u32>> {
    ///     if value % 2 == 0 {
    ///         Some(Ghost::<u32, Unproven>::new(value).prove(IsEven))
    ///     } else {
    ///         None
    ///     }
    /// }
    ///
    /// let proven = checked_even(8).unwrap();
    /// assert_eq!(*proven.value(), 8);
    /// ```
    ///
    /// [`Proof`]: tpt_zero_witness::Proof
    #[must_use]
    pub fn prove<W>(self, _witness: W) -> Ghost<T, Proven>
    where
        W: tpt_zero_witness::Proof,
    {
        Ghost {
            value: self.value,
            _marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ghost_is_zero_sized_for_value() {
        assert_eq!(core::mem::size_of::<Ghost<u32, Unproven>>(), 4);
        assert_eq!(core::mem::size_of::<Ghost<u32, Proven>>(), 4);
        assert_eq!(core::mem::size_of::<GhostProven<u32>>(), 4);
        assert_eq!(core::mem::size_of::<Ghost<(), Unproven>>(), 0);
    }

    #[test]
    fn value_is_carried_unchanged() {
        let g: Ghost<u32, Unproven> = Ghost::new(123);
        assert_eq!(*g.value(), 123);
        assert_eq!(g.into_inner(), 123);
    }

    #[test]
    fn assume_proven_preserves_value() {
        let raw: Ghost<u32, Unproven> = Ghost::new(42);
        let proven: GhostProven<u32> = raw.assume_proven();
        assert_eq!(*proven.value(), 42);
    }

    #[test]
    fn map_resets_provenance() {
        let g: Ghost<u32, Unproven> = Ghost::new(2);
        let mapped: Ghost<u32, Unproven> = g.map(|x| x + 40);
        assert_eq!(*mapped.value(), 42);
    }

    #[test]
    fn markers_are_zero_sized() {
        assert_eq!(core::mem::size_of::<Proven>(), 0);
        assert_eq!(core::mem::size_of::<Unproven>(), 0);
    }

    #[test]
    fn equality_and_copy() {
        let a: Ghost<u8, Unproven> = Ghost::new(7);
        let b: Ghost<u8, Unproven> = Ghost::new(7);
        assert_eq!(a, b);
        let c = a;
        assert_eq!(a, c);
    }

    #[cfg(feature = "witness")]
    #[test]
    fn prove_with_witness() {
        use tpt_zero_witness::Proof;

        #[derive(Clone, Copy, Debug)]
        struct IsPositive;
        impl Proof for IsPositive {}

        let raw: Ghost<i32, Unproven> = Ghost::new(5);
        let proven: GhostProven<i32> = raw.prove(IsPositive);
        assert_eq!(*proven.value(), 5);
    }
}
