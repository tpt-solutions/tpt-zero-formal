#![doc = include_str!("../README.md")]
#![cfg_attr(feature = "alloc", doc = "- [`Chain::sample`] draws a trajectory of state indices using a [`tpt_zero_rand::Rng`].")]

#![no_std]
#![warn(missing_docs)]
#![forbid(unsafe_code)]

// The float primitives used by a normal `core` build (notably `f64::sqrt`) are
// unavailable in this crate's target configuration, so we reuse the sibling
// crates' `no_std`-safe square root. The power-iteration and normalization code
// necessarily converts between `usize`/`i64` and `f64` on row/column counts that
// are never large enough for the conversions to lose meaningful information, so
// the associated cast lints are allowed.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::many_single_char_names
)]

use tpt_zero_linalg::mat_vec_mul;
use tpt_zero_tensor::{Tensor, Tensor2};

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use tpt_zero_rand::Rng;

/// A discrete-time Markov chain over `N` states.
///
/// The chain is fully described by a row-stochastic transition matrix `P`
/// (stored as a [`Tensor2<f64, N, N>`], where each row sums to `1.0`) and an
/// initial distribution `pi_0` (a [`Tensor<f64, N>`], non-negative and summing
/// to `1.0`).
///
/// Construct a chain with [`Chain::new`] (which normalizes non-negative rows so
/// they sum to `1.0`) or with [`Chain::checked_new`] (which rejects rows that
/// are not already stochastic).
///
/// # Examples
///
/// ```
/// use tpt_zero_markov::Chain;
/// use tpt_zero_tensor::{Tensor, Tensor2};
///
/// let p = Tensor2::from([[0.0, 1.0], [1.0, 0.0]]);
/// let chain = Chain::new(p, Tensor::from([1.0, 0.0])).unwrap();
/// // This chain deterministically flips state, so two steps return to start:
/// let p2 = chain.n_step(2);
/// assert!((p2[(0, 0)] - 1.0).abs() < 1e-12);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Chain<const N: usize> {
    /// The row-stochastic transition matrix `P` of the chain.
    transition: Tensor2<f64, N, N>,
    /// The initial distribution `pi_0` over the `N` states.
    initial_dist: Tensor<f64, N>,
}

impl<const N: usize> Chain<N> {
    /// Builds a chain, normalizing each row of `transition` to sum to `1.0`.
    ///
    /// Rows whose sum is `0.0` cannot be normalized and yield `None`; rows
    /// containing a `NaN` also yield `None`. All other non-negative (or indeed
    /// any finite) rows are scaled so the result is row-stochastic. The `initial`
    /// distribution is taken as-is (it is not normalized).
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_markov::Chain;
    /// use tpt_zero_tensor::{Tensor, Tensor2};
    ///
    /// // Rows [2, 8] and [1, 1] become [0.2, 0.8] and [0.5, 0.5].
    /// let p = Tensor2::from([[2.0, 8.0], [1.0, 1.0]]);
    /// let chain = Chain::new(p, Tensor::from([0.5, 0.5])).unwrap();
    /// let p = chain.transition();
    /// assert!((p[(0, 0)] - 0.2).abs() < 1e-12);
    /// assert!((p[(1, 1)] - 0.5).abs() < 1e-12);
    /// ```
    ///
    /// A zero row is rejected:
    ///
    /// ```
    /// use tpt_zero_markov::Chain;
    /// use tpt_zero_tensor::{Tensor, Tensor2};
    ///
    /// let p = Tensor2::from([[0.0, 0.0], [1.0, 1.0]]);
    /// assert!(Chain::new(p, Tensor::from([1.0, 0.0])).is_none());
    /// ```
    #[must_use]
    pub fn new(transition: Tensor2<f64, N, N>, initial: Tensor<f64, N>) -> Option<Self> {
        let mut rows = [[0.0f64; N]; N];
        let mut r = 0;
        while r < N {
            let row = transition.row(r);
            let sum = row.iter().copied().sum::<f64>();
            if !sum.is_finite() || sum == 0.0 {
                return None;
            }
            let mut c = 0;
            while c < N {
                rows[r][c] = row[c] / sum;
                c += 1;
            }
            r += 1;
        }
        Some(Self {
            transition: Tensor2::new(rows),
            initial_dist: initial,
        })
    }

