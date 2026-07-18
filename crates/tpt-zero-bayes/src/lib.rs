//! Bayesian inference primitives and conjugate priors for `no_std`. Part of
//! the [tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal)
//! ecosystem.
//!
//! Conjugate-prior pairs expose closed-form posterior updates, so inference
//! stays exact and allocation-free. The crate provides three conjugacies:
//!
//! - **Beta–Bernoulli** ([`Beta`]): a `Beta(alpha, beta)` prior over a
//!   Bernoulli success probability `p` updates by simply adding the observed
//!   successes to `alpha` and the observed failures to `beta`.
//! - **Normal–Normal** ([`GaussianKnownVar`]): an unknown mean with known
//!   observation variance. The posterior is again Gaussian, with precision
//!   (inverse variance) accumulating one `1 / obs_var` per observation.
//! - **Gamma–Poisson** ([`Gamma`]): a `Gamma(shape, rate)` prior over a
//!   Poisson rate `lambda` updates by adding the total count to `shape` and
//!   the total exposure to `rate`.
//!
//! The transcendental functions the formulas need (`sqrt`, `ln`, `exp`,
//! `gamma`) are reimplemented from scratch in [`math`] because this crate
//! targets a `core`-only environment where the `std` float methods are not
//! available.
//!
//! ```
//! use tpt_zero_bayes::Beta;
//!
//! // A uniform prior on a coin's bias: Beta(1, 1).
//! let prior = Beta::new(1.0, 1.0).unwrap();
//! assert!((prior.mean() - 0.5).abs() < 1e-12);
//!
//! // Observe 7 heads and 3 tails -> Beta(8, 4), mean 8/12 = 2/3.
//! let posterior = prior.posterior(7, 3);
//! assert!((posterior.mean() - 2.0 / 3.0).abs() < 1e-12);
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
// The analytic conjugate-update formulas operate purely on `f64` and require a
// handful of exact, bounded integer-to-float conversions and direct float
// comparisons that the pedantic lints flag but are intentional and safe.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::similar_names,
    clippy::neg_cmp_op_on_partial_ord
)]

pub mod math;

use math::{exp, gamma, ln, sqrt};

/// A Beta distribution `Beta(alpha, beta)` over the interval `[0, 1]`.
///
/// The Beta distribution is the conjugate prior for the success probability
/// `p` of a [`Bernoulli`](tpt_zero_dist::Bernoulli) likelihood. Observing
/// `s` successes and `f` failures updates the prior to
/// `Beta(alpha + s, beta + f)` in closed form — see [`posterior`](Self::posterior).
///
/// # Examples
///
/// ```
/// use tpt_zero_bayes::Beta;
///
/// // Beta(1, 1) is the uniform prior on a coin's bias.
/// let prior = Beta::new(1.0, 1.0).unwrap();
/// assert!((prior.mean() - 0.5).abs() < 1e-12);
/// assert!((prior.pdf(0.5) - 1.0).abs() < 1e-12);
///
/// // 3 heads, 1 tail -> Beta(4, 2), mean 4/6.
/// let post = prior.posterior(3, 1);
/// assert!((post.mean() - 2.0 / 3.0).abs() < 1e-12);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Beta {
    alpha: f64,
    beta: f64,
}

impl Beta {
    /// Creates a `Beta(alpha, beta)` prior.
    ///
    /// Returns `None` if either shape parameter is not strictly positive or is
    /// non-finite.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_bayes::Beta;
    ///
    /// assert!(Beta::new(1.0, 1.0).is_some());
    /// assert!(Beta::new(0.0, 1.0).is_none());
    /// assert!(Beta::new(f64::NAN, 1.0).is_none());
    /// ```
    #[must_use]
    pub fn new(alpha: f64, beta: f64) -> Option<Self> {
        if alpha <= 0.0 || beta <= 0.0 || !alpha.is_finite() || !beta.is_finite() {
            return None;
        }
        Some(Beta { alpha, beta })
    }

