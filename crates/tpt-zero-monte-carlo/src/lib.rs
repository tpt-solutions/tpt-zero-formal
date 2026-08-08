#![doc = include_str!("../README.md")]

#![no_std]
#![warn(missing_docs)]
#![forbid(unsafe_code)]

// This crate's `core`-only target does not expose `f64::sqrt`, so we provide
// our own. The implementation necessarily converts between `usize`/`i64` and
// `f64` to index repeated-refinement loops; the values are small bounded
// iteration counts, so the cast lints are allowed here.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]

use tpt_zero_rand::Rng;
use tpt_zero_stats::{mean, variance};

/// Estimates the integral of `f` over the unit interval `[0, 1]` by plain
/// Monte Carlo sampling.
///
/// Each sample `x` is drawn uniformly from `[0, 1)` via `rng.next_f64()` and
/// the estimator is the arithmetic mean of `f(x)` over the `n` draws:
///
/// ```text
/// I â‰ˆ (1/n) Î£_{i=1}^n f(x_i)
/// ```
///
/// The error of this estimator is `O(1/âˆšn)` and independent of the dimension
/// of the domain, which is the defining advantage of Monte Carlo integration.
///
/// # Type parameters
///
/// * `R` â€” the random source, any [`Rng`].
/// * `F` â€” the integrand `Fn(f64) -> f64`.
///
/// # Examples
///
/// ```
/// use tpt_zero_rand::{Pcg32, Rng, SeedableRng};
/// use tpt_zero_monte_carlo::monte_carlo_integral;
///
/// let mut rng = Pcg32::seed_from_u64(1);
/// // âˆ«_0^1 x^2 dx = 1/3
/// let est = monte_carlo_integral(|x| x * x, &mut rng, 20_000);
/// assert!((est - 1.0 / 3.0).abs() < 0.02);
/// ```
#[must_use]
pub fn monte_carlo_integral<R, F>(f: F, rng: &mut R, n: usize) -> f64
where
    R: Rng,
    F: Fn(f64) -> f64,
{
    if n == 0 {
        return 0.0;
    }
    let mut sum = 0.0;
    for _ in 0..n {
        let x = rng.next_f64();
        sum += f(x);
    }
    sum / (n as f64)
}

/// Returns the sample mean of `samples` and the standard error of that mean.
///
/// The standard error of the mean (SEM) is the estimated standard deviation of
/// the sampling distribution of the mean:
///
/// ```text
/// SEM = std_dev(samples) / sqrt(n)
/// ```
///
/// It quantifies how far the mean of `samples` is likely to be from the true
/// mean, and shrinks like `1/âˆšn` as the sample size grows. A `None` standard
/// deviation (fewer than two samples) yields a standard error of `0.0`.
///
/// This delegates the mean and standard deviation to
/// [`tpt-zero-stats`](https://docs.rs/tpt-zero-stats) and provides its own
/// `sqrt`.
///
/// # Examples
///
/// ```
/// use tpt_zero_rand::{Pcg32, Rng, SeedableRng};
/// use tpt_zero_monte_carlo::{estimate_mean_with_error, monte_carlo_integral};
///
/// let mut rng = Pcg32::seed_from_u64(3);
/// let mut samples = [0.0f64; 5_000];
/// for s in &mut samples {
///     *s = monte_carlo_integral(|x| x, &mut rng, 100);
/// }
/// let (mean, sem) = estimate_mean_with_error(&samples);
/// assert!((mean - 0.5).abs() < 0.05);
/// assert!(sem > 0.0 && sem < 0.05);
/// ```
#[must_use]
pub fn estimate_mean_with_error(samples: &[f64]) -> (f64, f64) {
    let mean = mean(samples).unwrap_or(0.0);
    let sem = match variance(samples) {
        Some(v) if v >= 0.0 => sqrt(v / (samples.len() as f64)),
        _ => 0.0,
    };
    (mean, sem)
}

