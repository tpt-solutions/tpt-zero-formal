#![doc = include_str!("../README.md")]

#![no_std]
#![warn(missing_docs)]
#![forbid(unsafe_code)]

use tpt_zero_prob::Distribution;
use tpt_zero_rand::Rng;

/// Samples a target density by rejection sampling against a proposal.
///
/// The proposal `proposal` must dominate the target: there must exist a finite
/// constant `m` such that `target_pdf(x) <= m * proposal.pdf(x)` for every `x`
/// in the support. The `envelope` argument is exactly this constant `m`. A
/// draw `x` from the proposal is accepted with probability
/// `target_pdf(x) / (m * proposal.pdf(x))`.
///
/// Returns `Some(x)` when a draw is accepted within `max_trials` attempts, or
/// `None` if the envelope is exhausted without an acceptance (for example when
/// `m` is too small to actually dominate, or the density is degenerate).
///
/// # Type parameters
///
/// * `R` â€” the random source, any [`Rng`].
/// * `D` â€” the proposal distribution, any [`Distribution`] returning `f64`.
///
/// # Arguments
///
/// * `target_pdf` â€” the (unnormalized) target density.
/// * `proposal` â€” the proposal distribution; its `pdf` must dominate the
///   target scaled by `envelope`.
/// * `envelope` â€” the constant `m` such that `target_pdf(x) <= m * proposal.pdf(x)`.
/// * `rng` â€” the source of randomness.
/// * `max_trials` â€” the maximum number of proposal draws before giving up.
///
/// # Examples
///
/// ```
/// use tpt_zero_prob::distributions::Uniform;
/// use tpt_zero_prob::Distribution;
/// use tpt_zero_rand::{Pcg32, Rng, SeedableRng};
/// use tpt_zero_sampler::rejection_sample;
///
/// let proposal = Uniform::new(0.0, 1.0).unwrap();
/// let mut rng = Pcg32::seed_from_u64(1);
/// // Target is the same uniform, scaled by 0.9 (so envelope m = 1 dominates).
/// let sample = rejection_sample(
///     |x| 0.9 * proposal.pdf(x),
///     &proposal,
///     1.0,
///     &mut rng,
///     100,
/// );
/// let x = sample.unwrap();
/// assert!(x >= 0.0 && x <= 1.0);
/// ```
#[must_use]
pub fn rejection_sample<R, D, F>(
    target_pdf: F,
    proposal: &D,
    envelope: f64,
    rng: &mut R,
    max_trials: usize,
) -> Option<f64>
where
    R: Rng,
    D: Distribution<Value = f64>,
    F: Fn(f64) -> f64,
{
    if envelope <= 0.0 || !envelope.is_finite() {
        return None;
    }
    for _ in 0..max_trials {
        // Draw a candidate from the proposal. We use the support mapping of the
        // proposal via its own inverse CDF only when available; here we instead
        // sample the candidate by inverse-transforming a uniform draw through the
        // proposal's analytic CDF/inverse, which requires a uniform draw.
        let u = rng.next_f64();
        // Reconstruct a candidate on the proposal by a uniform-in-CDF draw.
        let x = inverse_cdf_uniform(proposal, u);
        let p = proposal.pdf(x);
        if p <= 0.0 || !p.is_finite() {
            continue;
        }
        let target = target_pdf(x);
        if target < 0.0 || !target.is_finite() {
            continue;
        }
        let threshold = target / (envelope * p);
        if rng.next_f64() < threshold {
            return Some(x);
        }
    }
    None
}

/// Approximates the inverse CDF of a [`Distribution`] by bisection on its `cdf`.
///
/// Used internally to draw a candidate value `x` given a uniform `u` in `[0,1)`
/// equal to `cdf(x)`. This avoids relying on a closed-form inverse being
/// available for every [`Distribution`]. When `u` is exactly `0` the lower
/// support bound is returned; when `u >= 1` the upper bound is returned.
#[must_use]
fn inverse_cdf_uniform<D: Distribution<Value = f64>>(dist: &D, u: f64) -> f64 {
    // Bracket the support by walking outward from the mean until the CDF spans
    // the target probability. We start from the analytic mean when finite.
    let start = dist.mean().unwrap_or(0.0);
    if u <= 0.0 {
        return bisect_cdf(dist, 0.0, start);
    }
    if u >= 1.0 {
        return bisect_cdf(dist, 1.0, start);
    }
    bisect_cdf(dist, u, start)
}

