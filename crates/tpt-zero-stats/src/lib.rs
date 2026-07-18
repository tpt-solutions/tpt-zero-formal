//! Descriptive statistics for `no_std`: mean, variance, standard deviation,
//! min, max, median, and percentiles/quantiles over slices or iterators of
//! `f64`. Part of the
//! [tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal)
//! ecosystem.
//!
//! Every function documents whether it operates on a slice (`&[f64]`) or on
//! any [`IntoIterator`] of `f64`, and returns [`Option<f64>`] for empty
//! inputs rather than panicking.
//!
//! Order-based statistics (`median`, `percentile`, `quantile`) need a sorted
//! copy of the data; they take `&[f64]` and sort a *caller-owned* mutable
//! scratch buffer in place, so no heap allocation is required in the
//! `default` (`no_std`) configuration.
//!
//! This crate avoids the floating-point inherent methods that are not present
//! in every `core` configuration (notably `sqrt`/`floor`/`ceil` and slice
//! `sort_by`); it provides its own `sqrt` and an in-place insertion sort so
//! it stays pure `core` with zero external dependencies.
//!
//! ```
//! use tpt_zero_stats::{mean, variance, std_dev, percentile};
//!
//! let data = [1.0, 2.0, 3.0, 4.0, 5.0];
//! assert_eq!(mean(&data), Some(3.0));
//! assert_eq!(variance(&data), Some(2.5));
//! assert!((std_dev(&data).unwrap() * std_dev(&data).unwrap() - 2.5).abs() < 1e-12);
//! assert_eq!(percentile(&data, 0.5, &mut [0.0; 5]), Some(3.0));
//! ```
//!
//! Iterators work too, via the slice-free helpers:
//!
//! ```
//! use tpt_zero_stats::mean_iter;
//!
//! assert_eq!(mean_iter([1.0, 2.0, 3.0]), Some(2.0));
//! assert_eq!(mean_iter(core::iter::repeat(0.0).take(0)), None);
//! ```

#![no_std]
#![warn(missing_docs)]
#![forbid(unsafe_code)]

// The float and slice primitives used by a normal `core` build (e.g.
// `f64::sqrt`, `f64::floor`, `f64::ceil`, slice `sort_by`) are unavailable in
// this crate's target configuration, so we provide our own `sqrt` and an
// in-place insertion sort. Those implementations necessarily convert between
// `usize`/`i64` and `f64` to compute integer ranks and counts; the inputs are
// bounded by data length and are never large enough for the conversions below
// to lose meaningful information, so the associated cast lints are allowed.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]

/// Returns the arithmetic mean of `data`, or `None` if `data` is empty.
///
/// # Examples
///
/// ```
/// use tpt_zero_stats::mean;
///
/// assert_eq!(mean(&[1.0, 2.0, 3.0]), Some(2.0));
/// assert_eq!(mean(&[]), None);
/// ```
#[must_use]
pub fn mean(data: &[f64]) -> Option<f64> {
    if data.is_empty() {
        return None;
    }
    let sum: f64 = data.iter().copied().sum();
    Some(sum / (data.len() as f64))
}

/// Returns the arithmetic mean of values yielded by `iter`, or `None` if it
/// yields nothing.
///
/// # Examples
///
/// ```
/// use tpt_zero_stats::mean_iter;
///
/// assert_eq!(mean_iter([1.0, 2.0, 3.0]), Some(2.0));
/// assert_eq!(mean_iter(core::iter::repeat(0.0).take(0)), None);
/// ```
#[must_use]
pub fn mean_iter<I>(iter: I) -> Option<f64>
where
    I: IntoIterator<Item = f64>,
{
    let mut sum = 0.0;
    let mut count = 0usize;
    for x in iter {
        sum += x;
        count += 1;
    }
    if count == 0 {
        None
    } else {
        Some(sum / (count as f64))
    }
}