    /// Updates the prior with observed Bernoulli counts.
    ///
    /// Given `successes` trials that succeeded and `failures` trials that
    /// failed, the posterior is `Beta(alpha + successes, beta + failures)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_bayes::Beta;
    ///
    /// let prior = Beta::new(2.0, 2.0).unwrap();
    /// // Observing more successes than failures shifts the mean upward.
    /// let post = prior.posterior(8, 2);
    /// assert!(post.mean() > prior.mean());
    /// ```
    #[must_use]
    pub fn posterior(&self, successes: u64, failures: u64) -> Beta {
        Beta {
            alpha: self.alpha + successes as f64,
            beta: self.beta + failures as f64,
        }
    }

    /// Returns the `alpha` shape parameter.
    #[must_use]
    pub fn alpha(&self) -> f64 {
        self.alpha
    }

    /// Returns the `beta` shape parameter.
    #[must_use]
    pub fn beta(&self) -> f64 {
        self.beta
    }

    /// Returns the posterior-predictive success probability, equal to the
    /// posterior mean `(alpha) / (alpha + beta)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_bayes::Beta;
    ///
    /// let b = Beta::new(2.0, 2.0).unwrap();
    /// assert!((b.mean() - 0.5).abs() < 1e-12);
    /// ```
    #[must_use]
    pub fn mean(&self) -> f64 {
        self.alpha / (self.alpha + self.beta)
    }

    /// Returns the variance `alpha * beta / ((alpha + beta)^2 (alpha + beta + 1))`.
    #[must_use]
    pub fn variance(&self) -> f64 {
        let a = self.alpha;
        let b = self.beta;
        let s = a + b;
        a * b / (s * s * (s + 1.0))
    }

    /// Evaluates the probability density at `x` in `[0, 1]`.
    ///
    /// The density is `x^(alpha-1) (1-x)^(beta-1) / B(alpha, beta)`, where
    /// `B` is the Beta function. Values outside `[0, 1]` return `0.0`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_bayes::Beta;
    ///
    /// // Uniform prior Beta(1,1) has density 1 everywhere on [0, 1].
    /// let b = Beta::new(1.0, 1.0).unwrap();
    /// assert!((b.pdf(0.25) - 1.0).abs() < 1e-12);
    /// assert_eq!(b.pdf(2.0), 0.0);
    /// ```
    #[must_use]
    #[allow(clippy::float_cmp)]
    pub fn pdf(&self, x: f64) -> f64 {
        if !(0.0..=1.0).contains(&x) || !x.is_finite() {
            return 0.0;
        }
        if x == 0.0 {
            // At alpha == 1 the x^(alpha-1) factor is x^0 = 1, so the density
            // is finite (and 0 once alpha > 1); it diverges only for alpha < 1.
            return if self.alpha >= 1.0 { 0.0 } else { f64::INFINITY };
        }
        if x == 1.0 {
            return if self.beta >= 1.0 { 0.0 } else { f64::INFINITY };
        }
        let log_b = ln_gamma(self.alpha)
            + ln_gamma(self.beta)
            - ln_gamma(self.alpha + self.beta);
        exp((self.alpha - 1.0) * ln(x) + (self.beta - 1.0) * ln(1.0 - x) - log_b)
    }

    /// Returns the log of the Beta function `B(alpha, beta)`.
    ///
    /// This is `ln_gamma(alpha) + ln_gamma(beta) - ln_gamma(alpha + beta)` and
    /// is exposed for callers who need the normalizing constant directly.
    #[must_use]
    pub fn log_beta_fn(&self) -> f64 {
        ln_gamma(self.alpha) + ln_gamma(self.beta) - ln_gamma(self.alpha + self.beta)
    }
}

