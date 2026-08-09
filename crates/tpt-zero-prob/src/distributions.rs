//! Concrete [`Distribution`] implementations: [`Uniform`], [`Normal`], and
//! [`Exponential`].
//!
//! Each distribution stores its parameters and exposes the analytic `pdf`,
//! `cdf`, `mean`, and `variance`. Constructors validate their parameters and
//! return `Option` (or `Result`) so that an ill-formed distribution cannot be
//! constructed.

use crate::Distribution;

/// `1 / sqrt(2 * pi)`, the normalizing constant of the standard normal PDF.
const FRAC_1_SQRT_2PI: f64 = 0.398_942_280_401_432_7;

/// Computes `e^x`, matching `f64::exp`.
///
/// `f64::exp` is not available in this crate's `core`-only target
/// configuration, so we delegate to the shared, subnormal-safe
/// [`tpt_zero_float`] implementation, which handles the full range (including
/// the subnormal underflow region) and never returns a negative value for
/// large negative `x`.
fn exp(x: f64) -> f64 {
    tpt_zero_float::exp(x)
}

/// A continuous uniform distribution `U(a, b)` over the interval `[a, b]`.
///
/// The probability density is constant at `1 / (b - a)` inside the interval
/// and zero outside it; the mean is `(a + b) / 2` and the variance is
/// `(b - a)^2 / 12`.
///
/// # Examples
///
/// ```
/// use tpt_zero_prob::distributions::Uniform;
/// use tpt_zero_prob::Distribution;
///
/// let u = Uniform::new(0.0, 1.0).unwrap();
/// assert_eq!(u.pdf(0.5), 1.0);
/// assert_eq!(u.cdf(0.5), 0.5);
/// assert_eq!(u.mean(), Some(0.5));
/// assert!((u.variance().unwrap() - 1.0 / 12.0).abs() < 1e-12);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Uniform {
    /// Lower bound (inclusive).
    a: f64,
    /// Upper bound (inclusive).
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
    /// use tpt_zero_prob::distributions::Uniform;
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
}

impl Distribution for Uniform {
    type Value = f64;

    fn pdf(&self, x: f64) -> f64 {
        let width = self.b - self.a;
        if x >= self.a && x <= self.b {
            1.0 / width
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

    fn mean(&self) -> Option<f64> {
        Some((self.a + self.b) / 2.0)
    }

    fn variance(&self) -> Option<f64> {
        let width = self.b - self.a;
        Some(width * width / 12.0)
    }
}

/// A normal (Gaussian) distribution `N(mu, sigma^2)` with mean `mu` and
/// standard deviation `sigma > 0`.
///
/// # Examples
///
/// ```
/// use tpt_zero_prob::distributions::Normal;
/// use tpt_zero_prob::Distribution;
///
/// let n = Normal::standard();
/// assert_eq!(n.mean(), Some(0.0));
/// assert_eq!(n.variance(), Some(1.0));
/// assert!((n.pdf(0.0) - 0.398_942_280_401_432_7).abs() < 1e-12);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Normal {
    /// Mean.
    mu: f64,
    /// Standard deviation (always positive after construction).
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
    /// use tpt_zero_prob::distributions::Normal;
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
}

impl Distribution for Normal {
    type Value = f64;

    fn pdf(&self, x: f64) -> f64 {
        let z = (x - self.mu) / self.sigma;
        let inv_sqrt_2pi = FRAC_1_SQRT_2PI;
        inv_sqrt_2pi / self.sigma * exp(-0.5 * z * z)
    }

    fn cdf(&self, x: f64) -> f64 {
        // Standard normal CDF via the Abramowitz & Stegun 7.1.26 approximation,
        // accurate to about 7.5e-8.
        let sn = (x - self.mu) / self.sigma;
        let z = sn.abs();
        let t = 1.0 / (1.0 + 0.231_641_9 * z);
        let poly = (((((1.061_405_429 * t - 1.453_152_027) * t) + 1.421_413_741) * t
            - 0.284_496_736)
            * t
            + 0.254_829_592)
            * t;
        // Abramowitz & Stegun 7.1.26: Φ(z) = 1 - 0.5 * poly * e^{-z^2/2}.
        let y = 1.0 - 0.5 * poly * exp(-0.5 * z * z);
        if sn >= 0.0 {
            y
        } else {
            1.0 - y
        }
    }

    fn mean(&self) -> Option<f64> {
        Some(self.mu)
    }

    fn variance(&self) -> Option<f64> {
        Some(self.sigma * self.sigma)
    }
}

/// An exponential distribution `Exp(lambda)` with rate `lambda > 0`.
///
/// The density is `lambda * exp(-lambda * x)` for `x >= 0` and zero for
/// `x < 0`; the mean is `1 / lambda` and the variance is `1 / lambda^2`.
///
/// # Examples
///
/// ```
/// use tpt_zero_prob::distributions::Exponential;
/// use tpt_zero_prob::Distribution;
///
/// let e = Exponential::new(2.0).unwrap();
/// assert_eq!(e.mean(), Some(0.5));
/// assert_eq!(e.variance(), Some(0.25));
/// assert_eq!(e.cdf(0.0), 0.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Exponential {
    /// Rate parameter (always positive after construction).
    lambda: f64,
}

