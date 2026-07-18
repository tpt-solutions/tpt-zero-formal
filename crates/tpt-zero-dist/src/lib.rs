//! Concrete probability distributions for `no_std`: [`Uniform`], [`Normal`],
//! [`Bernoulli`], and [`Poisson`]. Part of the
//! [tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal)
//! ecosystem.
//!
//! Each distribution exposes its analytic density (PDF for continuous
//! distributions, PMF for discrete ones), cumulative distribution function,
//! mean, and variance, and can draw random samples from any generator that
//! implements [`tpt_zero_rand::Rng`]. Sampling is dependency-free and uses
//! textbook algorithms: inverse-transform scaling for [`Uniform`], the
//! Box–Muller transform for [`Normal`], a Bernoulli trial for [`Bernoulli`],
//! and Knuth's multiplication algorithm for [`Poisson`].
//!
//! The transcendental functions those algorithms need (`sqrt`, `ln`, `exp`,
//! `floor`) are reimplemented from scratch in [`math`] because this crate
//! targets a `core`-only environment where the `std` float methods are not
//! available.
//!
//! ```
//! use tpt_zero_dist::{Distribution, Normal};
//! use tpt_zero_rand::{Pcg32, SeedableRng};
//!
//! let n = Normal::new(0.0, 1.0).unwrap();
//! assert_eq!(n.mean(), 0.0);
//! assert_eq!(n.variance(), 1.0);
//!
//! let mut rng = Pcg32::seed_from_u64(42);
//! let x = n.sample(&mut rng);
//! assert!(x.is_finite());
//! ```
//!
//! # Features
//!
//! | Feature | Default | Enables |
//! |---|---|---|
//! | `alloc` | off | reserved for future alloc-dependent helpers |
//! | `std` | off | implies `alloc` |
//!
//! This crate builds with `--no-default-features` (pure `core`, no `alloc`).

#![no_std]
#![warn(missing_docs)]
#![forbid(unsafe_code)]
// The analytic formulas and Knuth/Box-Muller samplers operate purely on `f64`
// and require a handful of exact, bounded integer-to-float conversions and
// direct float comparisons that the pedantic lints flag but are intentional.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::similar_names,
    clippy::neg_cmp_op_on_partial_ord
)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod math;

use math::{exp, ln, sqrt};
use tpt_zero_rand::Rng;

/// `1 / sqrt(2 * pi)`, the normalizing constant of the standard normal PDF.
const FRAC_1_SQRT_2PI: f64 = 0.398_942_280_401_432_7;

/// A probability distribution with analytic moments and RNG sampling.
///
/// The trait unifies continuous and discrete distributions. Each implementor
/// draws values of an associated [`Self::Value`] type from a
/// [`Rng`], and exposes its [`mean`](Self::mean) and
/// [`variance`](Self::variance) in closed form.
///
/// The *density* is deliberately split: continuous distributions implement
/// [`pdf`](Self::pdf) (the probability density function evaluated at a real
/// `x`) while discrete distributions implement [`pmf`](Self::pmf) (the
/// probability mass function evaluated at an integer `k`). The default methods
/// return `0.0` so each distribution overrides only the one that applies to
/// it, and both kinds share a single [`cdf`](Self::cdf).
///
/// # Examples
///
/// ```
/// use tpt_zero_dist::{Distribution, Uniform};
///
/// let u = Uniform::new(0.0, 2.0).unwrap();
/// assert_eq!(u.pdf(1.0), 0.5);
/// assert_eq!(u.cdf(1.0), 0.5);
/// assert_eq!(u.mean(), 1.0);
/// ```
pub trait Distribution {
    /// The kind of value produced by [`sample`](Self::sample).
    type Value;

    /// Draws a single random value from `rng`.
    fn sample<R: Rng>(&self, rng: &mut R) -> Self::Value;

    /// Returns the analytic mean (expected value).
    fn mean(&self) -> f64;

    /// Returns the analytic variance.
    fn variance(&self) -> f64;

    /// Returns the analytic standard deviation, `sqrt(variance)`.
    #[must_use]
    fn std_dev(&self) -> f64 {
        sqrt(self.variance())
    }

    /// Returns the probability *density* at the real point `x`.
    ///
    /// The default implementation returns `0.0`; continuous distributions
    /// override it. Discrete distributions leave it as `0.0` (their mass lives
    /// in [`pmf`](Self::pmf)).
    fn pdf(&self, x: f64) -> f64 {
        let _ = x;
        0.0
    }