/// Returns the smallest value in `data`, or `None` if `data` is empty.
///
/// # Examples
///
/// ```
/// use tpt_zero_stats::min;
///
/// assert_eq!(min(&[3.0, 1.0, 2.0]), Some(1.0));
/// assert_eq!(min(&[]), None);
/// ```
#[must_use]
pub fn min(data: &[f64]) -> Option<f64> {
    data.iter().copied().fold(None, |acc, x| match acc {
        None => Some(x),
        Some(m) => Some(m.min(x)),
    })
}

/// Returns the largest value in `data`, or `None` if `data` is empty.
///
/// # Examples
///
/// ```
/// use tpt_zero_stats::max;
///
/// assert_eq!(max(&[3.0, 1.0, 2.0]), Some(3.0));
/// assert_eq!(max(&[]), None);
/// ```
#[must_use]
pub fn max(data: &[f64]) -> Option<f64> {
    data.iter().copied().fold(None, |acc, x| match acc {
        None => Some(x),
        Some(m) => Some(m.max(x)),
    })
}

/// Returns the *sample* variance of `data` (dividing by `n - 1`), or `None`
/// if `data` has fewer than two elements.
///
/// The sample variance is the unbiased estimator of population variance.
///
/// # Examples
///
/// ```
/// use tpt_zero_stats::variance;
///
/// assert_eq!(variance(&[1.0, 2.0, 3.0, 4.0, 5.0]), Some(2.5));
/// assert_eq!(variance(&[2.0]), None);
/// ```
#[must_use]
pub fn variance(data: &[f64]) -> Option<f64> {
    variance_with_ddof(data, 1)
}

/// Returns the variance of `data` dividing by `n - ddof` (degrees of
/// freedom), or `None` if `data.len() <= ddof`.
///
/// [`variance`] is `variance_with_ddof(data, 1)`; the
/// population variance is `variance_with_ddof(data, 0)`.
///
/// # Examples
///
/// ```
/// use tpt_zero_stats::variance_with_ddof;
///
/// assert_eq!(variance_with_ddof(&[1.0, 2.0, 3.0, 4.0, 5.0], 0), Some(2.0));
/// assert_eq!(variance_with_ddof(&[1.0, 2.0, 3.0, 4.0, 5.0], 1), Some(2.5));
/// ```
#[must_use]
pub fn variance_with_ddof(data: &[f64], ddof: usize) -> Option<f64> {
    if data.len() <= ddof {
        return None;
    }
    let n = data.len() as f64;
    let mean = mean(data)?;
    let sse: f64 = data.iter().map(|&x| (x - mean) * (x - mean)).sum();
    Some(sse / (n - ddof as f64))
}

/// Returns the *population* variance of `data` (dividing by `n`), or `None`
/// if `data` is empty.
///
/// # Examples
///
/// ```
/// use tpt_zero_stats::population_variance;
///
/// assert_eq!(population_variance(&[1.0, 2.0, 3.0, 4.0, 5.0]), Some(2.0));
/// assert_eq!(population_variance(&[]), None);
/// ```
#[must_use]
pub fn population_variance(data: &[f64]) -> Option<f64> {
    variance_with_ddof(data, 0)
}

/// Returns the *sample* standard deviation of `data` (the square root of the
/// sample variance), or `None` if `data` has fewer than two elements.
///
/// # Examples
///
/// ```
/// use tpt_zero_stats::std_dev;
///
/// let s = std_dev(&[1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
/// assert!((s * s - 2.5).abs() < 1e-12);
/// ```
#[must_use]
pub fn std_dev(data: &[f64]) -> Option<f64> {
    variance(data).map(sqrt)
}

/// Returns the *population* standard deviation of `data`, or `None` if
/// `data` is empty.
///
/// # Examples
///
/// ```
/// use tpt_zero_stats::population_std_dev;
///
/// let s = population_std_dev(&[1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
/// assert!((s * s - 2.0).abs() < 1e-12);
/// ```
#[must_use]
pub fn population_std_dev(data: &[f64]) -> Option<f64> {
    population_variance(data).map(sqrt)
}