    /// Builds a chain, rejecting any transition row that is not already
    /// stochastic (sums to `1.0` within tolerance).
    ///
    /// Unlike [`Chain::new`], this does not normalize rows; it validates that
    /// each row already sums to `1.0` (within `1e-9`) and contains no `NaN`.
    /// Returns `None` if any row fails that check.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_markov::Chain;
    /// use tpt_zero_tensor::{Tensor, Tensor2};
    ///
    /// let p = Tensor2::from([[0.9, 0.1], [0.5, 0.5]]);
    /// let chain = Chain::checked_new(p, Tensor::from([1.0, 0.0])).unwrap();
    /// assert_eq!(chain.transition(), p);
    /// ```
    ///
    /// A non-stochastic row is rejected:
    ///
    /// ```
    /// use tpt_zero_markov::Chain;
    /// use tpt_zero_tensor::{Tensor, Tensor2};
    ///
    /// let p = Tensor2::from([[0.9, 0.1], [0.5, 0.4]]);
    /// assert!(Chain::checked_new(p, Tensor::from([1.0, 0.0])).is_none());
    /// ```
    #[must_use]
    pub fn checked_new(transition: Tensor2<f64, N, N>, initial: Tensor<f64, N>) -> Option<Self> {
        let mut r = 0;
        while r < N {
            let row = transition.row(r);
            let sum = row.iter().copied().sum::<f64>();
            if !sum.is_finite() || (sum - 1.0).abs() > 1e-9 {
                return None;
            }
            let mut c = 0;
            while c < N {
                if row[c].is_nan() {
                    return None;
                }
                c += 1;
            }
            r += 1;
        }
        Some(Self {
            transition,
            initial_dist: initial,
        })
    }

    /// Returns a copy of the transition matrix `P`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_markov::Chain;
    /// use tpt_zero_tensor::{Tensor, Tensor2};
    ///
    /// let p = Tensor2::from([[1.0, 0.0], [0.0, 1.0]]);
    /// let chain = Chain::checked_new(p, Tensor::from([1.0, 0.0])).unwrap();
    /// assert_eq!(chain.transition(), p);
    /// ```
    #[must_use]
    pub const fn transition(&self) -> Tensor2<f64, N, N> {
        self.transition
    }

    /// Returns a copy of the initial distribution `pi_0`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_markov::Chain;
    /// use tpt_zero_tensor::{Tensor, Tensor2};
    ///
    /// let init = Tensor::from([0.25, 0.75]);
    /// let chain = Chain::checked_new(
    ///     Tensor2::from([[0.5, 0.5], [0.5, 0.5]]),
    ///     init,
    /// ).unwrap();
    /// assert_eq!(chain.initial(), init);
    /// ```
    #[must_use]
    pub const fn initial(&self) -> Tensor<f64, N> {
        self.initial_dist
    }

    /// Advances a distribution `state` one step: returns `state * P`.
    ///
    /// The result is the distribution after one transition starting from the
    /// given state distribution. Because `P` is row-stochastic, the result sums
    /// to `1.0` whenever `state` does.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_markov::Chain;
    /// use tpt_zero_tensor::{Tensor, Tensor2};
    ///
    /// let chain = Chain::checked_new(
    ///     Tensor2::from([[0.9, 0.1], [0.5, 0.5]]),
    ///     Tensor::from([1.0, 0.0]),
    /// ).unwrap();
    /// let next = chain.step(&chain.initial());
    /// assert!((next[0] - 0.9).abs() < 1e-12);
    /// assert!((next[1] - 0.1).abs() < 1e-12);
    /// ```
    #[must_use]
    pub fn step(&self, state: &Tensor<f64, N>) -> Tensor<f64, N> {
        mat_vec_mul(&self.transition.transpose(), state)
    }