    /// Returns the probability *mass* at the integer point `k`.
    ///
    /// The default implementation returns `0.0`; discrete distributions
    /// override it. Continuous distributions leave it as `0.0` (their mass is
    /// zero at any single point).
    fn pmf(&self, k: u64) -> f64 {
        let _ = k;
        0.0
    }

    /// Returns the cumulative probability `P(X <= x)`.
    fn cdf(&self, x: f64) -> f64;
}

/// A continuous uniform distribution `U(a, b)` over the interval `[a, b]`.
///
/// The density is constant at `1 / (b - a)` inside the interval and zero
/// outside it; the mean is `(a + b) / 2` and the variance is `(b - a)^2 / 12`.
///
/// # Examples
///
/// ```
/// use tpt_zero_dist::{Distribution, Uniform};
///
/// let u = Uniform::new(0.0, 1.0).unwrap();
/// assert_eq!(u.pdf(0.5), 1.0);
/// assert_eq!(u.cdf(0.5), 0.5);
/// assert_eq!(u.mean(), 0.5);
/// assert!((u.variance() - 1.0 / 12.0).abs() < 1e-12);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Uniform {
    a: f64,
    b: f64,
}

impl Uniform {
    /// Creates a `Uniform` distribution over `[a, b]`.
    ///
    /// Returns `None` if `a >= b` (the interval is empty or inverted) or if
    /// either bound is non-finite.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_dist::Uniform;
    ///
    /// assert!(Uniform::new(0.0, 1.0).is_some());
    /// assert!(Uniform::new(1.0, 0.0).is_none());
    /// assert!(Uniform::new(f64::NAN, 1.0).is_none());
    /// ```
    #[must_use]
    pub fn new(a: f64, b: f64) -> Option<Self> {
        if !(a < b) || !a.is_finite() || !b.is_finite() {
            return None;
        }
        Some(Uniform { a, b })
    }

    /// Returns the lower bound `a`.
    #[must_use]
    pub fn low(&self) -> f64 {
        self.a
    }

    /// Returns the upper bound `b`.
    #[must_use]
    pub fn high(&self) -> f64 {
        self.b
    }
}

impl Distribution for Uniform {
    type Value = f64;

    fn sample<R: Rng>(&self, rng: &mut R) -> f64 {
        // rng.next_f64() is in [0, 1); scale into [a, b).
        self.a + (self.b - self.a) * rng.next_f64()
    }

    fn mean(&self) -> f64 {
        (self.a + self.b) / 2.0
    }

    fn variance(&self) -> f64 {
        let width = self.b - self.a;
        width * width / 12.0
    }

    fn pdf(&self, x: f64) -> f64 {
        if x >= self.a && x <= self.b {
            1.0 / (self.b - self.a)
        } else {
            0.0
        }
    }

    fn cdf(&self, x: f64) -> f64 {
        if x <= self.a {
            0.0
        } else if x >= self.b {
            1.0
        } else {
            (x - self.a) / (self.b - self.a)
        }
    }
}

/// A normal (Gaussian) distribution `N(mu, sigma^2)` with mean `mu` and
/// standard deviation `sigma > 0`.
///
/// # Examples
///
/// ```
/// use tpt_zero_dist::{Distribution, Normal};
///
/// let n = Normal::standard();
/// assert_eq!(n.mean(), 0.0);
/// assert_eq!(n.variance(), 1.0);
/// assert!((n.pdf(0.0) - 0.398_942_280_401_432_7).abs() < 1e-12);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Normal {
    mu: f64,
    sigma: f64,
}

impl Normal {
    /// Creates a normal distribution with mean `mu` and standard deviation
    /// `sigma`.
    ///
    /// Returns `None` if `sigma <= 0` or if any parameter is non-finite.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_dist::Normal;
    ///
    /// assert!(Normal::new(0.0, 1.0).is_some());
    /// assert!(Normal::new(0.0, 0.0).is_none());
    /// ```
    #[must_use]
    pub fn new(mu: f64, sigma: f64) -> Option<Self> {
        if !(sigma > 0.0) || !mu.is_finite() || !sigma.is_finite() {
            return None;
        }
        Some(Normal { mu, sigma })
    }