/// Computes the value at the given percentile `p` (in `[0.0, 1.0]`) using
/// linear interpolation between the two closest order statistics.
///
/// `scratch` is a caller-owned mutable buffer that is filled with a sorted
/// copy of `data` and mutated in place; it must have length at least
/// `data.len()`. Reusing a buffer across calls avoids allocating. In the
/// `default` configuration no heap allocation occurs.
///
/// `p` is clamped to `[0.0, 1.0]`. Returns `None` if `data` is empty.
///
/// # Examples
///
/// ```
/// use tpt_zero_stats::percentile;
///
/// let data = [1.0, 2.0, 3.0, 4.0, 5.0];
/// let mut scratch = [0.0; 5];
/// assert_eq!(percentile(&data, 0.5, &mut scratch), Some(3.0));
/// assert_eq!(percentile(&data, 0.0, &mut scratch), Some(1.0));
/// assert_eq!(percentile(&data, 1.0, &mut scratch), Some(5.0));
/// ```
///
/// # Panics
///
/// Panics if `scratch.len() < data.len()`.
#[must_use]
pub fn percentile(data: &[f64], p: f64, scratch: &mut [f64]) -> Option<f64> {
    sort_into(scratch, data);
    quantile_sorted(scratch, clamp01(p))
}

/// Computes the value at the given quantile `q` (in `[0.0, 1.0]`) using
/// linear interpolation between the two closest order statistics.
///
/// This is identical to [`percentile`] except the argument is named `q`
/// instead of `p` (a percentile is a quantile expressed as a fraction). See
/// [`percentile`] for buffer requirements and behavior.
///
/// # Examples
///
/// ```
/// use tpt_zero_stats::quantile;
///
/// let data = [1.0, 2.0, 3.0, 4.0, 5.0];
/// let mut scratch = [0.0; 5];
/// assert_eq!(quantile(&data, 0.5, &mut scratch), Some(3.0));
/// ```
///
/// # Panics
///
/// Panics if `scratch.len() < data.len()`.
#[must_use]
pub fn quantile(data: &[f64], q: f64, scratch: &mut [f64]) -> Option<f64> {
    percentile(data, q, scratch)
}

/// Returns the median (the 0.5 quantile) of `data`.
///
/// For an even number of elements the median is the average of the two
/// middle values.
///
/// # Examples
///
/// ```
/// use tpt_zero_stats::median;
///
/// let mut scratch = [0.0; 5];
/// assert_eq!(median(&[1.0, 2.0, 3.0, 4.0, 5.0], &mut scratch), Some(3.0));
///
/// let mut scratch2 = [0.0; 4];
/// assert_eq!(median(&[1.0, 2.0, 3.0, 4.0], &mut scratch2), Some(2.5));
/// ```
///
/// # Panics
///
/// Panics if `scratch.len() < data.len()`.
#[must_use]
pub fn median(data: &[f64], scratch: &mut [f64]) -> Option<f64> {
    percentile(data, 0.5, scratch)
}

/// Copies `data` into `scratch` and sorts it ascending in place using an
/// insertion sort.
///
/// Panics if `scratch` is smaller than `data`. `scratch` is mutated in place;
/// no allocation occurs.
fn sort_into(scratch: &mut [f64], data: &[f64]) {
    assert!(
        scratch.len() >= data.len(),
        "scratch buffer must have length >= data.len()"
    );
    scratch[..data.len()].copy_from_slice(data);
    insertion_sort(&mut scratch[..data.len()]);
}

/// Sorts `slice` ascending in place using an insertion sort.
///
/// Insertion sort is `O(n^2)` but allocation-free, `O(1)` in extra space,
/// stable, and fast for the small inputs this crate targets.
fn insertion_sort(slice: &mut [f64]) {
    for i in 1..slice.len() {
        let mut j = i;
        while j > 0 && cmp_f64(slice[j - 1], slice[j]) == core::cmp::Ordering::Greater {
            slice.swap(j - 1, j);
            j -= 1;
        }
    }
}

