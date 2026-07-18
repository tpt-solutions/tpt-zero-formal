//! Probability building blocks for `no_std`: a [`Distribution`] trait that
//! captures the analytic properties of a probability distribution, and a
//! [`Dist`] container that holds a set of observed samples and derives
//! empirical statistics from them. Part of the
//! [tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal)
//! ecosystem.
//!
//! The crate depends only on [`tpt-zero-stats`](https://docs.rs/tpt-zero-stats)
//! for its descriptive statistics, and is therefore `no_std` with zero
//! external dependencies.
//!
//! A [`Distribution`] describes a *parametric* family (for example the
//! standard normal) and exposes its probability density, cumulative
//! distribution function, mean, and variance analytically where they exist.
//! A [`Dist`] holds *observed* values — a view over `&[f64]` in the
//! `default` configuration, or an owned `Vec<f64>` behind the `alloc`
//! feature — and computes their empirical statistics with the helpers
//! re-exported from `tpt-zero-stats`.
//!
//! ```
//! use tpt_zero_prob::{Dist, Distribution};
//!
//! // An empirical sample viewed over a borrowed slice (no allocation).
//! let samples = [0.1, 0.5, 0.9, 0.2, 0.8];
//! let dist = Dist::new(&samples);
//! assert!(dist.empirical_mean().is_some());
//!
//! // An analytic distribution.
//! let u = tpt_zero_prob::distributions::Uniform::new(0.0, 1.0).unwrap();
//! assert_eq!(u.mean(), Some(0.5));
//! assert!((u.variance().unwrap() - 1.0 / 12.0).abs() < 1e-12);
//! ```
//!
//! # Features
//!
//! | Feature | Default | Enables |
//! |---|---|---|
//! | `alloc` | off | owned [`Dist`] storage via `Vec<f64>` |
//! | `std` | off | implies `alloc` |
//!
//! This crate builds with `--no-default-features` (pure `core`, samples held
//! as a borrowed `&[f64]`).

#![no_std]
#![warn(missing_docs)]
#![forbid(unsafe_code)]
// The Distribution trait operates purely on `f64` and the analytic formulas
// require a handful of integer-to-float conversions and float comparisons
// that the pedantic lints flag but are exact and bounded by data length, so
// they are allowed here.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::similar_names,
    clippy::neg_cmp_op_on_partial_ord
)]

use tpt_zero_stats as stats;

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod distributions;
mod sample;

pub use distributions::{Exponential, Normal, Uniform};
pub use sample::Sample;

/// A probability distribution described analytically.
///
/// Implementors expose the probability density function ([`pdf`](Self::pdf)),
/// cumulative distribution function ([`cdf`](Self::cdf)), and — where closed
/// forms exist — the [`mean`](Self::mean) and [`variance`](Self::variance).
///
/// The trait is generic over the value type [`Value`](Self::Value) so that a
/// distribution may describe scalars, vectors, or other outcomes, while the
/// analytic accessors (`pdf`, `cdf`, `mean`, `variance`) are uniformly `f64`
/// because the crate targets one-dimensional probability over the reals.
///
/// # Examples
///
/// ```
/// use tpt_zero_prob::Distribution;
///
/// let u = tpt_zero_prob::distributions::Uniform::new(0.0, 1.0).unwrap();
/// assert_eq!(u.pdf(0.5), 1.0);
/// assert_eq!(u.cdf(0.5), 0.5);
/// assert_eq!(u.mean(), Some(0.5));
/// assert!((u.variance().unwrap() - 1.0 / 12.0).abs() < 1e-12);
/// ```
pub trait Distribution {
    /// The kind of value this distribution produces.
    type Value;

    /// Returns the probability density of `x`.
    ///
    /// For discrete distributions this is the probability mass function.
    fn pdf(&self, x: f64) -> f64;

    /// Returns the cumulative probability `P(X <= x)`.
    fn cdf(&self, x: f64) -> f64;

    /// Returns the analytic mean (expected value), if a closed form exists.
    ///
    /// Returns `None` when the distribution has no finite mean (for example
    /// a Cauchy-like distribution).
    fn mean(&self) -> Option<f64>;