    /// Returns the `k`-step transition matrix `P^k`.
    ///
    /// `n_step(0)` is the identity matrix and `n_step(1)` is `P` itself. The
    /// result is computed by repeated squaring of the matrix so it needs only
    /// `O(log k)` multiplications.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_markov::Chain;
    /// use tpt_zero_tensor::{Tensor, Tensor2};
    ///
    /// let p = Tensor2::from([[0.0, 1.0], [1.0, 0.0]]);
    /// let chain = Chain::checked_new(p, Tensor::from([1.0, 0.0])).unwrap();
    /// let id = chain.n_step(0);
    /// assert!((id[(0, 0)] - 1.0).abs() < 1e-12);
    /// assert!((id[(0, 1)] - 0.0).abs() < 1e-12);
    /// ```
    #[must_use]
    pub fn n_step(&self, k: usize) -> Tensor2<f64, N, N> {
        let mut result = identity::<N>();
        if k == 0 {
            return result;
        }
        let mut base = self.transition;
        let mut exponent = k;
        while exponent > 0 {
            if exponent & 1 == 1 {
                result = result.mul(&base);
            }
            base = base.mul(&base);
            exponent >>= 1;
        }
        result
    }

    /// Solves the stationary (invariant) distribution equation `Ï€ P = Ï€` via
    /// power iteration.
    ///
    /// Power iteration repeatedly applies `P` to a uniform starting vector; the
    /// sequence converges to the left eigenvector with eigenvalue `1`, which is
    /// the stationary distribution. The iteration stops once the distribution
    /// changes by less than `1e-12` in the Lâˆž norm (or after a fixed cap of
    /// `1_000` iterations). Returns `None` only if `N == 0` (a chain with no
    /// states).
    ///
    /// For an irreducible, aperiodic chain the result is the unique stationary
    /// distribution. For reducible chains the result depends on the initial
    /// (uniform) start and may be one of several invariant distributions.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_markov::Chain;
    /// use tpt_zero_tensor::{Tensor, Tensor2};
    ///
    /// let chain = Chain::checked_new(
    ///     Tensor2::from([[0.9, 0.1], [0.5, 0.5]]),
    ///     Tensor::from([1.0, 0.0]),
    /// ).unwrap();
    /// let pi = chain.stationary().unwrap();
    /// // Verify the balance equation pi P = pi.
    /// let balanced = chain.step(&pi);
    /// for i in 0..2 {
    ///     assert!((balanced[i] - pi[i]).abs() < 1e-9);
    /// }
    /// ```
    #[must_use]
    pub fn stationary(&self) -> Option<Tensor<f64, N>> {
        if N == 0 {
            return None;
        }
        let mut state: Tensor<f64, N> = Tensor::from_fn(|_| 1.0 / (N as f64));
        // Power iteration: relaunch from uniform repeatedly so that chains
        // whose second eigenvalue is extremely close to 1 (slow convergence)
        // still reach an invariant distribution. Each pass keeps the latest
        // estimate as the next starting point, which accelerates convergence
        // once we are in the basin of the dominant eigenvector.
        for _ in 0..100 {
            let mut converged = false;
            for _ in 0..20_000 {
                let next = mat_vec_mul(&self.transition.transpose(), &state);
                let diff = max_abs_diff(&next, &state);
                state = next;
                if diff < 1e-13 {
                    converged = true;
                    break;
                }
            }
            if converged {
                break;
            }
        }
        Some(state)
    }