/// Bisection search for `x` such that `cdf(x) â‰ˆ target`, expanding the
/// bracketing interval geometrically until `cdf` spans `target`.
#[must_use]
fn bisect_cdf<D: Distribution<Value = f64>>(dist: &D, target: f64, start: f64) -> f64 {
    const ITER: usize = 60;
    let mut lo = start;
    let mut hi = start;
    if dist.cdf(lo) > target {
        // Walk left.
        let mut step = 1.0f64;
        for _ in 0..ITER {
            let cand = lo - step;
            if dist.cdf(cand) <= target || !dist.cdf(cand).is_finite() {
                hi = lo;
                lo = cand;
                break;
            }
            lo = cand;
            step *= 2.0;
        }
    } else {
        // Walk right.
        let mut step = 1.0f64;
        for _ in 0..ITER {
            let cand = hi + step;
            if dist.cdf(cand) >= target || !dist.cdf(cand).is_finite() {
                lo = hi;
                hi = cand;
                break;
            }
            hi = cand;
            step *= 2.0;
        }
    }
    // Standard bisection between lo and hi.
    for _ in 0..ITER {
        let mid = lo + 0.5 * (hi - lo);
        if dist.cdf(mid) < target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo + 0.5 * (hi - lo)
}

/// Samples a value by inverse-transform sampling through an inverse CDF.
///
/// Given `cdf_inverse` â€” the inverse of a cumulative distribution function,
/// mapping a uniform draw `u in [0,1)` to a sample `x` â€” this returns
/// `cdf_inverse(rng.next_f64())`. Because `rng.next_f64()` is uniform on
/// `[0,1)`, the returned value follows the distribution whose CDF is inverted.
///
/// # Type parameters
///
/// * `R` â€” the random source, any [`Rng`].
/// * `F` â€” the inverse-CDF closure `Fn(f64) -> f64`.
///
/// # Examples
///
/// ```
/// use tpt_zero_rand::{Pcg32, Rng, SeedableRng};
/// use tpt_zero_sampler::inverse_transform_sample;
///
/// let mut rng = Pcg32::seed_from_u64(3);
/// // Inverse CDF of Uniform(0,1) is the identity.
/// let x = inverse_transform_sample(|u| u, &mut rng);
/// assert!(x >= 0.0 && x < 1.0);
/// ```
///
/// For a `Uniform(a, b)` the inverse CDF is `a + u * (b - a)`:
///
/// ```
/// use tpt_zero_rand::{Pcg32, Rng, SeedableRng};
/// use tpt_zero_sampler::inverse_transform_sample;
///
/// let mut rng = Pcg32::seed_from_u64(5);
/// let x = inverse_transform_sample(|u| -1.0 + u * 2.0, &mut rng);
/// assert!(x >= -1.0 && x < 1.0);
/// ```
#[must_use]
pub fn inverse_transform_sample<R, F>(cdf_inverse: F, rng: &mut R) -> f64
where
    R: Rng,
    F: Fn(f64) -> f64,
{
    cdf_inverse(rng.next_f64())
}

/// Runs one step of a symmetric-proposal Metropolis-Hastings sampler.
///
/// Given the current state `current`, an (unnormalized) `target` log-density,
/// and a symmetric proposal `propose` that maps the current state to a
/// candidate (for example `current + step * (rng.next_f64() - 0.5)`), this
/// returns the *next* state. Because the proposal is symmetric its acceptance
/// ratio reduces to the ratio of target densities:
///
/// ```text
/// alpha = min(1, exp(target(candidate) - target(current)))
/// ```
///
/// The candidate is accepted with probability `alpha`, otherwise the walker
/// stays at `current`.
///
/// The caller is expected to drive the chain â€” calling this repeatedly,
/// recording states, and discarding an initial burn-in â€” which keeps the
/// function allocation-free and free of any convergence assumptions.
///
/// # Type parameters
///
/// * `R` â€” the random source, any [`Rng`].
/// * `F` â€” the target log-density `Fn(f64) -> f64`.
/// * `P` â€” the symmetric proposal `Fn(f64) -> f64`.
///
/// # Examples
///
/// ```
/// use tpt_zero_rand::{Pcg32, Rng, SeedableRng};
/// use tpt_zero_sampler::metropolis_hastings;
///
/// let mut rng = Pcg32::seed_from_u64(11);
/// // Sample a standard normal via a symmetric random-walk proposal.
/// let target = |x: f64| -0.5 * x * x;
/// let mut x = 0.0f64;
/// for _ in 0..200 {
///     let step = rng.next_f64() - 0.5;
///     x = metropolis_hastings(x, target, |c| c + step, &mut rng);
/// }
/// assert!(x.is_finite());
/// ```
#[must_use]
pub fn metropolis_hastings<R, F, P>(
    current: f64,
    target: F,
    propose: P,
    rng: &mut R,
) -> f64
where
    R: Rng,
    F: Fn(f64) -> f64,
    P: Fn(f64) -> f64,
{
    let candidate = propose(current);
    let log_current = target(current);
    let log_candidate = target(candidate);
    if !log_candidate.is_finite() {
        // Candidate has zero density; always reject to stay finite.
        return current;
    }
    let log_ratio = log_candidate - log_current;
    // alpha = min(1, exp(log_ratio)). Accept when rng < alpha. We compare
    // log(rng) < log_ratio to avoid an explicit exp, but a direct exp keeps
    // the dependency surface tiny and is exact enough here.
    let alpha = if log_ratio >= 0.0 {
        1.0
    } else {
        exp(log_ratio)
    };
    if rng.next_f64() < alpha {
        candidate
    } else {
        current
    }
}

/// Computes `e^x`, matching `f64::exp` which is unavailable on this crate's
/// `core`-only target.
///
/// Delegates to the shared, subnormal-safe [`out_zero_float`] implementation,
/// which handles the full range (including the subnormal underflow region) and
/// never returns a negative value for large negative `x`.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn exp(x: f64) -> f64 {
    out_zero_float::exp(x)
}