    /// Creates the standard normal distribution `N(0, 1)`.
    #[must_use]
    pub const fn standard() -> Self {
        Normal {
            mu: 0.0,
            sigma: 1.0,
        }
    }

    /// Returns the mean parameter `mu`.
    #[must_use]
    pub fn mu(&self) -> f64 {
        self.mu
    }

    /// Returns the standard-deviation parameter `sigma`.
    #[must_use]
    pub fn sigma(&self) -> f64 {
        self.sigma
    }
}

impl Distribution for Normal {
    type Value = f64;

    fn sample<R: Rng>(&self, rng: &mut R) -> f64 {
        // Box-Muller transform: two uniforms -> one standard normal, then
        // shift and scale. `u1` is drawn from (0, 1] so `ln(u1)` is finite.
        let mut u1 = rng.next_f64();
        if u1 <= 0.0 {
            u1 = f64::MIN_POSITIVE;
        }
        let u2 = rng.next_f64();
        let radius = sqrt(-2.0 * ln(u1));
        let theta = core::f64::consts::TAU * u2;
        let z = radius * cos(theta);
        self.mu + self.sigma * z
    }

    fn mean(&self) -> f64 {
        self.mu
    }

    fn variance(&self) -> f64 {
        self.sigma * self.sigma
    }

    fn pdf(&self, x: f64) -> f64 {
        let z = (x - self.mu) / self.sigma;
        FRAC_1_SQRT_2PI / self.sigma * exp(-0.5 * z * z)
    }

    fn cdf(&self, x: f64) -> f64 {
        // Abramowitz & Stegun 7.1.26, accurate to about 7.5e-8.
        let sn = (x - self.mu) / self.sigma;
        let z = sn.abs();
        let t = 1.0 / (1.0 + 0.231_641_9 * z);
        let poly = (((((1.061_405_429 * t - 1.453_152_027) * t) + 1.421_413_741) * t
            - 0.284_496_736)
            * t
            + 0.254_829_592)
            * t;
        let y = 1.0 - 0.5 * poly * exp(-0.5 * z * z);
        if sn >= 0.0 { y } else { 1.0 - y }
    }
}

/// A Bernoulli distribution: a single trial that yields `1` with probability
/// `p` and `0` with probability `1 - p`.
///
/// The mean is `p`, the variance is `p * (1 - p)`, and the PMF is only nonzero
/// at `0` and `1`.
///
/// # Examples
///
/// ```
/// use tpt_zero_dist::{Distribution, Bernoulli};
///
/// let b = Bernoulli::new(0.25).unwrap();
/// assert_eq!(b.mean(), 0.25);
/// assert!((b.variance() - 0.1875).abs() < 1e-12);
/// assert_eq!(b.pmf(1), 0.25);
/// assert_eq!(b.pmf(0), 0.75);
/// assert_eq!(b.pmf(2), 0.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bernoulli {
    p: f64,
}

impl Bernoulli {
    /// Creates a `Bernoulli` distribution with success probability `p`.
    ///
    /// Returns `None` if `p` is outside `[0, 1]` or non-finite.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_dist::Bernoulli;
    ///
    /// assert!(Bernoulli::new(0.5).is_some());
    /// assert!(Bernoulli::new(1.5).is_none());
    /// assert!(Bernoulli::new(f64::NAN).is_none());
    /// ```
    #[must_use]
    pub fn new(p: f64) -> Option<Self> {
        if !p.is_finite() || !(0.0..=1.0).contains(&p) {
            return None;
        }
        Some(Bernoulli { p })
    }

    /// Returns the success probability `p`.
    #[must_use]
    pub fn p(&self) -> f64 {
        self.p
    }
}

impl Distribution for Bernoulli {
    type Value = u64;

    fn sample<R: Rng>(&self, rng: &mut R) -> u64 {
        u64::from(rng.next_f64() < self.p)
    }

    fn mean(&self) -> f64 {
        self.p
    }

    fn variance(&self) -> f64 {
        self.p * (1.0 - self.p)
    }

    fn pmf(&self, k: u64) -> f64 {
        match k {
            0 => 1.0 - self.p,
            1 => self.p,
            _ => 0.0,
        }
    }

    fn cdf(&self, x: f64) -> f64 {
        if x < 0.0 {
            0.0
        } else if x < 1.0 {
            1.0 - self.p
        } else {
            1.0
        }
    }
}