    /// Returns the analytic variance, if a closed form exists.
    ///
    /// Returns `None` when the distribution has no finite variance.
    fn variance(&self) -> Option<f64>;
}

/// A container holding observed samples and providing empirical statistics.
///
/// In the `default` configuration `Dist` borrows its samples from a `&[f64]`
/// (a zero-allocation view). When the `alloc` feature is enabled it can also
/// own a `Vec<f64>` via the `from_vec` constructor.
///
/// Empirical statistics are delegated to [`tpt-zero-stats`], so they return
/// `Option<f64>` for insufficient data rather than panicking.
///
/// # Examples
///
/// ```
/// use tpt_zero_prob::Dist;
///
/// let samples = [1.0, 2.0, 3.0, 4.0, 5.0];
/// let dist = Dist::new(&samples);
/// assert_eq!(dist.empirical_mean(), Some(3.0));
/// assert!((dist.empirical_variance().unwrap() - 2.5).abs() < 1e-12);
/// ```
pub struct Dist<T> {
    samples: T,
}

impl Dist<&'static [f64]> {
    /// Creates an owned-from-`static` empty sample set.
    ///
    /// Mostly useful for doctests and `const` contexts; prefer
    /// [`Dist::new`](Self::new) for runtime data.
    #[must_use]
    pub const fn empty_static() -> Self {
        Dist { samples: &[] }
    }
}

impl<'a> Dist<&'a [f64]> {
    /// Creates a `Dist` that borrows the given slice of samples.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_prob::Dist;
    ///
    /// let dist = Dist::new(&[0.0, 1.0, 2.0]);
    /// assert_eq!(dist.len(), 3);
    /// ```
    #[must_use]
    pub const fn new(samples: &'a [f64]) -> Self {
        Dist { samples }
    }
}

#[cfg(feature = "alloc")]
impl Dist<alloc::vec::Vec<f64>> {
    /// Creates a `Dist` that owns the given vector of samples.
    ///
    /// Only available with the `alloc` feature.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_prob::Dist;
    ///
    /// let dist = Dist::from_vec(alloc::vec![0.0, 1.0, 2.0]);
    /// assert_eq!(dist.len(), 3);
    /// ```
    #[must_use]
    pub fn from_vec(samples: alloc::vec::Vec<f64>) -> Self {
        Dist { samples }
    }
}

impl<T: Sample> Dist<T> {
    /// Returns the number of observed samples.
    #[must_use]
    pub fn len(&self) -> usize {
        self.samples.as_slice().len()
    }