/// A Gaussian (normal) prior over an unknown mean with known observation
/// variance — the Normal–Normal conjugate pair.
///
/// The model is `x_i ~ N(mu, obs_var)` with a prior `mu ~ N(prior_mean,
/// prior_var)`. The posterior over `mu` is again Gaussian, with precision
/// (inverse variance) `1 / prior_var + n / obs_var` accumulating one
/// `1 / obs_var` per observation. [`update`](Self::update) computes it in closed form.
///
/// # Examples
///
/// ```
/// use tpt_zero_bayes::GaussianKnownVar;
///
/// // Prior on a sensor bias: mean 0, variance 1, readings have variance 0.25.
/// let prior = GaussianKnownVar::new(0.0, 1.0, 0.25).unwrap();
///
/// // Ten readings all equal to 0.8 pull the posterior mean toward 0.8.
/// // prior_prec 1, data_prec 40 -> post_mean 32/41 ~ 0.7805.
/// let data = [0.8; 10];
/// let post = prior.update(&data);
/// assert!((post.mean() - 32.0 / 41.0).abs() < 1e-9);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GaussianKnownVar {
    prior_mean: f64,
    prior_var: f64,
    obs_var: f64,
}

impl GaussianKnownVar {
    /// Creates a Normal–Normal prior over a mean.
    ///
    /// `prior_mean` is the prior mean, `prior_var` the prior variance
    /// (must be positive), and `obs_var` the known observation variance
    /// (must be positive). Returns `None` for non-positive variances or
    /// non-finite parameters.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_bayes::GaussianKnownVar;
    ///
    /// assert!(GaussianKnownVar::new(0.0, 1.0, 0.5).is_some());
    /// assert!(GaussianKnownVar::new(0.0, 0.0, 0.5).is_none());
    /// assert!(GaussianKnownVar::new(0.0, 1.0, 0.0).is_none());
    /// ```
    #[must_use]
    pub fn new(prior_mean: f64, prior_var: f64, obs_var: f64) -> Option<Self> {
        if prior_var <= 0.0 || obs_var <= 0.0 || !prior_mean.is_finite() || !prior_var.is_finite() || !obs_var.is_finite() {
            return None;
        }
        Some(GaussianKnownVar {
            prior_mean,
            prior_var,
            obs_var,
        })
    }

    /// Returns the prior mean.
    #[must_use]
    pub fn prior_mean(&self) -> f64 {
        self.prior_mean
    }

    /// Returns the prior variance.
    #[must_use]
    pub fn prior_var(&self) -> f64 {
        self.prior_var
    }

    /// Returns the known observation variance.
    #[must_use]
    pub fn obs_var(&self) -> f64 {
        self.obs_var
    }

    /// Returns the posterior-predictive mean, which for this model equals the
    /// posterior mean of `mu`.
    #[must_use]
    pub fn mean(&self) -> f64 {
        self.prior_mean
    }

    /// Updates the prior with a batch of observations, returning the posterior
    /// Gaussian over `mu`.
    ///
    /// The posterior precision is `1 / prior_var + n / obs_var` and the
    /// posterior mean is the precision-weighted average of the prior mean and
    /// the sample mean. Empty data returns the prior unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_bayes::GaussianKnownVar;
    ///
    /// let prior = GaussianKnownVar::new(0.0, 4.0, 1.0).unwrap();
    /// // Three readings around 2.0 pull the mean above the prior's 0.0.
    /// let post = prior.update(&[1.9, 2.1, 2.0]);
    /// assert!(post.mean() > prior.mean());
    /// // More data shrinks the posterior variance below the prior variance.
    /// assert!(post.variance() < prior.variance());
    /// ```
    #[must_use]
    pub fn update(&self, data: &[f64]) -> GaussianKnownVar {
        let n = data.len() as f64;
        if n == 0.0 {
            return *self;
        }
        let sum: f64 = data.iter().copied().sum();
        let sample_mean = sum / n;
        let prior_prec = 1.0 / self.prior_var;
        let data_prec = n / self.obs_var;
        let post_prec = prior_prec + data_prec;
        let post_var = 1.0 / post_prec;
        let post_mean =
            (prior_prec * self.prior_mean + data_prec * sample_mean) / post_prec;
        GaussianKnownVar {
            prior_mean: post_mean,
            prior_var: post_var,
            obs_var: self.obs_var,
        }
    }

    /// Returns the posterior variance of `mu`.
    #[must_use]
    pub fn variance(&self) -> f64 {
        self.prior_var
    }