/// A Poisson distribution with rate `lambda > 0`: the number of events in a
/// fixed interval when events occur independently at a constant average rate.
///
/// The mean and variance both equal `lambda`, and the PMF is
/// `lambda^k * e^{-lambda} / k!`.
///
/// # Examples
///
/// ```
/// use tpt_zero_dist::{Distribution, Poisson};
///
/// let p = Poisson::new(3.0).unwrap();
/// assert_eq!(p.mean(), 3.0);
/// assert_eq!(p.variance(), 3.0);
/// assert!((p.pmf(0) - tpt_zero_dist::math::exp(-3.0)).abs() < 1e-9);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Poisson {
    lambda: f64,
}

impl Poisson {
    /// Creates a `Poisson` distribution with rate `lambda`.
    ///
    /// Returns `None` if `lambda <= 0` or is non-finite.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_dist::Poisson;
    ///
    /// assert!(Poisson::new(2.0).is_some());
    /// assert!(Poisson::new(0.0).is_none());
    /// ```
    #[must_use]
    pub fn new(lambda: f64) -> Option<Self> {
        if !(lambda > 0.0) || !lambda.is_finite() {
            return None;
        }
        Some(Poisson { lambda })
    }

    /// Returns the rate parameter `lambda`.
    #[must_use]
    pub fn lambda(&self) -> f64 {
        self.lambda
    }
}

impl Distribution for Poisson {
    type Value = u64;

    fn sample<R: Rng>(&self, rng: &mut R) -> u64 {
        // Knuth's algorithm: multiply uniforms until the running product drops
        // below e^{-lambda}, counting the number of multiplications.
        let threshold = exp(-self.lambda);
        let mut k: u64 = 0;
        let mut product = 1.0;
        loop {
            product *= rng.next_f64();
            if product <= threshold {
                return k;
            }
            k += 1;
        }
    }

    fn mean(&self) -> f64 {
        self.lambda
    }

    fn variance(&self) -> f64 {
        self.lambda
    }

    fn pmf(&self, k: u64) -> f64 {
        // lambda^k * e^{-lambda} / k!, computed iteratively to avoid overflow.
        let mut term = exp(-self.lambda);
        for i in 1..=k {
            term *= self.lambda / (i as f64);
        }
        term
    }

    fn cdf(&self, x: f64) -> f64 {
        if x < 0.0 {
            return 0.0;
        }
        let n = math::floor(x) as u64;
        let mut sum = 0.0;
        for k in 0..=n {
            sum += self.pmf(k);
        }
        sum.min(1.0)
    }
}

// --- Interop with the `tpt-zero-prob` analytic `Distribution` trait. ---
//
// The continuous distributions in this crate also satisfy the shape of
// [`tpt_zero_prob::Distribution`] (analytic `pdf`/`cdf` plus `Option` moments),
// so we implement it for `Uniform` and `Normal`. This lets these distributions
// flow into `tpt-zero-prob` APIs directly.

impl tpt_zero_prob::Distribution for Uniform {
    type Value = f64;

    fn pdf(&self, x: f64) -> f64 {
        <Self as Distribution>::pdf(self, x)
    }

    fn cdf(&self, x: f64) -> f64 {
        <Self as Distribution>::cdf(self, x)
    }

    fn mean(&self) -> Option<f64> {
        Some(<Self as Distribution>::mean(self))
    }

    fn variance(&self) -> Option<f64> {
        Some(<Self as Distribution>::variance(self))
    }
}

impl tpt_zero_prob::Distribution for Normal {
    type Value = f64;

    fn pdf(&self, x: f64) -> f64 {
        <Self as Distribution>::pdf(self, x)
    }

    fn cdf(&self, x: f64) -> f64 {
        <Self as Distribution>::cdf(self, x)
    }

    fn mean(&self) -> Option<f64> {
        Some(<Self as Distribution>::mean(self))
    }

    fn variance(&self) -> Option<f64> {
        Some(<Self as Distribution>::variance(self))
    }
}