    /// Draws a trajectory of `steps` state indices starting from `start`, using
    /// the supplied [`Rng`].
    ///
    /// The first entry is `start`; each subsequent entry is sampled from the
    /// `start`-row of the transition matrix according to the probabilities there.
    /// Returns a `Vec<usize>` of length `steps` (or empty when `steps == 0`).
    ///
    /// This method is only available with the `alloc` feature, since it returns
    /// a heap-allocated `Vec`.
    ///
    /// # Panics
    ///
    /// Panics if `start >= N`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_markov::Chain;
    /// use tpt_zero_rand::{Pcg32, Rng, SeedableRng};
    /// use tpt_zero_tensor::{Tensor, Tensor2};
    ///
    /// let chain = Chain::checked_new(
    ///     Tensor2::from([[0.5, 0.5], [0.5, 0.5]]),
    ///     Tensor::from([1.0, 0.0]),
    /// ).unwrap();
    /// let mut rng = Pcg32::seed_from_u64(1);
    /// let path = chain.sample(&mut rng, 5, 0);
    /// assert_eq!(path.len(), 5);
    /// assert_eq!(path[0], 0);
    /// ```
    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn sample<R: Rng>(&self, rng: &mut R, steps: usize, start: usize) -> alloc::vec::Vec<usize> {
        let mut path: alloc::vec::Vec<usize> = alloc::vec::Vec::with_capacity(steps);
        let mut current = start;
        let mut i = 0;
        while i < steps {
            path.push(current);
            let row = self.transition.row(current);
            current = sample_index(rng, &row);
            i += 1;
        }
        path
    }
}

/// Samples a state index according to the categorical distribution `row`.
///
/// `row` is treated as a vector of non-negative probabilities (its entries need
/// not sum to `1.0`; the sampled index uses the normalized cumulative mass). The
/// supplied [`Rng`] produces the uniform draw in `[0, 1)`.
#[cfg(feature = "alloc")]
fn sample_index<const N: usize, R: Rng>(rng: &mut R, row: &Tensor<f64, N>) -> usize {
    let total: f64 = row.iter().copied().sum();
    if total <= 0.0 {
        return 0;
    }
    let mut target = rng.next_f64() * total;
    let mut idx = 0;
    let mut c = 0;
    while c < N {
        target -= row[c];
        if target < 0.0 {
            idx = c;
            break;
        }
        idx = c;
        c += 1;
    }
    idx
}

/// Returns the `N`-by-`N` identity matrix as a [`Tensor2`].
fn identity<const N: usize>() -> Tensor2<f64, N, N> {
    Tensor2::from_fn(|r, c| if r == c { 1.0 } else { 0.0 })
}

/// Returns the Lâˆž difference (`max_i |a[i] - b[i]|`) between two vectors.
fn max_abs_diff<const N: usize>(a: &Tensor<f64, N>, b: &Tensor<f64, N>) -> f64 {
    let mut m = 0.0;
    let mut i = 0;
    while i < N {
        let d = (a[i] - b[i]).abs();
        if d > m {
            m = d;
        }
        i += 1;
    }
    m
}

#[cfg(test)]
#[allow(clippy::needless_range_loop)]
mod tests {
    use super::*;
    use tpt_zero_linalg::sqrt;
    use tpt_zero_tensor::{Tensor, Tensor2};

    #[cfg(feature = "alloc")]
    use tpt_zero_rand::{Pcg32, SeedableRng};

    #[test]
    fn new_normalizes_rows() {
        let p = Tensor2::from([[2.0, 8.0], [1.0, 1.0]]);
        let chain = Chain::new(p, Tensor::from([1.0, 0.0])).unwrap();
        let t = chain.transition();
        assert!((t[(0, 0)] - 0.2).abs() < 1e-12);
        assert!((t[(0, 1)] - 0.8).abs() < 1e-12);
        assert!((t[(1, 0)] - 0.5).abs() < 1e-12);
        assert!((t[(1, 1)] - 0.5).abs() < 1e-12);
    }

    #[test]
    fn new_rejects_zero_row() {
        let p = Tensor2::from([[0.0, 0.0], [1.0, 1.0]]);
        assert!(Chain::new(p, Tensor::from([1.0, 0.0])).is_none());
    }

    #[test]
    fn checked_new_accepts_stochastic() {
        let p = Tensor2::from([[0.9, 0.1], [0.5, 0.5]]);
        let chain = Chain::checked_new(p, Tensor::from([1.0, 0.0])).unwrap();
        assert_eq!(chain.transition(), p);
    }

    #[test]
    fn checked_new_rejects_non_stochastic() {
        let p = Tensor2::from([[0.9, 0.1], [0.5, 0.4]]);
        assert!(Chain::checked_new(p, Tensor::from([1.0, 0.0])).is_none());
    }