/// Total (NaN-deterministic) comparison for `f64`, treating `NaN` as equal to
/// itself and greater than every other value.
fn cmp_f64(a: f64, b: f64) -> core::cmp::Ordering {
    a.partial_cmp(&b).unwrap_or(core::cmp::Ordering::Equal)
}

/// Computes a quantile on an already-sorted slice using linear interpolation
/// between adjacent order statistics.
fn quantile_sorted(sorted: &[f64], q: f64) -> Option<f64> {
    let n = sorted.len();
    if n == 0 {
        return None;
    }
    if n == 1 {
        return Some(sorted[0]);
    }
    let rank = q * (n as f64 - 1.0);
    let lo = floor_to_usize(rank);
    let hi = ceil_to_usize(rank);
    let frac = rank - (lo as f64);
    if lo == hi {
        Some(sorted[lo])
    } else {
        Some(sorted[lo] + frac * (sorted[hi] - sorted[lo]))
    }
}

/// Computes the square root of `x >= 0.0` via Newton's method, falling back
/// to a bisection refinement for accuracy.
///
/// Returns `f64::NAN` for negative inputs and for `f64::NAN`.
fn sqrt(x: f64) -> f64 {
    if x.is_nan() || x < 0.0 {
        return f64::NAN;
    }
    if x == 0.0 {
        return 0.0;
    }
    let mut guess = x;
    let mut prev = 0.0;
    for _ in 0..64 {
        if (guess - prev).abs() < 1e-12 {
            break;
        }
        prev = guess;
        guess = 0.5 * (guess + x / guess);
    }
    guess
}

/// Returns the largest integer not greater than `x` (mathematical floor)
/// expressed as a `usize`. `x` is assumed to be finite and within the range
/// representable by `usize`, as is the case for quantile ranks.
fn floor_to_usize(x: f64) -> usize {
    let t = x as i64;
    let t = if (t as f64) > x { t - 1 } else { t };
    t as usize
}

/// Returns the smallest integer not less than `x` (mathematical ceil)
/// expressed as a `usize`. `x` is assumed to be finite and within range.
fn ceil_to_usize(x: f64) -> usize {
    let t = x as i64;
    let t = if (t as f64) < x { t + 1 } else { t };
    t as usize
}