/// Estimates `âˆ«_0^1 f` using antithetic variates, a variance-reduction
/// technique.
///
/// For each pair, one uniform draw `x` produces two evaluations `f(x)` and
/// `f(1 - x)` whose negative correlation cancels part of the estimator
/// variance. The estimate is the mean of `(f(x) + f(1 - x)) / 2` over the `n`
/// draws:
///
/// ```text
/// I â‰ˆ (1/n) Î£_{i=1}^n (f(x_i) + f(1 - x_i)) / 2
/// ```
///
/// Antithetic variates are most effective for monotone integrands, where the
/// variance reduction is strongest. The function `f` is evaluated exactly `n`
/// times (one draw per iteration, two evaluations).
///
/// # Type parameters
///
/// * `R` â€” the random source, any [`Rng`].
/// * `F` â€” the integrand `Fn(f64) -> f64`.
///
/// # Examples
///
/// ```
/// use tpt_zero_rand::{Pcg32, Rng, SeedableRng};
/// use tpt_zero_monte_carlo::antithetic_estimate;
///
/// let mut rng = Pcg32::seed_from_u64(2);
/// // âˆ«_0^1 e^x dx = e - 1
/// let est = antithetic_estimate(|x| x.exp(), &mut rng, 20_000);
/// assert!((est - (core::f64::consts::E - 1.0)).abs() < 0.02);
/// ```
#[must_use]
pub fn antithetic_estimate<R, F>(f: F, rng: &mut R, n: usize) -> f64
where
    R: Rng,
    F: Fn(f64) -> f64,
{
    if n == 0 {
        return 0.0;
    }
    let mut sum = 0.0;
    for _ in 0..n {
        let x = rng.next_f64();
        sum += 0.5 * (f(x) + f(1.0 - x));
    }
    sum / (n as f64)
}

/// Estimates `âˆ«_0^1 f` using control variates, a variance-reduction technique.
///
/// A *control* function `g` with known integral `g_int` over `[0, 1]` is used
/// to reduce variance. The corrected estimator subtracts the (mean-zero)
/// control's empirical integral from the plain estimate of `f`:
///
/// ```text
/// I â‰ˆ (mean of f(x_i)) - (mean of g(x_i) - g_int)
///   =  mean of (f(x_i) - g(x_i))  + g_int
/// ```
///
/// Because `âˆ«_0^1 (f - g) = âˆ«_0^1 f - g_int`, the corrected estimator is still
/// unbiased. When `g` is correlated with `f`, the variance of `f - g` is
/// smaller than that of `f`, which is the whole point. The function `f` and the
/// control `g` are each evaluated exactly `n` times.
///
/// # Type parameters
///
/// * `R` â€” the random source, any [`Rng`].
/// * `F` â€” the integrand `Fn(f64) -> f64`.
/// * `G` â€” the control variate `Fn(f64) -> f64`.
///
/// # Examples
///
/// ```
/// use tpt_zero_rand::{Pcg32, Rng, SeedableRng};
/// use tpt_zero_monte_carlo::control_variate_estimate;
///
/// let mut rng = Pcg32::seed_from_u64(4);
/// // Estimate âˆ« x^2 dx; use g(x) = x with known âˆ«_0^1 x dx = 0.5 as control.
/// let est = control_variate_estimate(|x| x * x, |x| x, 0.5, &mut rng, 20_000);
/// assert!((est - 1.0 / 3.0).abs() < 0.02);
/// ```
#[must_use]
pub fn control_variate_estimate<R, F, G>(
    f: F,
    g: G,
    g_int: f64,
    rng: &mut R,
    n: usize,
) -> f64
where
    R: Rng,
    F: Fn(f64) -> f64,
    G: Fn(f64) -> f64,
{
    if n == 0 {
        return 0.0;
    }
    let mut sum_f = 0.0;
    let mut sum_g = 0.0;
    for _ in 0..n {
        let x = rng.next_f64();
        sum_f += f(x);
        sum_g += g(x);
    }
    (sum_f - sum_g) / (n as f64) + g_int
}

