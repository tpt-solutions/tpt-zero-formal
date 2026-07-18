//! Integration tests for `tpt-zero-monte-carlo`.
//!
//! These run under `std` (the default Cargo test target), so heap allocation
//! and the full standard library are available; the crate itself remains
//! `no_std`.

#![allow(clippy::cast_precision_loss)]

use tpt_zero_monte_carlo::{
    antithetic_estimate, control_variate_estimate, estimate_mean_with_error, monte_carlo_integral,
    sample_mean_variance,
};
use tpt_zero_rand::{Pcg32, SeedableRng};

#[test]
fn integral_of_x_over_unit_interval_is_half() {
    let mut rng = Pcg32::seed_from_u64(0xABCD);
    let est = monte_carlo_integral(|x| x, &mut rng, 50_000);
    assert!((est - 0.5).abs() < 1e-2, "estimate {est} too far from 0.5");
}

#[test]
fn integral_of_x_squared_is_one_third() {
    let mut rng = Pcg32::seed_from_u64(0x1234);
    let est = monte_carlo_integral(|x| x * x, &mut rng, 80_000);
    assert!((est - 1.0 / 3.0).abs() < 2e-2, "estimate {est}");
}

#[test]
fn integral_of_exp_is_e_minus_one() {
    let mut rng = Pcg32::seed_from_u64(0x5678);
    let est = monte_carlo_integral(f64::exp, &mut rng, 80_000);
    let truth = core::f64::consts::E - 1.0;
    assert!((est - truth).abs() < 2e-2, "estimate {est}, truth {truth}");
}

#[test]
fn antithetic_reduces_variance_for_monotone_integrand() {
    // For f(x) = x the antithetic estimator is exactly 0.5 every run, so its
    // across-replicate variance is 0; the plain estimator has positive
    // variance.
    let plain_var = {
        let mut rng = Pcg32::seed_from_u64(0x9999);
        sample_mean_variance(|x| x, &mut rng, 400, 300).1
    };
    let anti_var = {
        let mut rng = Pcg32::seed_from_u64(0x9999);
        let mut estimates = Vec::new();
        for _ in 0..400 {
            estimates.push(antithetic_estimate(|x| x, &mut rng, 300));
        }
        let mean = estimates.iter().sum::<f64>() / estimates.len() as f64;
        let v = estimates
            .iter()
            .map(|&e| (e - mean) * (e - mean))
            .sum::<f64>()
            / (estimates.len() as f64 - 1.0);
        v
    };
    assert!(
        anti_var < plain_var,
        "antithetic variance {anti_var} >= plain {plain_var}"
    );
}

#[test]
fn control_variate_recovers_known_integral() {
    // ∫_0^1 (x^2 + x) dx = 1/3 + 1/2 = 5/6.
    // Use control g(x) = x with ∫_0^1 x dx = 0.5.
    let mut rng = Pcg32::seed_from_u64(0x7777);
    let est = control_variate_estimate(|x| x * x + x, |x| x, 0.5, &mut rng, 80_000);
    assert!((est - 5.0 / 6.0).abs() < 2e-2, "estimate {est}");
}

#[test]
fn standard_error_shrinks_roughly_like_inverse_sqrt_n() {
    let (sem_small, sem_large) = {
        let mut rng = Pcg32::seed_from_u64(0x2468);
        let small: Vec<f64> = (0..500)
            .map(|_| monte_carlo_integral(|x| x, &mut rng, 100))
            .collect();
        let (_, sem_s) = estimate_mean_with_error(&small);
        let mut rng2 = Pcg32::seed_from_u64(0x2468);
        let large: Vec<f64> = (0..2000)
            .map(|_| monte_carlo_integral(|x| x, &mut rng2, 100))
            .collect();
        let (mean_l, sem_l) = estimate_mean_with_error(&large);
        assert!((mean_l - 0.5).abs() < 0.05);
        (sem_s, sem_l)
    };
    // 4x the samples => SEM at most ~half (loosely, strictly less).
    assert!(
        sem_large < sem_small,
        "sem_large {sem_large} >= sem_small {sem_small}"
    );
}

#[test]
fn sample_mean_variance_positive_and_centered() {
    let mut rng = Pcg32::seed_from_u64(0x1357);
    let (mean, var) = sample_mean_variance(|x| x, &mut rng, 300, 400);
    assert!((mean - 0.5).abs() < 0.05);
    assert!(var > 0.0, "expected positive estimator variance, got {var}");
    assert!(var < 1.0);
}