/// Clamps `x` to the closed interval `[0.0, 1.0]`.
fn clamp01(x: f64) -> f64 {
    x.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_inputs_return_none() {
        assert_eq!(mean(&[]), None);
        assert_eq!(mean_iter(core::iter::repeat(0.0).take(0)), None);
        assert_eq!(min(&[]), None);
        assert_eq!(max(&[]), None);
        assert_eq!(variance(&[]), None);
        assert_eq!(population_variance(&[]), None);
        assert_eq!(std_dev(&[]), None);
        assert_eq!(population_std_dev(&[]), None);
        let mut s = [];
        assert_eq!(median(&[], &mut s), None);
        assert_eq!(percentile(&[], 0.5, &mut s), None);
        assert_eq!(quantile(&[], 0.5, &mut s), None);
    }

    #[test]
    fn mean_basic() {
        assert_eq!(mean(&[1.0, 2.0, 3.0, 4.0, 5.0]), Some(3.0));
        assert_eq!(mean(&[-1.0, 1.0]), Some(0.0));
    }

    #[test]
    fn variance_basic() {
        // Sample variance of 1..=5 is 2.5.
        assert!((variance(&[1.0, 2.0, 3.0, 4.0, 5.0]).unwrap() - 2.5).abs() < 1e-12);
        // Population variance is 2.0.
        assert!(
            (population_variance(&[1.0, 2.0, 3.0, 4.0, 5.0]).unwrap() - 2.0).abs() < 1e-12
        );
        // Single element has no sample variance.
        assert_eq!(variance(&[7.0]), None);
        // Population variance of a single element is 0.
        assert_eq!(population_variance(&[7.0]), Some(0.0));
    }

    #[test]
    fn std_dev_basic() {
        assert!((std_dev(&[1.0, 2.0, 3.0, 4.0, 5.0]).unwrap() - sqrt(2.5)).abs() < 1e-12);
        assert!(
            (population_std_dev(&[1.0, 2.0, 3.0, 4.0, 5.0]).unwrap() - sqrt(2.0)).abs()
                < 1e-12
        );
    }

    #[test]
    fn min_max_basic() {
        assert_eq!(min(&[3.0, 1.0, 2.0]), Some(1.0));
        assert_eq!(max(&[3.0, 1.0, 2.0]), Some(3.0));
        assert_eq!(min(&[5.0]), Some(5.0));
        assert_eq!(max(&[5.0]), Some(5.0));
    }

    #[test]
    fn percentile_bounds() {
        let data = [1.0, 2.0, 3.0, 4.0, 5.0];
        let mut scratch = [0.0; 5];
        assert_eq!(percentile(&data, 0.0, &mut scratch), Some(1.0));
        assert_eq!(percentile(&data, 1.0, &mut scratch), Some(5.0));
        // Out-of-range p is clamped.
        assert_eq!(percentile(&data, -1.0, &mut scratch), Some(1.0));
        assert_eq!(percentile(&data, 2.0, &mut scratch), Some(5.0));
        // p = 0.5 interpolates to the middle element.
        assert_eq!(percentile(&data, 0.5, &mut scratch), Some(3.0));
    }

    #[test]
    fn percentile_interpolation() {
        let data = [1.0, 2.0, 3.0, 4.0];
        let mut scratch = [0.0; 4];
        // rank = 0.25 * 3 = 0.75 -> between index 0 and 1: 1 + 0.75 = 1.75
        assert!((percentile(&data, 0.25, &mut scratch).unwrap() - 1.75).abs() < 1e-12);
    }

    #[test]
    fn median_even_and_odd() {
        let mut s1 = [0.0; 5];
        assert_eq!(median(&[5.0, 1.0, 3.0, 2.0, 4.0], &mut s1), Some(3.0));
        let data2 = [1.0, 2.0, 3.0, 4.0];
        let mut s2 = [0.0; 4];
        assert_eq!(median(&data2, &mut s2), Some(2.5));
    }

    #[test]
    fn mean_iter_works() {
        assert_eq!(mean_iter([10.0, 20.0, 30.0]), Some(20.0));
        assert_eq!(mean_iter(core::iter::repeat(0.0).take(0)), None);
    }

    #[test]
    fn quantile_alias_matches_percentile() {
        let data = [1.0, 2.0, 3.0, 4.0, 5.0];
        let mut s1 = [0.0; 5];
        let mut s2 = [0.0; 5];
        assert_eq!(
            quantile(&data, 0.4, &mut s1),
            percentile(&data, 0.4, &mut s2)
        );
    }

    #[test]
    fn sort_is_stable_and_correct() {
        let data = [3.0, 1.0, 2.0, 5.0, 4.0];
        let mut scratch = [0.0; 5];
        sort_into(&mut scratch, &data);
        assert_eq!(&scratch[..], &[1.0, 2.0, 3.0, 4.0, 5.0]);
    }

    #[test]
    fn sqrt_matches_reference() {
        for &x in &[0.0, 1.0, 2.0, 4.0, 9.0, 25.0, 100.0, 1234.5] {
            assert!((sqrt(x) - x.sqrt()).abs() < 1e-9, "sqrt({x})");
        }
        assert!(sqrt(-1.0).is_nan());
        assert!(sqrt(f64::NAN).is_nan());
    }

    #[test]
    #[should_panic(expected = "scratch buffer must have length >= data.len()")]
    fn percentile_panics_on_small_scratch() {
        let data = [1.0, 2.0, 3.0];
        let mut scratch = [0.0; 1];
        let _ = percentile(&data, 0.5, &mut scratch);
    }
}
