//! Property-based tests for `tpt-zero-dist`.
#![allow(clippy::cast_precision_loss)]

use proptest::prelude::*;
use tpt_zero_dist::{Bernoulli, Distribution, Normal, Poisson, Uniform};
use tpt_zero_rand::{Pcg32, SeedableRng};

proptest! {
    #[test]
    fn uniform_sample_mean_close(
        a in -50.0f64..50.0,
        width in 0.5f64..100.0,
        seed in any::<u64>(),
    ) {
        let b = a + width;
        let u = Uniform::new(a, b).unwrap();
        let mut rng = Pcg32::seed_from_u64(seed);
        let n = 20_000;
        let mut sum = 0.0;
        for _ in 0..n {
            let x = u.sample(&mut rng);
            prop_assert!((a..b).contains(&x), "sample {x} outside [{a}, {b})");
            sum += x;
        }
        let mean = sum / f64::from(n);
        // Tolerance scales with the width of the interval.
        let tol = width * 0.03 + 0.05;
        prop_assert!((mean - u.mean()).abs() < tol, "mean {mean} vs {}", u.mean());
    }

    #[test]
    fn normal_sample_mean_close(
        mu in -20.0f64..20.0,
        sigma in 0.2f64..10.0,
        seed in any::<u64>(),
    ) {
        let n = Normal::new(mu, sigma).unwrap();
        let mut rng = Pcg32::seed_from_u64(seed);
        let count = 40_000;
        let mut sum = 0.0;
        for _ in 0..count {
            sum += n.sample(&mut rng);
        }
        let mean = sum / f64::from(count);
        let tol = sigma * 0.05 + 0.05;
        prop_assert!((mean - mu).abs() < tol, "mean {mean} vs {mu}");
    }

    #[test]
    fn bernoulli_sample_is_binary(
        p in 0.0f64..=1.0,
        seed in any::<u64>(),
    ) {
        let b = Bernoulli::new(p).unwrap();
        let mut rng = Pcg32::seed_from_u64(seed);
        let n = 5_000u64;
        let mut ones = 0u64;
        for _ in 0..n {
            let s = b.sample(&mut rng);
            prop_assert!(s == 0 || s == 1, "sample {s} not binary");
            ones += s;
        }
        let frac = ones as f64 / n as f64;
        prop_assert!((frac - p).abs() < 0.05, "frac {frac} vs {p}");
    }

    #[test]
    fn poisson_sample_mean_close(
        lambda in 0.5f64..20.0,
        seed in any::<u64>(),
    ) {
        let dist = Poisson::new(lambda).unwrap();
        let mut rng = Pcg32::seed_from_u64(seed);
        let n = 40_000u64;
        let mut sum = 0.0;
        for _ in 0..n {
            sum += dist.sample(&mut rng) as f64;
        }
        let mean = sum / n as f64;
        let tol = lambda * 0.05 + 0.1;
        prop_assert!((mean - lambda).abs() < tol, "mean {mean} vs {lambda}");
    }

    #[test]
    fn poisson_pmf_sums_to_one(lambda in 0.1f64..15.0) {
        let dist = Poisson::new(lambda).unwrap();
        let mut sum = 0.0;
        for k in 0..200u64 {
            sum += dist.pmf(k);
        }
        prop_assert!((sum - 1.0).abs() < 1e-6, "pmf sum {sum}");
    }

    #[test]
    fn uniform_pdf_integrates_to_one(
        a in -20.0f64..20.0,
        width in 1.0f64..40.0,
    ) {
        let b = a + width;
        let u = Uniform::new(a, b).unwrap();
        // Riemann sum of the pdf over a slightly padded range.
        let steps = 5_000;
        let lo = a - 1.0;
        let hi = b + 1.0;
        let dx = (hi - lo) / f64::from(steps);
        let mut area = 0.0;
        for i in 0..steps {
            let x = lo + (f64::from(i) + 0.5) * dx;
            area += u.pdf(x) * dx;
        }
        prop_assert!((area - 1.0).abs() < 0.02, "area {area}");
    }
}