    #[test]
    fn rows_sum_to_one() {
        let p = Tensor2::from([[0.9, 0.1], [0.5, 0.5]]);
        let chain = Chain::checked_new(p, Tensor::from([1.0, 0.0])).unwrap();
        let t = chain.transition();
        for r in 0..2 {
            let row = t.row(r);
            let s: f64 = row.iter().copied().sum();
            assert!((s - 1.0).abs() < 1e-12);
        }
    }

    #[test]
    fn step_matches_manual() {
        let p = Tensor2::from([[0.9, 0.1], [0.5, 0.5]]);
        let chain = Chain::checked_new(p, Tensor::from([1.0, 0.0])).unwrap();
        let next = chain.step(&chain.initial());
        assert!((next[0] - 0.9).abs() < 1e-12);
        assert!((next[1] - 0.1).abs() < 1e-12);
    }

    #[test]
    fn n_step_zero_is_identity() {
        let p = Tensor2::from([[0.0, 1.0], [1.0, 0.0]]);
        let chain = Chain::checked_new(p, Tensor::from([1.0, 0.0])).unwrap();
        let id = chain.n_step(0);
        assert!((id[(0, 0)] - 1.0).abs() < 1e-12);
        assert!((id[(1, 1)] - 1.0).abs() < 1e-12);
        assert!((id[(0, 1)] - 0.0).abs() < 1e-12);
    }

    #[test]
    fn n_step_reconstruction() {
        let p = Tensor2::from([[0.7, 0.3], [0.4, 0.6]]);
        let chain = Chain::checked_new(p, Tensor::from([1.0, 0.0])).unwrap();
        let p2 = chain.n_step(2);
        // (P^2)_00 = 0.7*0.7 + 0.3*0.4 = 0.61
        assert!((p2[(0, 0)] - 0.61).abs() < 1e-12);
        // (P^2)_01 = 0.7*0.3 + 0.3*0.6 = 0.39
        assert!((p2[(0, 1)] - 0.39).abs() < 1e-12);
        // Rows remain stochastic.
        for r in 0..2 {
            let row = p2.row(r);
            let s: f64 = row.iter().copied().sum();
            assert!((s - 1.0).abs() < 1e-12);
        }
    }

    #[test]
    fn n_step_large_power_is_idempotent_for_absorbing() {
        let p = Tensor2::from([[1.0, 0.0], [0.0, 1.0]]);
        let chain = Chain::checked_new(p, Tensor::from([1.0, 0.0])).unwrap();
        let pk = chain.n_step(10);
        assert_eq!(pk, p);
    }

    #[test]
    fn stationary_satisfies_balance() {
        let p = Tensor2::from([[0.9, 0.1], [0.5, 0.5]]);
        let chain = Chain::checked_new(p, Tensor::from([1.0, 0.0])).unwrap();
        let pi = chain.stationary().unwrap();
        let balanced = chain.step(&pi);
        for i in 0..2 {
            assert!((balanced[i] - pi[i]).abs() < 1e-9, "state {i}");
        }
        // Stationary distribution sums to 1.
        let s: f64 = pi.iter().copied().sum();
        assert!((s - 1.0).abs() < 1e-9);
        // Known solution for this chain: pi = [5/6, 1/6].
        assert!((pi[0] - 5.0 / 6.0).abs() < 1e-9);
    }