/// Measures the variance of the Monte Carlo estimator of `âˆ«_0^1 f` by running
/// `n` independent replications.
///
/// Each replication is a fresh estimate from `reps` samples. The returned
/// variance describes how much the estimator itself fluctuates across runs; in
/// expectation it equals `Var(f) / reps`, shrinking like `1/reps` (equivalently
/// like `1/n^2` combined with the per-sample `1/âˆšreps`). A single replication
/// yields a variance of `0.0`.
///
/// The variance is accumulated online (Welford's algorithm), so it needs no
/// heap allocation and works in the `default` (`no_std`) configuration.
///
/// # Type parameters
///
/// * `R` â€” the random source, any [`Rng`].
/// * `F` â€” the integrand `Fn(f64) -> f64`, required to be `Copy`.
///
/// # Examples
///
/// ```
/// use tpt_zero_rand::{Pcg32, SeedableRng};
/// use tpt_zero_monte_carlo::sample_mean_variance;
///
/// let mut rng = Pcg32::seed_from_u64(5);
/// // âˆ«_0^1 x dx over 200 replications of 500 samples each.
/// let (mean, var) = sample_mean_variance(|x| x, &mut rng, 200, 500);
/// assert!((mean - 0.5).abs() < 0.05);
/// assert!(var > 0.0 && var < 1.0);
/// ```
#[must_use]
pub fn sample_mean_variance<R, F>(f: F, rng: &mut R, n: usize, reps: usize) -> (f64, f64)
where
    R: Rng,
    F: Fn(f64) -> f64 + Copy,
{
    if n == 0 {
        return (0.0, 0.0);
    }
    let mut mean_acc = 0.0;
    let mut m2 = 0.0;
    for i in 0..n {
        let est = monte_carlo_integral(f, rng, reps);
        let count = (i + 1) as f64;
        let delta = est - mean_acc;
        mean_acc += delta / count;
        m2 += delta * (est - mean_acc);
    }
    let var = if n > 1 { m2 / ((n - 1) as f64) } else { 0.0 };
    (mean_acc, var)
}

/// Computes the square root of `x >= 0.0`.
///
/// Returns `f64::NAN` for negative inputs and for `f64::NAN`. Delegates to the
/// shared, subnormal-safe [`out_zero_float`] implementation so it stays
/// dependency-free on the `core`-only target while remaining accurate for tiny
/// and huge magnitudes.
fn sqrt(x: f64) -> f64 {
    out_zero_float::sqrt(x)
}

#[cfg(test)]
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::manual_range_contains
)]
mod tests {
    use super::*;
    use tpt_zero_rand::{Pcg32, SeedableRng};

    #[allow(clippy::cast_precision_loss)]
    fn in_unit(v: f64) -> bool {
        v >= 0.0 && v < 1.0
    }

    #[test]
    fn integral_of_x_is_half() {
        let mut rng = Pcg32::seed_from_u64(1);
        let est = monte_carlo_integral(|x| x, &mut rng, 20_000);
        assert!((est - 0.5).abs() < 1e-2, "estimate {est} too far from 0.5");
    }

    #[test]
    fn integral_of_constant_is_constant() {
        let mut rng = Pcg32::seed_from_u64(2);
        let est = monte_carlo_integral(|_| 3.0, &mut rng, 1_000);
        assert!((est - 3.0).abs() < 1e-9);
    }

    #[test]
    fn integral_of_x_squared_is_third() {
        let mut rng = Pcg32::seed_from_u64(3);
        let est = monte_carlo_integral(|x| x * x, &mut rng, 40_000);
        assert!((est - 1.0 / 3.0).abs() < 2e-2, "estimate {est}");
    }

    #[test]
    fn zero_samples_give_zero_estimate() {
        let mut rng = Pcg32::seed_from_u64(4);
        assert!((monte_carlo_integral(|x| x, &mut rng, 0) - 0.0).abs() < 1e-12);
    }

    #[test]
    fn antithetic_integral_of_x_is_half() {
        let mut rng = Pcg32::seed_from_u64(5);
        let est = antithetic_estimate(|x| x, &mut rng, 20_000);
        assert!((est - 0.5).abs() < 1e-2, "antithetic estimate {est}");
    }

    #[test]
    fn antithetic_reduces_variance_for_monotone_f() {
        // For f(x) = x, the antithetic estimator is EXACTLY 0.5 every run
        // (variance 0), while the plain estimator has positive variance. We
        // compare estimator variance across replications.
        let plain_var = {
            let mut rng = Pcg32::seed_from_u64(6);
            sample_mean_variance(|x| x, &mut rng, 500, 200).1
        };
        let anti_var = {
            let mut rng = Pcg32::seed_from_u64(6);
            let mut estimates = [0.0f64; 500];
            for e in &mut estimates {
                *e = antithetic_estimate(|x| x, &mut rng, 200);
            }
            variance(&estimates).unwrap_or(f64::INFINITY)
        };
        assert!(anti_var < plain_var, "antithetic {anti_var} >= plain {plain_var}");
    }

