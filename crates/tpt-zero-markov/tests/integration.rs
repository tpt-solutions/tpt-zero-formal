//! Integration tests for `tpt-zero-markov`.
//!
//! These tests exercise the public API of the crate and are allowed to use the
//! standard library (this is a `#[cfg(test)]`-style examples directory, but
//! written as a standalone integration test so `std` is available).

use tpt_zero_markov::Chain;
use tpt_zero_tensor::{Tensor, Tensor2};

#[cfg(feature = "alloc")]
use tpt_zero_rand::{Pcg32, SeedableRng};

#[test]
fn integration_rows_stochastic_and_n_step() {
    let p = Tensor2::from([[0.8, 0.2], [0.3, 0.7]]);
    let chain = Chain::checked_new(p, Tensor::from([1.0, 0.0])).unwrap();

    // Every row of the (already stochastic) transition matrix sums to 1.
    let t = chain.transition();
    for r in 0..2 {
        let row = t.row(r);
        let s: f64 = row.iter().copied().sum();
        assert!((s - 1.0).abs() < 1e-12);
    }

    // n_step(3) reconstructs correctly and stays stochastic.
    let p3 = chain.n_step(3);
    for r in 0..2 {
        let row = p3.row(r);
        let s: f64 = row.iter().copied().sum();
        assert!((s - 1.0).abs() < 1e-12);
    }
}

#[test]
fn integration_stationary_balance_three_state() {
    // A 3-state chain with a known stationary distribution.
    let p = Tensor2::from([
        [0.7, 0.2, 0.1],
        [0.1, 0.8, 0.1],
        [0.2, 0.3, 0.5],
    ]);
    let chain = Chain::checked_new(p, Tensor::from([1.0, 0.0, 0.0])).unwrap();
    let pi = chain.stationary().unwrap();

    // Balance: pi P = pi.
    let balanced = chain.step(&pi);
    for i in 0..3 {
        assert!(
            (balanced[i] - pi[i]).abs() < 1e-7,
            "stationary balance violated at {i}: {} vs {}",
            balanced[i],
            pi[i]
        );
    }

    // Normalized: sums to 1.
    let s: f64 = pi.iter().copied().sum();
    assert!((s - 1.0).abs() < 1e-7);

    // All entries non-negative.
    for i in 0..3 {
        assert!(pi[i] >= 0.0, "entry {i} negative: {}", pi[i]);
    }
}

#[cfg(feature = "alloc")]
#[test]
fn integration_sample_deterministic_for_seed() {
    let p = Tensor2::from([[0.5, 0.5], [0.5, 0.5]]);
    let chain = Chain::checked_new(p, Tensor::from([1.0, 0.0])).unwrap();

    let mut rng_a = Pcg32::seed_from_u64(123);
    let path_a = chain.sample(&mut rng_a, 20, 0);

    let mut rng_b = Pcg32::seed_from_u64(123);
    let path_b = chain.sample(&mut rng_b, 20, 0);

    assert_eq!(path_a, path_b);
    assert_eq!(path_a.len(), 20);
    assert_eq!(path_a[0], 0);
}

#[test]
fn integration_n_step_power_identity() {
    // For the deterministic flip chain, P^2 = I.
    let p = Tensor2::from([[0.0, 1.0], [1.0, 0.0]]);
    let chain = Chain::checked_new(p, Tensor::from([1.0, 0.0])).unwrap();
    let p2 = chain.n_step(2);
    assert!((p2[(0, 0)] - 1.0).abs() < 1e-12);
    assert!((p2[(1, 1)] - 1.0).abs() < 1e-12);
    assert!((p2[(0, 1)] - 0.0).abs() < 1e-12);
}