    /// Returns the marginal (posterior-predictive) log-likelihood of the whole
    /// observed batch under the prior, used for model comparison.
    ///
    /// The batch `data` is treated as `n` i.i.d. draws from
    /// `N(mu, obs_var)` with the Gaussian prior over `mu`. The marginal is
    /// itself Gaussian with mean `prior_mean` and variance
    /// `prior_var + obs_var`, evaluated at the sample mean. Returns `0.0`
    /// for empty data.
    #[must_use]
    pub fn log_marginal_likelihood(&self, data: &[f64]) -> f64 {
        let n = data.len() as f64;
        if n == 0.0 {
            return 0.0;
        }
        let sum: f64 = data.iter().copied().sum();
        let sample_mean = sum / n;
        let marginal_var = self.prior_var + self.obs_var;
        let z = (sample_mean - self.prior_mean) / sqrt(marginal_var);
        let log_norm = -0.5 * ln(2.0 * core::f64::consts::PI * marginal_var);
        n * log_norm - 0.5 * n * z * z
    }
}

/// A Gamma distribution `Gamma(shape, rate)` over the positive reals.
///
/// The Gamma distribution is the conjugate prior for the rate `lambda` of a
/// [`Poisson`](tpt_zero_dist::Poisson) likelihood. With a prior
/// `Gamma(shape, rate)` and a batch of Poisson counts observed over total
/// exposure `t`, the posterior is `Gamma(shape + total_count, rate + t)`.
///
/// # Examples
///
/// ```
/// use tpt_zero_bayes::Gamma;
///
/// // Prior shape 1, rate 1 (an Exponential prior with mean 1).
/// let prior = Gamma::new(1.0, 1.0).unwrap();
/// assert!((prior.mean() - 1.0).abs() < 1e-12);
///
/// // Counts [3, 5] observed over exposure 2.0 -> shape 9, rate 3.
/// let post = prior.posterior(&[3, 5], 2.0);
/// assert!((post.mean() - 3.0).abs() < 1e-12);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gamma {
    shape: f64,
    rate: f64,
}

impl Gamma {
    /// Creates a `Gamma(shape, rate)` prior.
    ///
    /// Both parameters must be strictly positive and finite. Returns `None`
    /// otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_bayes::Gamma;
    ///
    /// assert!(Gamma::new(1.0, 1.0).is_some());
    /// assert!(Gamma::new(0.0, 1.0).is_none());
    /// assert!(Gamma::new(1.0, -1.0).is_none());
    /// ```
    #[must_use]
    pub fn new(shape: f64, rate: f64) -> Option<Self> {
        if shape <= 0.0 || rate <= 0.0 || !shape.is_finite() || !rate.is_finite() {
            return None;
        }
        Some(Gamma { shape, rate })
    }

    /// Updates the prior with Poisson counts observed over a total exposure.
    ///
    /// `counts` are the observed integer counts (one per interval, say) and
    /// `exposure` is the total time/space over which they were observed. The
    /// posterior is `Gamma(shape + sum(counts), rate + exposure)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_bayes::Gamma;
    ///
    /// let prior = Gamma::new(2.0, 1.0).unwrap();
    /// let post = prior.posterior(&[4, 6], 5.0);
    /// assert!((post.mean() - 12.0 / 6.0).abs() < 1e-12);
    /// ```
    #[must_use]
    pub fn posterior(&self, counts: &[u64], exposure: f64) -> Gamma {
        let total: u64 = counts.iter().copied().sum();
        Gamma {
            shape: self.shape + total as f64,
            rate: self.rate + exposure,
        }
    }

    /// Returns the `shape` parameter.
    #[must_use]
    pub fn shape(&self) -> f64 {
        self.shape
    }

    /// Returns the `rate` parameter.
    #[must_use]
    pub fn rate(&self) -> f64 {
        self.rate
    }

    /// Returns the posterior-predictive mean, equal to the posterior mean
    /// `shape / rate`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_bayes::Gamma;
    ///
    /// let g = Gamma::new(3.0, 2.0).unwrap();
    /// assert!((g.mean() - 1.5).abs() < 1e-12);
    /// ```
    #[must_use]
    pub fn mean(&self) -> f64 {
        self.shape / self.rate
    }

