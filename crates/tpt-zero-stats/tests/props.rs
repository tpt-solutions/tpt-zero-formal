//! Proptest-based property tests for `tpt-zero-stats`.

#![allow(clippy::cast_precision_loss)]

use proptest::prelude::*;
use tpt_zero_stats::{
    mean, mean_iter, percentile, population_variance, std_dev, variance,
};

proptest! {
    #[test]
    fn mean_of_constant_equals_constant(values in prop::collection::vec(any::<f64>(), 1..50)) {
        let c = 4.25_f64;
        let data: Vec<f64> = values.iter().map(|_| c).collect();
        prop_assert!((mean(&data).unwrap() - c).abs() < 1e-9);
        prop_assert!((mean_iter(data.clone()).unwrap() - c).abs() < 1e-9);
    }

    #[test]
    fn variance_of_constant_is_zero(values in prop::collection::vec(any::<f64>(), 2..50)) {
        let c = -1.5_f64;
        let data: Vec<f64> = values.iter().map(|_| c).collect();
        prop_assert!(variance(&data).unwrap().abs() < 1e-9);
        prop_assert!(population_variance(&data).unwrap().abs() < 1e-9);
        prop_assert!(std_dev(&data).unwrap().abs() < 1e-9);
    }

    #[test]
    fn percentile_bounds_hold(data in prop::collection::vec(any::<f64>(), 1..50), p in any::<f64>()) {
        let n = data.len();
        let mut scratch = vec![0.0; n];
        let result = percentile(&data, p, &mut scratch);
        prop_assert!(result.is_some());
        let v = result.unwrap();
        let lo = data.iter().copied().fold(f64::INFINITY, f64::min);
        let hi = data.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        prop_assert!(v >= lo - 1e-9 && v <= hi + 1e-9, "percentile {} outside [{}, {}]", v, lo, hi);
    }

    #[test]
    fn mean_matches_manual_sum(data in prop::collection::vec(any::<f64>().prop_filter("finite", |x| x.is_finite()), 1..50)) {
        let n = data.len() as f64;
        let sum: f64 = data.iter().copied().sum();
        // `f64` summation overflows to +/-inf for large-magnitude inputs; the
        // `mean == sum/n` identity is only well-defined when the sum is finite.
        prop_assume!(sum.is_finite());
        prop_assert!((mean(&data).unwrap() - sum / n).abs() < 1e-6 * (1.0 + sum.abs()));
    }
}
