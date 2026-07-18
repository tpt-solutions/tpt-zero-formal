//! The [`Sample`] trait abstracting over the storage backing a [`Dist`].

/// A collection of observed samples that supports empirical statistics.
///
/// `Sample` abstracts over the backing storage so that the same empirical
/// helpers work whether the values are a borrowed slice (the `default`
/// `no_std` configuration) or an owned allocation (behind the `alloc`
/// feature).
///
/// # Examples
///
/// ```
/// use tpt_zero_prob::{Dist, Sample};
///
/// let samples = [1.0, 2.0, 3.0];
/// let dist = Dist::new(&samples);
/// let view: &[f64] = dist.as_slice();
/// assert_eq!(view, &[1.0, 2.0, 3.0]);
/// ```
pub trait Sample {
    /// Returns the observed values as a slice.
    ///
    /// The returned slice is never `None`; an empty backing store yields an
    /// empty slice, and callers should treat empty data via the
    /// `Option`-returning statistics on [`Dist`](crate::Dist).
    fn as_slice(&self) -> &[f64];
}