    #[test]
    fn control_variate_integral_of_x_squared() {
        let mut rng = Pcg32::seed_from_u64(7);
        // g(x) = x, âˆ«_0^1 x dx = 0.5
        let est = control_variate_estimate(|x| x * x, |x| x, 0.5, &mut rng, 40_000);
        assert!((est - 1.0 / 3.0).abs() < 2e-2, "control variate estimate {est}");
    }

    #[test]
    fn control_variate_with_identity_control_is_exact_for_constant() {
        let mut rng = Pcg32::seed_from_u64(8);
        // f(x) = c + x, g(x) = x; âˆ« g = 0.5 => estimate should be c + 0.5.
        let est = control_variate_estimate(|x| 2.0 + x, |x| x, 0.5, &mut rng, 1_000);
        assert!((est - 2.5).abs() < 1e-9, "estimate {est}");
    }

    #[test]
    fn mean_and_error_shrink_with_n() {
        let mut rng = Pcg32::seed_from_u64(9);
        let mut small = [0.0f64; 200];
        for s in &mut small {
            *s = monte_carlo_integral(|x| x, &mut rng, 100);
        }
        let (_, sem_small) = estimate_mean_with_error(&small);

        let mut rng2 = Pcg32::seed_from_u64(9);
        let mut large = [0.0f64; 800];
        for s in &mut large {
            *s = monte_carlo_integral(|x| x, &mut rng2, 100);
        }
        let (mean_large, sem_large) = estimate_mean_with_error(&large);

        assert!((mean_large - 0.5).abs() < 0.05);
        // Standard error should scale roughly like 1/sqrt(n); with 4x the
        // samples it should be at most half (loosely: strictly smaller).
        assert!(
            sem_large < sem_small,
            "sem_large {sem_large} >= sem_small {sem_small}"
        );
    }

    #[test]
    fn estimate_mean_empty_returns_zero() {
        let samples: &[f64] = &[];
        assert_eq!(estimate_mean_with_error(samples), (0.0, 0.0));
    }

    #[test]
    fn sample_mean_variance_single_rep_is_zero_var() {
        let mut rng = Pcg32::seed_from_u64(10);
        let (mean, var) = sample_mean_variance(|x| x, &mut rng, 1, 500);
        assert!((mean - 0.5).abs() < 0.1);
        assert!((var - 0.0).abs() < 1e-12);
    }

    #[test]
    fn all_samples_in_unit_interval() {
        // Sanity: next_f64 stays in [0,1), so the integrand inputs are valid.
        let mut rng = Pcg32::seed_from_u64(11);
        for _ in 0..1000 {
            assert!(in_unit(rng.next_f64()));
        }
    }

    mod proptests {
        #![allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::manual_range_contains
        )]

        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn integral_of_linear_is_exact_mean(seed in any::<u64>(), n in 1000usize..50_000) {
                // âˆ«_0^1 (a + b x) dx = a + b/2. With a=0,b=1 => 0.5.
                let mut rng = Pcg32::seed_from_u64(seed);
                let est = monte_carlo_integral(|x| x, &mut rng, n);
                prop_assert!((est - 0.5).abs() < 0.1, "estimate {est} for n={n}");
            }

            #[test]
            fn antithetic_constant_is_exact(seed in any::<u64>(), n in 100usize..10_000) {
                let mut rng = Pcg32::seed_from_u64(seed);
                let est = antithetic_estimate(|_| 7.0, &mut rng, n);
                prop_assert!((est - 7.0).abs() < 1e-12, "estimate {est}");
            }

            #[test]
            fn control_variate_constant_control(seed in any::<u64>(), n in 1000usize..50_000) {
                // f(x) = 1 + 2x, g(x) = x, âˆ« g = 0.5 => exact = 1 + 1 = 2.
                // The estimator is unbiased but carries Monte Carlo noise, so
                // we allow a statistical tolerance rather than a tight bound.
                let mut rng = Pcg32::seed_from_u64(seed);
                let est = control_variate_estimate(|x| 1.0 + 2.0 * x, |x| x, 0.5, &mut rng, n);
                prop_assert!((est - 2.0).abs() < 0.05, "estimate {est}");
            }
        }
    }
}