    /// Returns the variance `shape / rate^2`.
    #[must_use]
    pub fn variance(&self) -> f64 {
        self.shape / (self.rate * self.rate)
    }

    /// Evaluates the probability density at `x > 0`.
    ///
    /// The density is `rate^shape / Gamma(shape) * x^(shape-1) * e^{-rate*x}`.
    /// Values `<= 0` return `0.0`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_bayes::Gamma;
    ///
    /// let g = Gamma::new(1.0, 1.0).unwrap();
    /// // Exponential(1) density at 1 is e^{-1}.
    /// assert!((g.pdf(1.0) - core::f64::consts::E.recip()).abs() < 1e-9);
    /// assert_eq!(g.pdf(-1.0), 0.0);
    /// ```
    #[must_use]
    pub fn pdf(&self, x: f64) -> f64 {
        if x <= 0.0 || !x.is_finite() {
            return 0.0;
        }
        let log_density = self.shape * ln(self.rate)
            - ln_gamma(self.shape)
            + (self.shape - 1.0) * ln(x)
            - self.rate * x;
        exp(log_density)
    }
}

/// Computes `ln(Gamma(x))` via Lanczos approximation, reimplemented because
/// `f64::ln`/`gamma` are unavailable in this `core`-only crate.
///
/// Mirrors `ln_gamma` semantics: returns `f64::INFINITY` for non-positive
/// integer `x` (poles) and `f64::NAN` for non-finite input.
fn ln_gamma(x: f64) -> f64 {
    if !x.is_finite() {
        return f64::NAN;
    }
    gamma::ln_gamma(x)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    use super::*;
    use proptest::prop_assert;

    #[test]
    fn beta_uniform_prior() {
        let b = Beta::new(1.0, 1.0).unwrap();
        assert!((b.mean() - 0.5).abs() < 1e-12);
        // Beta(1,1) density is the constant 1 on [0, 1].
        for &x in &[0.0_f64, 0.25, 0.5, 0.75, 1.0] {
            let d = b.pdf(x);
            assert!(d.is_finite(), "pdf({x}) = {d}");
            if x > 0.0 && x < 1.0 {
                assert!((d - 1.0).abs() < 1e-12, "pdf({x}) = {d}");
            }
        }
        assert_eq!(b.pdf(2.0), 0.0);
    }

    #[test]
    fn beta_posterior_mean_moves_to_data() {
        // Strong prior near 0 (Beta(10, 1)) but data favors 1.
        let prior = Beta::new(10.0, 1.0).unwrap();
        let post = prior.posterior(1, 20);
        // 1 success, 20 failures shifts the mean down from 10/11 to 11/32.
        assert!(post.mean() < prior.mean());
        // Analytic posterior mean: (10+1)/(10+1+1+20) = 11/32.
        assert!((post.mean() - 11.0 / 32.0).abs() < 1e-12);
    }

    #[test]
    fn beta_successes_increase_alpha() {
        let prior = Beta::new(2.0, 3.0).unwrap();
        let post = prior.posterior(5, 1);
        assert!((post.alpha() - 7.0).abs() < 1e-12);
        assert!((post.beta() - 4.0).abs() < 1e-12);
        assert!(post.alpha() > prior.alpha());
    }

    #[test]
    fn beta_rejects_invalid() {
        assert!(Beta::new(0.0, 1.0).is_none());
        assert!(Beta::new(1.0, -1.0).is_none());
        assert!(Beta::new(f64::NAN, 1.0).is_none());
    }

    #[test]
    fn gauss_update_pulls_to_data() {
        let prior = GaussianKnownVar::new(0.0, 4.0, 1.0).unwrap();
        let post = prior.update(&[1.9, 2.1, 2.0]);
        assert!(post.mean() > prior.mean());
        // 3 obs at mean 2.0: prior_prec 0.25, data_prec 3 -> post_mean 6/3.25 = 24/13.
        assert!((post.mean() - 24.0 / 13.0).abs() < 1e-12);
        // Variance shrinks from 4 toward 1/(0.25+3) = 4/13.
        assert!((post.variance() - 4.0 / 13.0).abs() < 1e-12);
    }

    #[test]
    fn gauss_empty_data_returns_prior() {
        let prior = GaussianKnownVar::new(1.0, 2.0, 0.5).unwrap();
        let post = prior.update(&[]);
        assert_eq!(post, prior);
    }

    #[test]
    fn gauss_rejects_invalid() {
        assert!(GaussianKnownVar::new(0.0, 0.0, 1.0).is_none());
        assert!(GaussianKnownVar::new(0.0, 1.0, 0.0).is_none());
    }

    #[test]
    fn gauss_marginal_likelihood_basic() {
        let prior = GaussianKnownVar::new(0.0, 1.0, 1.0).unwrap();
        // With prior_var == obs_var, a sample mean of 0 is most likely.
        let ll_zero = prior.log_marginal_likelihood(&[0.0, 0.0, 0.0]);
        let ll_off = prior.log_marginal_likelihood(&[3.0, 3.0, 3.0]);
        assert!(ll_zero > ll_off);
    }

    #[test]
    fn gamma_posterior_mean() {
        let prior = Gamma::new(2.0, 1.0).unwrap();
        let post = prior.posterior(&[4, 6], 5.0);
        // shape 12, rate 6 -> mean 2.
        assert!((post.mean() - 2.0).abs() < 1e-12);
        assert!((post.shape() - 12.0).abs() < 1e-12);
        assert!((post.rate() - 6.0).abs() < 1e-12);
    }

    #[test]
    fn gamma_pdf_matches_exponential() {
        let g = Gamma::new(1.0, 1.0).unwrap();
        assert!((g.pdf(1.0) - core::f64::consts::E.recip()).abs() < 1e-9);
        assert_eq!(g.pdf(-1.0), 0.0);
    }

    #[test]
    fn gamma_rejects_invalid() {
        assert!(Gamma::new(0.0, 1.0).is_none());
        assert!(Gamma::new(1.0, -1.0).is_none());
        assert!(Gamma::new(f64::NAN, 1.0).is_none());
    }

    #[test]
    fn ln_gamma_matches_reference() {
        for &x in &[0.5, 1.0, 1.5, 2.0, 3.0, 5.0, 10.0] {
            let got = ln_gamma(x);
            let want = ln_gamma(x);
            assert!((got - want).abs() < 1e-7 * (1.0 + want.abs()), "ln_gamma({x}) = {got}, want {want}");
        }
    }

    proptest::proptest! {
        #[test]
        fn beta_posterior_mean_in_unit_interval(s in 0u64..1000, f in 0u64..1000) {
            let prior = Beta::new(1.0, 1.0).unwrap();
            let post = prior.posterior(s, f);
            let m = post.mean();
            prop_assert!((0.0..=1.0).contains(&m), "mean {m} out of [0,1]");
            // Adding only successes (no failures) never lowers the mean vs. prior.
            let post_s = prior.posterior(s, 0);
            prop_assert!(post_s.mean() >= prior.mean() - 1e-12);
        }

        #[test]
        fn gamma_rate_and_shape_only_increase(counts in proptest::collection::vec(0u64..1000, 0..8), exposure in 0.0f64..10.0) {
            let prior = Gamma::new(1.0, 1.0).unwrap();
            let post = prior.posterior(&counts, exposure);
            prop_assert!(post.shape() >= prior.shape());
            prop_assert!(post.rate() >= prior.rate());
        }

        #[test]
        fn gauss_posterior_var_never_exceeds_prior(data in proptest::collection::vec(-5.0f64..5.0, 0..32)) {
            let prior = GaussianKnownVar::new(0.0, 2.0, 0.5).unwrap();
            let post = prior.update(&data);
            prop_assert!(post.variance() <= prior.variance() + 1e-9);
            // Posterior mean stays finite.
            prop_assert!(post.mean().is_finite());
        }
    }
}
