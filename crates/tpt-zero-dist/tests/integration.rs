//! Integration tests for `tpt-zero-dist`. `std` is available here.
#![allow(clippy::cast_precision_loss, clippy::cast_lossless)]

use tpt_zero_dist::{Bernoulli, Distribution, Normal, Poisson, Uniform};
use tpt_zero_rand::{Pcg32, SeedableRng};

fn empirical_mean(samples: &[f64]) -> f64 {
    samples.iter().sum::<f64>() / samples.len() as f64
}

#[test]
fn uniform_sampling_matches_analytic() {
    let u = Uniform::new(-2.0, 6.0).unwrap();
    let mut rng = Pcg32::seed_from_u64(11);
    let mut samples = Vec::with_capacity(50_000);
    for _ in 0..50_000 {
        samples.push(u.sample(&mut rng));
    }
    let mean = empirical_mean(&samples);
    assert!((mean - u.mean()).abs() < 0.05, "mean {mean} vs {}", u.mean());
    // Min and max stay inside the support.
    let lo = samples.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = samples.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    assert!(lo >= -2.0);
    assert!(hi < 6.0);
}

#[test]
fn normal_sampling_matches_analytic() {
    let n = Normal::new(10.0, 3.0).unwrap();
    let mut rng = Pcg32::seed_from_u64(22);
    let mut samples = Vec::with_capacity(100_000);
    for _ in 0..100_000 {
        samples.push(n.sample(&mut rng));
    }
    let mean = empirical_mean(&samples);
    assert!((mean - n.mean()).abs() < 0.05, "mean {mean} vs {}", n.mean());
    // Empirical variance.
    let var = samples.iter().map(|&x| (x - mean) * (x - mean)).sum::<f64>() / samples.len() as f64;
    assert!((var - n.variance()).abs() < 0.2, "var {var} vs {}", n.variance());
}

#[test]
fn bernoulli_samples_are_binary() {
    let b = Bernoulli::new(0.4).unwrap();
    let mut rng = Pcg32::seed_from_u64(33);
    let mut samples = Vec::with_capacity(20_000);
    for _ in 0..20_000 {
        let s = b.sample(&mut rng);
        assert!(s == 0 || s == 1);
        samples.push(s as f64);
    }
    let frac = samples.iter().sum::<f64>() / samples.len() as f64;
    assert!((frac - 0.4).abs() < 0.02, "frac {frac}");
}

#[test]
fn poisson_sampling_matches_analytic() {
    let p = Poisson::new(5.0).unwrap();
    let mut rng = Pcg32::seed_from_u64(44);
    let mut samples = Vec::with_capacity(100_000);
    for _ in 0..100_000 {
        samples.push(p.sample(&mut rng) as f64);
    }
    let mean = empirical_mean(&samples);
    assert!((mean - p.mean()).abs() < 0.05, "mean {mean} vs {}", p.mean());
}

#[test]
fn pmf_sums_to_one() {
    let b = Bernoulli::new(0.7).unwrap();
    let bsum: f64 = (0..=1).map(|k| b.pmf(k)).sum();
    assert!((bsum - 1.0).abs() < 1e-12);

    let p = Poisson::new(7.0).unwrap();
    let psum: f64 = (0..=60).map(|k| p.pmf(k)).sum();
    assert!((psum - 1.0).abs() < 1e-9, "poisson pmf sum {psum}");
}

#[test]
fn cdf_is_monotone_and_bounded() {
    let n = Normal::new(0.0, 1.0).unwrap();
    let mut prev = -1.0;
    for i in -10..=10 {
        let x = i as f64 * 0.5;
        let c = n.cdf(x);
        assert!((c - prev) >= -1e-12, "cdf not monotone at {x}");
        assert!((0.0..=1.0).contains(&c));
        prev = c;
    }
}