#[cfg(test)]
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::manual_range_contains,
    clippy::neg_cmp_op_on_partial_ord,
    clippy::float_equality_without_abs
)]
mod tests {
    use super::*;
    use tpt_zero_prob::distributions::Uniform;
    use tpt_zero_rand::{Pcg32, Rng, SeedableRng};

    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::manual_range_contains
    )]
    fn in_unit(v: f64) -> bool {
        v >= 0.0 && v < 1.0
    }

    #[test]
    fn rejection_sample_uniform_from_uniform() {
        let proposal = Uniform::new(0.0, 1.0).unwrap();
        let mut rng = Pcg32::seed_from_u64(1);
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for _ in 0..200 {
            let x = rejection_sample(
                |x| 0.9 * proposal.pdf(x),
                &proposal,
                1.0,
                &mut rng,
                100,
            )
            .unwrap();
            assert!(x >= 0.0 && x <= 1.0);
            min = min.min(x);
            max = max.max(x);
        }
        // The sample should actually cover a good fraction of [0,1] (it is not
        // collapsed to a point).
        assert!(max - min > 0.5, "samples too concentrated: {min}..{max}");
    }

    #[test]
    fn rejection_sample_rejects_invalid_envelope() {
        let proposal = Uniform::new(0.0, 1.0).unwrap();
        let mut rng = Pcg32::seed_from_u64(2);
        // A non-positive or non-finite envelope cannot dominate the target, so
        // the sampler gives up immediately rather than producing undefined
        // acceptance ratios.
        assert!(rejection_sample(|x| proposal.pdf(x), &proposal, 0.0, &mut rng, 10).is_none());
        assert!(rejection_sample(|x| proposal.pdf(x), &proposal, -1.0, &mut rng, 10).is_none());
        assert!(
            rejection_sample(|x| proposal.pdf(x), &proposal, f64::NAN, &mut rng, 10).is_none()
        );
    }

    #[test]
    fn inverse_transform_identity_is_uniform() {
        let mut rng = Pcg32::seed_from_u64(3);
        for _ in 0..1000 {
            let x = inverse_transform_sample(|u| u, &mut rng);
            assert!(in_unit(x));
        }
    }

    #[test]
    fn inverse_transform_uniform_ab() {
        let mut rng = Pcg32::seed_from_u64(4);
        for _ in 0..1000 {
            let x = inverse_transform_sample(|u| -2.0 + u * 4.0, &mut rng);
            assert!(x >= -2.0 && x < 2.0);
        }
    }

    #[test]
    fn metropolis_hastings_stays_finite_and_mixes() {
        let mut rng = Pcg32::seed_from_u64(11);
        let target = |x: f64| -0.5 * x * x; // standard normal log-density.
        let mut x = 0.0f64;
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for _ in 0..500 {
            let step = rng.next_f64() - 0.5;
            x = metropolis_hastings(x, target, |c| c + step, &mut rng);
            assert!(x.is_finite());
            min = min.min(x);
            max = max.max(x);
        }
        assert!(max - min > 0.5, "MH chain too concentrated: {min}..{max}");
    }

    #[test]
    fn metropolis_hastings_accepts_improving_moves() {
        let mut rng = Pcg32::seed_from_u64(99);
        // Deterministic target that strictly increases toward 0 from below.
        let target = |x: f64| -x * x;
        // Force the proposal to always move toward higher density by stepping
        // from a negative current toward 0; acceptance should be guaranteed.
        let x = metropolis_hastings(-1.0, target, |_c| -0.5, &mut rng);
        assert!((x - (-0.5)).abs() < 1e-9);
    }

    #[test]
    fn exp_matches_std_for_small_args() {
        // exp is exposed only in this module; validate against the analytic
        // value of e^1 and e^-1 within Taylor tolerance.
        assert!((exp(1.0) - core::f64::consts::E).abs() < 1e-9);
        assert!((exp(-1.0) - 1.0 / core::f64::consts::E).abs() < 1e-9);
    }
}