impl Exponential {
    /// Creates an exponential distribution with rate `lambda`.
    ///
    /// Returns `None` if `lambda <= 0` or if `lambda` is non-finite.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_prob::distributions::Exponential;
    ///
    /// assert!(Exponential::new(1.0).is_some());
    /// assert!(Exponential::new(0.0).is_none());
    /// ```
    #[must_use]
    pub fn new(lambda: f64) -> Option<Self> {
        if !(lambda > 0.0) || !lambda.is_finite() {
            return None;
        }
        Some(Exponential { lambda })
    }
}

impl Distribution for Exponential {
    type Value = f64;

    fn pdf(&self, x: f64) -> f64 {
        if x < 0.0 {
            0.0
        } else {
            self.lambda * exp(-self.lambda * x)
        }
    }

    fn cdf(&self, x: f64) -> f64 {
        if x < 0.0 {
            0.0
        } else {
            1.0 - exp(-self.lambda * x)
        }
    }

    fn mean(&self) -> Option<f64> {
        Some(1.0 / self.lambda)
    }

    fn variance(&self) -> Option<f64> {
        Some(1.0 / (self.lambda * self.lambda))
    }
}

/// A trivial deterministic RNG used only by the test in this module to
/// draw uniform samples without pulling in `rand`.
///
/// It is a small xorshift generator; it exists solely to produce reproducible
/// `f64` draws in `[0, 1)` for empirical-statistic tests and is *not* part of
/// the public API.
#[cfg(test)]
struct TestRng {
    state: u64,
}

#[cfg(test)]
impl TestRng {
    fn new(seed: u64) -> Self {
        TestRng {
            state: seed | 1,
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    fn next_f64(&mut self) -> f64 {
        // 52 bits of mantissa -> [0, 1).
        let bits = self.next_u64() >> 12;
        (bits as f64) / (1u64 << 52) as f64
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::cast_possible_truncation,
        clippy::unreadable_literal,
        clippy::float_cmp
    )]
    use super::*;

    #[test]
    fn uniform_properties() {
        let u = Uniform::new(2.0, 4.0).unwrap();
        assert_eq!(u.mean(), Some(3.0));
        assert_eq!(u.variance(), Some(4.0 / 12.0));
        assert!((u.pdf(3.0) - 0.5).abs() < 1e-12);
        assert!((u.cdf(2.0) - 0.0).abs() < 1e-12);
        assert!((u.cdf(4.0) - 1.0).abs() < 1e-12);
        assert!((u.pdf(5.0) - 0.0).abs() < 1e-12);
    }

    #[test]
    fn uniform_rejects_invalid() {
        assert!(Uniform::new(1.0, 1.0).is_none());
        assert!(Uniform::new(2.0, 1.0).is_none());
        assert!(Uniform::new(f64::INFINITY, 1.0).is_none());
    }

    #[test]
    fn normal_standard() {
        let n = Normal::standard();
        assert_eq!(n.mean(), Some(0.0));
        assert_eq!(n.variance(), Some(1.0));
        assert!((n.cdf(0.0) - 0.5).abs() < 1e-3);
        assert!((n.cdf(1.96) - 0.975).abs() < 1e-2);
    }

    #[test]
    fn normal_shift_scale() {
        let n = Normal::new(10.0, 2.0).unwrap();
        assert_eq!(n.mean(), Some(10.0));
        assert_eq!(n.variance(), Some(4.0));
        assert!((n.cdf(10.0) - 0.5).abs() < 1e-3);
    }

    #[test]
    fn normal_rejects_invalid() {
        assert!(Normal::new(0.0, 0.0).is_none());
        assert!(Normal::new(0.0, -1.0).is_none());
    }

    #[test]
    fn exponential_properties() {
        let e = Exponential::new(3.0).unwrap();
        assert_eq!(e.mean(), Some(1.0 / 3.0));
        assert_eq!(e.variance(), Some(1.0 / 9.0));
        assert!((e.cdf(0.0) - 0.0).abs() < 1e-12);
        assert!((e.cdf(1.0) - (1.0 - (-3.0f64).exp())).abs() < 1e-12);
        assert!((e.pdf(-1.0) - 0.0).abs() < 1e-12);
    }

    #[test]
    fn proptest_empirical_mean_of_uniform() {
        // Draw many uniform samples and check the empirical mean is close to
        // the analytic mean (0.5 for U(0,1)). This is an empirical check, not
        // a strict law-of-large-numbers proof, so a tolerance is used.
        let mut rng = TestRng::new(0x9E37_79B9_7F4A_7C15);
        let mut buf = [0.0f64; 1500];
        for x in &mut buf {
            *x = rng.next_f64();
        }
        let dist = crate::Dist::new(&buf);
        let mean = dist.empirical_mean().unwrap();
        assert!((mean - 0.5).abs() < 0.05, "empirical mean {mean} too far from 0.5");
        let var = dist.empirical_variance().unwrap();
        assert!((var - 1.0 / 12.0).abs() < 0.05, "empirical var {var} too far from 1/12");
    }
}