    /// Returns `true` if there are no observed samples.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.samples.as_slice().is_empty()
    }

    /// Returns the observed samples as a slice.
    #[must_use]
    pub fn as_slice(&self) -> &[f64] {
        self.samples.as_slice()
    }

    /// Returns the empirical (arithmetic) mean of the samples, or `None` if
    /// the sample set is empty.
    #[must_use]
    pub fn empirical_mean(&self) -> Option<f64> {
        stats::mean(self.samples.as_slice())
    }

    /// Returns the empirical *sample* variance of the samples, or `None` if
    /// there are fewer than two samples.
    #[must_use]
    pub fn empirical_variance(&self) -> Option<f64> {
        stats::variance(self.samples.as_slice())
    }

    /// Returns the empirical *population* variance of the samples, or `None`
    /// if the sample set is empty.
    #[must_use]
    pub fn empirical_population_variance(&self) -> Option<f64> {
        stats::population_variance(self.samples.as_slice())
    }

    /// Returns the empirical *sample* standard deviation of the samples, or
    /// `None` if there are fewer than two samples.
    #[must_use]
    pub fn empirical_std_dev(&self) -> Option<f64> {
        stats::std_dev(self.samples.as_slice())
    }

    /// Returns the minimum observed value, or `None` if the sample set is
    /// empty.
    #[must_use]
    pub fn min(&self) -> Option<f64> {
        stats::min(self.samples.as_slice())
    }

    /// Returns the maximum observed value, or `None` if the sample set is
    /// empty.
    #[must_use]
    pub fn max(&self) -> Option<f64> {
        stats::max(self.samples.as_slice())
    }

    /// Returns a histogram of the samples with `bins` equal-width bins
    /// spanning `[min, max]`.
    ///
    /// Returns an array `counts` of length `bins` where `counts[i]` is the
    /// number of samples falling in bin `i`. The last bin is closed on the
    /// right (`x == max` is counted in the final bin) so the entire range is
    /// covered. Returns `None` if `bins == 0` or the sample set is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_prob::Dist;
    ///
    /// let dist = Dist::new(&[0.0, 0.5, 1.0]);
    /// let hist = dist.histogram::<2>().unwrap();
    /// assert_eq!(hist, [1, 2]);
    /// ```
    #[must_use]
    pub fn histogram<const BINS: usize>(&self) -> Option<[usize; BINS]> {
        let data = self.samples.as_slice();
        if BINS == 0 || data.is_empty() {
            return None;
        }
        let lo = stats::min(data)?;
        let hi = stats::max(data)?;
        let width = hi - lo;
        let mut counts = [0usize; BINS];
        if width == 0.0 {
            // All samples equal: pile them into the final bin.
            for _ in data {
                counts[BINS - 1] += 1;
            }
            return Some(counts);
        }
        for &x in data {
            let idx = (((x - lo) / width) * (BINS as f64)) as usize;
            let idx = idx.min(BINS - 1);
            counts[idx] += 1;
        }
        Some(counts)
    }
}

impl<T: Sample> Sample for Dist<T> {
    fn as_slice(&self) -> &[f64] {
        self.samples.as_slice()
    }
}

impl Sample for &[f64] {
    fn as_slice(&self) -> &[f64] {
        self
    }
}

#[cfg(feature = "alloc")]
impl Sample for alloc::vec::Vec<f64> {
    fn as_slice(&self) -> &[f64] {
        self.as_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_dist_has_no_stats() {
        let dist = Dist::new(&[] as &[f64]);
        assert!(dist.empirical_mean().is_none());
        assert!(dist.empirical_variance().is_none());
        assert!(dist.histogram::<4>().is_none());
        assert!(dist.is_empty());
    }

    #[test]
    fn empirical_mean_and_variance() {
        let data = [1.0, 2.0, 3.0, 4.0, 5.0];
        let dist = Dist::new(&data);
        assert_eq!(dist.empirical_mean(), Some(3.0));
        assert!((dist.empirical_variance().unwrap() - 2.5).abs() < 1e-12);
        assert_eq!(dist.min(), Some(1.0));
        assert_eq!(dist.max(), Some(5.0));
        assert_eq!(dist.len(), 5);
    }

    #[test]
    fn histogram_covers_range() {
        let data = [0.0, 0.25, 0.5, 0.75, 1.0];
        let dist = Dist::new(&data);
        let hist = dist.histogram::<5>().unwrap();
        assert_eq!(hist.iter().sum::<usize>(), 5);
        assert_eq!(hist, [1, 1, 1, 1, 1]);
    }

    #[test]
    fn histogram_equal_values() {
        let data = [2.0, 2.0, 2.0];
        let dist = Dist::new(&data);
        let hist = dist.histogram::<3>().unwrap();
        assert_eq!(hist, [0, 0, 3]);
    }

    #[test]
    fn dist_is_a_sample() {
        let data = [1.0, 2.0];
        let dist = Dist::new(&data);
        let s: &dyn Sample = &dist;
        assert_eq!(s.as_slice(), &[1.0, 2.0]);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn owned_vec_dist() {
        let dist = Dist::from_vec(alloc::vec![1.0, 2.0, 3.0]);
        assert_eq!(dist.empirical_mean(), Some(2.0));
        assert!(!dist.is_empty());
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn vec_is_a_sample() {
        let v = alloc::vec![1.0, 2.0];
        let s: &dyn Sample = &v;
        assert_eq!(s.as_slice(), &[1.0, 2.0]);
    }
}
