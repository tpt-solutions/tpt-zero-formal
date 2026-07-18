//! Integration tests for `tpt-zero-bayes`.
//!
//! These tests exercise the public API end-to-end against the sibling
//! `tpt-zero-dist` / `tpt-zero-prob` / `tpt-zero-stats` crates and verify
//! the qualitative behaviour the crate guarantees: posterior means move toward
//! the data, the uniform `Beta(1, 1)` is flat, and observing successes
//! increases `alpha`.

#![allow(clippy::float_cmp, clippy::cast_precision_loss, clippy::float_cmp_const)]

use tpt_zero_bayes::{Beta, Gamma, GaussianKnownVar};

#[test]
fn beta_uniform_is_flat() {
    // Beta(1,1) must be uniform: mean 1/2 and constant density 1 on [0,1].
    let prior = Beta::new(1.0, 1.0).expect("valid uniform prior");
    assert!((prior.mean() - 0.5).abs() < 1e-12);
    for &x in &[0.1f64, 0.25, 0.5, 0.75, 0.9] {
        assert!((prior.pdf(x) - 1.0).abs() < 1e-12, "pdf({x}) = {}", prior.pdf(x));
    }
}

#[test]
fn beta_posterior_mean_moves_toward_data() {
    // Strong prior belief near 0.05, but data is mostly successes.
    let prior = Beta::new(1.0, 19.0).expect("valid prior");
    let prior_mean = prior.mean();
    let posterior = prior.posterior(80, 20);
    // The data success rate is 0.8; the posterior mean should land between
    // the prior mean and 0.8, and strictly above the prior mean.
    assert!(posterior.mean() > prior_mean);
    let target = 80.0 / (80.0 + 20.0);
    assert!(posterior.mean() <= target + 1e-9);
    assert!(posterior.mean() >= prior_mean);
}

#[test]
fn beta_successes_increase_alpha() {
    let prior = Beta::new(2.0, 3.0).expect("valid prior");
    let post = prior.posterior(5, 1);
    assert!(post.alpha() > prior.alpha());
    assert!((post.alpha() - 7.0).abs() < 1e-12);
    assert!((post.beta() - 4.0).abs() < 1e-12);
}

#[test]
fn gaussian_posterior_pulls_to_sample_mean() {
    let prior = GaussianKnownVar::new(0.0, 1.0, 0.25).expect("valid prior");
    let data = [0.9f64, 1.1, 0.8, 1.2, 1.0];
    let post = prior.update(&data);
    let sample_mean: f64 = data.iter().copied().sum::<f64>() / data.len() as f64;
    // Posterior mean is between the prior mean (0) and the sample mean.
    assert!(post.mean() > 0.0);
    assert!(post.mean() < sample_mean + 1e-9);
    // More data shrinks the variance below the prior variance.
    assert!(post.variance() < prior.variance());
}

#[test]
fn gaussian_empty_data_returns_prior() {
    let prior = GaussianKnownVar::new(1.0, 2.0, 0.5).expect("valid prior");
    let post = prior.update(&[]);
    assert_eq!(post.mean(), prior.mean());
    assert_eq!(post.variance(), prior.variance());
}

#[test]
fn gamma_posterior_integrates_counts_and_exposure() {
    let prior = Gamma::new(2.0, 1.0).expect("valid prior");
    let counts = [4u64, 6, 10];
    let exposure = 5.0;
    let post = prior.posterior(&counts, exposure);
    let total: u64 = counts.iter().copied().sum();
    assert!((post.shape() - (prior.shape() + total as f64)).abs() < 1e-12);
    assert!((post.rate() - (prior.rate() + exposure)).abs() < 1e-12);
    // Posterior mean is observable rate = shape / rate.
    assert!((post.mean() - (2.0 + total as f64) / (1.0 + exposure)).abs() < 1e-12);
}
