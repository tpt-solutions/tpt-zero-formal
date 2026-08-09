//! A conjugate-prior Bayesian update with zero dependencies.
use out_zero_formal::prelude::*;

fn main() {
    // Beta(1,1) uniform prior on a coin's bias; observe 7 heads, 3 tails.
    let prior = Beta::new(1.0, 1.0).unwrap();
    let posterior = prior.posterior(7, 3); // Beta(8, 4)
    println!(
        "posterior mean: {} (expected {})",
        posterior.mean(),
        8.0 / 12.0
    );
    assert!((posterior.mean() - 8.0 / 12.0).abs() < 1e-12);
}