/// The empirical mean and variance of a slice of observed samples.
///
/// A thin re-export of [`tpt-zero-stats`](https://docs.rs/tpt-zero-stats) so
/// callers can compare an analytic [`Distribution::mean`] / [`Distribution::variance`]
/// against a batch of draws without pulling in the stats crate directly.
/// Returns `None` for `mean` when the slice is empty, and `None` for
/// `variance` when there are fewer than two samples.
///
/// # Examples
///
/// ```
/// use tpt_zero_dist::{Distribution, Uniform, sample_summary};
/// use tpt_zero_rand::{Pcg32, SeedableRng};
///
/// let u = Uniform::new(0.0, 1.0).unwrap();
/// let mut rng = Pcg32::seed_from_u64(1);
/// let draws: [f64; 8] = core::array::from_fn(|_| u.sample(&mut rng));
/// let (mean, _var) = sample_summary(&draws);
/// assert!(mean.is_some());
/// ```
#[must_use]
pub fn sample_summary(samples: &[f64]) -> (Option<f64>, Option<f64>) {
    (
        tpt_zero_stats::mean(samples),
        tpt_zero_stats::variance(samples),
    )
}

/// Cosine via a range-reduced Taylor series.
///
/// Box–Muller only needs `cos` of an angle in `[0, 2*pi)`, so this reduces the
/// argument modulo `2*pi`, folds it into `[-pi, pi]`, and evaluates the Taylor
/// series. `f64::cos` is not available in this `core`-only crate.
fn cos(x: f64) -> f64 {
    const TAU: f64 = core::f64::consts::TAU;
    const PI: f64 = core::f64::consts::PI;
    // Reduce into [0, TAU).
    let mut a = x - math::floor(x / TAU) * TAU;
    // Fold into [-PI, PI] for best series accuracy.
    if a > PI {
        a -= TAU;
    }
    let a2 = a * a;
    let mut term = 1.0;
    let mut sum = 1.0;
    let mut sign = -1.0;
    let mut n = 0.0;
    for _ in 1..=12 {
        // Advance to the next even power: multiply by a^2 / ((2n-1)(2n)).
        n += 1.0;
        let denom = (2.0 * n - 1.0) * (2.0 * n);
        term *= a2 / denom;
        sum += sign * term;
        sign = -sign;
    }
    sum
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp, clippy::unreadable_literal)]
    use super::*;
    use tpt_zero_rand::{Pcg32, SeedableRng, XorShift64};

    #[test]
    fn uniform_properties() {
        let u = Uniform::new(2.0, 4.0).unwrap();
        assert_eq!(u.mean(), 3.0);
        assert_eq!(u.variance(), 4.0 / 12.0);
        assert!((u.pdf(3.0) - 0.5).abs() < 1e-12);
        assert_eq!(u.cdf(2.0), 0.0);
        assert_eq!(u.cdf(4.0), 1.0);
        assert_eq!(u.pdf(5.0), 0.0);
        assert_eq!(u.low(), 2.0);
        assert_eq!(u.high(), 4.0);
    }

    #[test]
    fn uniform_rejects_invalid() {
        assert!(Uniform::new(1.0, 1.0).is_none());
        assert!(Uniform::new(2.0, 1.0).is_none());
        assert!(Uniform::new(f64::INFINITY, 1.0).is_none());
    }

    #[test]
    fn uniform_samples_in_range() {
        let u = Uniform::new(-3.0, 5.0).unwrap();
        let mut rng = Pcg32::seed_from_u64(1);
        for _ in 0..10_000 {
            let x = u.sample(&mut rng);
            assert!((-3.0..5.0).contains(&x), "sample {x} out of range");
        }
    }

    #[test]
    fn uniform_sample_mean() {
        let u = Uniform::new(0.0, 10.0).unwrap();
        let mut rng = Pcg32::seed_from_u64(7);
        let mut sum = 0.0;
        let n = 100_000;
        for _ in 0..n {
            sum += u.sample(&mut rng);
        }
        let mean = sum / f64::from(n);
        assert!((mean - 5.0).abs() < 0.1, "empirical mean {mean}");
    }

    #[test]
    fn normal_standard() {
        let n = Normal::standard();
        assert_eq!(n.mean(), 0.0);
        assert_eq!(n.variance(), 1.0);
        assert_eq!(n.std_dev(), 1.0);
        assert!((n.cdf(0.0) - 0.5).abs() < 1e-3);
        assert!((n.cdf(1.96) - 0.975).abs() < 1e-2);
        assert!((n.pdf(0.0) - FRAC_1_SQRT_2PI).abs() < 1e-12);
    }

    #[test]
    fn normal_rejects_invalid() {
        assert!(Normal::new(0.0, 0.0).is_none());
        assert!(Normal::new(0.0, -1.0).is_none());
        assert!(Normal::new(f64::NAN, 1.0).is_none());
    }

    #[test]
    fn normal_sample_moments() {
        let n = Normal::new(5.0, 2.0).unwrap();
        let mut rng = XorShift64::seed_from_u64(0x1234);
        let count = 200_000;
        let mut sum = 0.0;
        let mut sumsq = 0.0;
        for _ in 0..count {
            let x = n.sample(&mut rng);
            sum += x;
            sumsq += x * x;
        }
        let mean = sum / f64::from(count);
        let var = sumsq / f64::from(count) - mean * mean;
        assert!((mean - 5.0).abs() < 0.05, "mean {mean}");
        assert!((var - 4.0).abs() < 0.1, "var {var}");
    }

    #[test]
    fn bernoulli_properties() {
        let b = Bernoulli::new(0.3).unwrap();
        assert_eq!(b.mean(), 0.3);
        assert!((b.variance() - 0.21).abs() < 1e-12);
        assert_eq!(b.pmf(0), 0.7);
        assert_eq!(b.pmf(1), 0.3);
        assert_eq!(b.pmf(2), 0.0);
        assert_eq!(b.p(), 0.3);
        assert_eq!(b.cdf(-0.5), 0.0);
        assert_eq!(b.cdf(0.5), 0.7);
        assert_eq!(b.cdf(1.5), 1.0);
    }

    #[test]
    fn bernoulli_rejects_invalid() {
        assert!(Bernoulli::new(-0.1).is_none());
        assert!(Bernoulli::new(1.1).is_none());
        assert!(Bernoulli::new(f64::NAN).is_none());
        assert!(Bernoulli::new(0.0).is_some());
        assert!(Bernoulli::new(1.0).is_some());
    }

    #[test]
    fn bernoulli_samples_binary() {
        let b = Bernoulli::new(0.5).unwrap();
        let mut rng = Pcg32::seed_from_u64(99);
        let mut ones = 0u64;
        let n = 100_000u64;
        for _ in 0..n {
            let s = b.sample(&mut rng);
            assert!(s == 0 || s == 1, "sample {s} not binary");
            ones += s;
        }
        let frac = ones as f64 / n as f64;
        assert!((frac - 0.5).abs() < 0.02, "frac {frac}");
    }

    #[test]
    fn poisson_properties() {
        let p = Poisson::new(4.0).unwrap();
        assert_eq!(p.mean(), 4.0);
        assert_eq!(p.variance(), 4.0);
        assert_eq!(p.lambda(), 4.0);
        // PMF sums to ~1 over a wide range.
        let mut sum = 0.0;
        for k in 0..50 {
            sum += p.pmf(k);
        }
        assert!((sum - 1.0).abs() < 1e-9, "pmf sum {sum}");
        assert_eq!(p.cdf(-1.0), 0.0);
        assert!((p.cdf(100.0) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn poisson_pmf_matches_recurrence() {
        let p = Poisson::new(2.5).unwrap();
        let p0 = exp(-2.5);
        assert!((p.pmf(0) - p0).abs() < 1e-12);
        assert!((p.pmf(1) - p0 * 2.5).abs() < 1e-12);
        assert!((p.pmf(2) - p0 * 2.5 * 2.5 / 2.0).abs() < 1e-12);
    }

    #[test]
    fn poisson_rejects_invalid() {
        assert!(Poisson::new(0.0).is_none());
        assert!(Poisson::new(-1.0).is_none());
        assert!(Poisson::new(f64::INFINITY).is_none());
    }

    #[test]
    fn poisson_sample_mean() {
        let p = Poisson::new(6.0).unwrap();
        let mut rng = Pcg32::seed_from_u64(2024);
        let n = 100_000u64;
        let mut sum = 0.0;
        for _ in 0..n {
            sum += p.sample(&mut rng) as f64;
        }
        let mean = sum / n as f64;
        assert!((mean - 6.0).abs() < 0.1, "poisson mean {mean}");
    }

    #[test]
    fn cos_matches_reference() {
        for i in -20..=20 {
            let x = f64::from(i) * 0.5;
            assert!((cos(x) - x.cos()).abs() < 1e-9, "cos({x})");
        }
    }
}