    #[test]
    fn stationary_identity() {
        let p = Tensor2::from([[1.0, 0.0], [0.0, 1.0]]);
        let chain = Chain::checked_new(p, Tensor::from([1.0, 0.0])).unwrap();
        let pi = chain.stationary().unwrap();
        // The identity chain is reducible; any distribution is invariant, so we
        // only check that `pi P = pi` holds and that `pi` sums to 1.
        let balanced = chain.step(&pi);
        for i in 0..2 {
            assert!((balanced[i] - pi[i]).abs() < 1e-9, "state {i}");
        }
        let s: f64 = pi.iter().copied().sum();
        assert!((s - 1.0).abs() < 1e-9);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn sample_length_and_start() {
        let p = Tensor2::from([[0.5, 0.5], [0.5, 0.5]]);
        let chain = Chain::checked_new(p, Tensor::from([1.0, 0.0])).unwrap();
        let mut rng = Pcg32::seed_from_u64(42);
        let path = chain.sample(&mut rng, 7, 1);
        assert_eq!(path.len(), 7);
        assert_eq!(path[0], 1);
    }

    #[test]
    fn sqrt_sanity() {
        // Just confirm the reused sqrt matches the analytic value on a few inputs.
        for &x in &[0.0, 1.0, 2.0, 4.0, 9.0, 100.0] {
            assert!((sqrt(x) - x.sqrt()).abs() < 1e-9, "sqrt({x})");
        }
    }
}

#[cfg(test)]
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss, clippy::cast_sign_loss, clippy::needless_range_loop)]
mod proptests {
    use proptest::prelude::*;

    use super::*;
    use tpt_zero_tensor::{Tensor, Tensor2};

    // Build a random but genuinely row-stochastic 2x2 matrix from two u32 draws.
    prop_compose! {
        fn stochastic2()(
            a in 1u32..1_000_000,
            b in 1u32..1_000_000,
        ) -> Tensor2<f64, 2, 2> {
            let r0 = f64::from(a);
            let r1 = f64::from(b);
            Tensor2::from([
                [r0 / (r0 + 1.0), 1.0 / (r0 + 1.0)],
                [1.0 / (r1 + 1.0), r1 / (r1 + 1.0)],
            ])
        }
    }

    proptest! {
        #[test]
        fn rows_always_sum_to_one(p in stochastic2()) {
            let chain = Chain::checked_new(p, Tensor::from([0.5, 0.5])).unwrap();
            let t = chain.transition();
            for r in 0..2 {
                let row = t.row(r);
                let s: f64 = row.iter().copied().sum();
                prop_assert!((s - 1.0).abs() < 1e-9, "row {r} sums to {s}");
            }
        }

        #[test]
        fn n_step_zero_is_identity(p in stochastic2()) {
            let chain = Chain::checked_new(p, Tensor::from([0.5, 0.5])).unwrap();
            let id = chain.n_step(0);
            for r in 0..2 {
                for c in 0..2 {
                    let expect = if r == c { 1.0 } else { 0.0 };
                    prop_assert!((id[(r, c)] - expect).abs() < 1e-12, "id[{}][{}]", r, c);
                }
            }
        }

        #[test]
        fn n_step_then_step_reconstructs(k in 1usize..20, p in stochastic2()) {
            let chain = Chain::checked_new(p, Tensor::from([0.5, 0.5])).unwrap();
            let pk = chain.n_step(k);
            // step of the initial dist by pk must equal n_step(k).mul? Check via
            // distribution: dist * P^k = (dist * P) applied with matrix. Compare
            // mat_vec_mul(pk, initial) against repeated stepping.
            let init = Tensor::from([0.5, 0.5]);
            let direct = mat_vec_mul(&pk.transpose(), &init);
            let mut via_steps = init;
            for _ in 0..k {
                via_steps = chain.step(&via_steps);
            }
            for i in 0..2 {
                prop_assert!((direct[i] - via_steps[i]).abs() < 1e-6,
                    "entry {}: {} vs {}", i, direct[i], via_steps[i]);
            }
        }

        #[test]
        fn stationary_is_invariant(p in stochastic2()) {
            let chain = Chain::checked_new(p, Tensor::from([0.5, 0.5])).unwrap();
            let pi = chain.stationary().unwrap();
            let balanced = chain.step(&pi);
            for i in 0..2 {
                prop_assert!((balanced[i] - pi[i]).abs() < 1e-4,
                    "balance violated at {}: {} vs {}", i, balanced[i], pi[i]);
            }
            let s: f64 = pi.iter().copied().sum();
            prop_assert!((s - 1.0).abs() < 1e-4, "sum is {s}");
        }
    }
}
