//! Property tests for `out-zero-refinement`.
//!
//! These exercise the core invariant: `Refined::new` succeeds exactly when
//! the predicate holds, and a successfully refined value round-trips.

use proptest::prelude::*;
use out_zero_refinement::{NonZeroU32, PositiveI64, Predicate, Refined};

proptest! {
    #[test]
    fn nonzero_u32_matches_predicate(v in any::<u32>()) {
        let result = Refined::<u32, NonZeroU32>::new(v);
        prop_assert_eq!(result.is_ok(), NonZeroU32::check(&v));
        if let Ok(r) = result {
            prop_assert_eq!(*r.get(), v);
            prop_assert_eq!(r.into_inner(), v);
        }
    }

    #[test]
    fn positive_i64_matches_predicate(v in any::<i64>()) {
        let result = Refined::<i64, PositiveI64>::new(v);
        prop_assert_eq!(result.is_ok(), PositiveI64::check(&v));
        prop_assert_eq!(result.is_ok(), v > 0);
    }

    #[test]
    fn rejected_value_is_returned(v in any::<u32>()) {
        if !NonZeroU32::check(&v) {
            let err = Refined::<u32, NonZeroU32>::new(v).unwrap_err();
            prop_assert_eq!(err.into_value(), v);
        }
    }
}
