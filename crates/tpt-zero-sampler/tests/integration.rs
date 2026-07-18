//! Integration tests for `tpt-zero-sampler`. These exercise the public API as
//! a downstream consumer would, using `std` (allowed in `tests/`).
//!
//! `cargo test -p tpt-zero-sampler` runs these alongside the in-crate unit
//! tests and doctests.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::manual_range_contains,
    clippy::neg_cmp_op_on_partial_ord,
    clippy::float_equality_without_abs
)]

use tpt_zero_prob::distributions::Uniform;
use tpt_zero_prob::Distribution;
use tpt_zero_rand::{Pcg32, Rng, SeedableRng};
use tpt_zero_sampler::{inverse_transform_sample, metropolis_hastings, rejection_sample};

#[test]
fn rejection_sample_uniform_proposal_contains_range() {
    let proposal = Uniform::new(0.0, 1.0).unwrap();
    let mut rng = Pcg32::seed_from_u64(123);
    for _ in 0..500 {
        let x = rejection_sample(
            |x| 0.5 * proposal.pdf(x),
            &proposal,
            1.0,
            &mut rng,
            100,
        )
        .expect("rejection sampling should accept within max_trials");
        assert!((0.0..=1.0).contains(&x));
    }
}

#[test]
fn inverse_transform_uniform_identity_in_unit_interval() {
    let mut rng = Pcg32::seed_from_u64(456);
    for _ in 0..2000 {
        let x = inverse_transform_sample(|u| u, &mut rng);
        assert!((0.0..1.0).contains(&x));
    }
}

#[test]
fn inverse_transform_uniform_ab_in_range() {
    let mut rng = Pcg32::seed_from_u64(789);
    for _ in 0..2000 {
        // Uniform(-3, 3) inverse CDF is -3 + u*6.
        let x = inverse_transform_sample(|u| -3.0 + u * 6.0, &mut rng);
        assert!((-3.0..3.0).contains(&x));
    }
}

#[test]
fn metropolis_hastings_normal_chain_is_finite() {
    let mut rng = Pcg32::seed_from_u64(2024);
    let target = |x: f64| -0.5 * x * x; // log N(0,1).
    let mut x = 0.0f64;
    for _ in 0..1000 {
        let step = rng.next_f64() - 0.5;
        x = metropolis_hastings(x, target, |c| c + step, &mut rng);
        assert!(x.is_finite());
    }
}

#[test]
fn metropolis_hastings_bimodal_mixes_across_modes() {
    // A bimodal target: two gaussians at -3 and +3. The chain should reach both
    // sides of zero over a long enough run.
    let mut rng = Pcg32::seed_from_u64(7);
    let target = |x: f64| {
        let a = -0.5 * (x - 3.0) * (x - 3.0);
        let b = -0.5 * (x + 3.0) * (x + 3.0);
        a.max(b)
    };
    let mut x = 3.0f64;
    let mut saw_positive = false;
    let mut saw_negative = false;
    for _ in 0..20000 {
        let step = rng.next_f64() - 0.5;
        x = metropolis_hastings(x, target, |c| c + 2.0 * step, &mut rng);
        if x > 0.0 {
            saw_positive = true;
        } else if x < 0.0 {
            saw_negative = true;
        }
        if saw_positive && saw_negative {
            break;
        }
    }
    assert!(saw_positive && saw_negative, "chain did not mix across both modes");
}
