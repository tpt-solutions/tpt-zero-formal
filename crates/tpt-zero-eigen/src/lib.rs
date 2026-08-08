#![doc = include_str!("../README.md")]

#![no_std]
#![warn(missing_docs)]
#![forbid(unsafe_code)]

// Numerical kernels iterate over small fixed-size matrices with single-letter
// index names and copy a matrix into a local nested array; the casts are on
// data-sized counts that never lose meaningful information, and the `sqrt`
// helper necessarily performs float casts. All are idiomatic here and allowed.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::comparison_chain
)]

use tpt_zero_linalg::mat_vec_mul;
use tpt_zero_solver::solve_gaussian;
use tpt_zero_tensor::{Tensor, Tensor2};

/// Computes the square root of `x >= 0.0`.
///
/// Returns `f64::NAN` for negative inputs and for `f64::NAN`. The
/// implementation lives in [`out_zero_float`]; it is subnormal-safe and uses a
/// relative convergence tolerance, so it remains accurate for both tiny and
/// huge magnitudes.
///
/// # Examples
///
/// ```
/// use tpt_zero_eigen::sqrt;
///
/// assert!((sqrt(2.0) - core::f64::consts::SQRT_2).abs() < 1e-12);
/// assert_eq!(sqrt(0.0), 0.0);
/// assert!(sqrt(-1.0).is_nan());
/// ```
#[must_use]
pub fn sqrt(x: f64) -> f64 {
    out_zero_float::sqrt(x)
}

/// Returns the Euclidean (L2) norm of `v`: `sqrt(Î£ v[i]Â²)`.
///
/// This is a local copy of the [`tpt_zero_linalg`] norm so that callers of this
/// crate do not pay a transitive dependency cost for a single helper; it uses
/// the crate's own [`sqrt`].
///
/// # Examples
///
/// ```
/// use tpt_zero_eigen::norm_l2;
/// use tpt_zero_tensor::Tensor;
///
/// let v = Tensor::from([3.0, 4.0]);
/// assert!((norm_l2(&v) - 5.0).abs() < 1e-12);
/// ```
#[must_use]
pub fn norm_l2<const N: usize>(v: &Tensor<f64, N>) -> f64 {
    let mut sum = 0.0;
    let mut i = 0;
    while i < N {
        sum += v[i] * v[i];
        i += 1;
    }
    sqrt(sum)
}

/// Returns `v` scaled to unit length (`v / â€–vâ€–`), or `None` if `v` is the zero
/// vector.
///
/// The returned vector satisfies `norm_l2(&result) == 1.0` whenever it is
/// `Some`.
///
/// # Examples
///
/// ```
/// use tpt_zero_eigen::{norm_l2, normalize};
/// use tpt_zero_tensor::Tensor;
///
/// let v = Tensor::from([3.0, 4.0]);
/// let n = normalize(&v).unwrap();
/// assert!((norm_l2(&n) - 1.0).abs() < 1e-12);
///
/// let zero = Tensor::from([0.0, 0.0]);
/// assert_eq!(normalize(&zero), None);
/// ```
#[must_use]
pub fn normalize<const N: usize>(v: &Tensor<f64, N>) -> Option<Tensor<f64, N>> {
    let len = norm_l2(v);
    if len == 0.0 {
        return None;
    }
    Some(Tensor::new(core::array::from_fn(|i| v[i] / len)))
}

/// Computes the Rayleigh quotient `váµ€ A v / (váµ€ v)` of a symmetric (or general)
/// matrix `A` with respect to a non-zero vector `v`.
///
/// For an eigenvector `v` of `A` the quotient equals the associated eigenvalue,
/// and it gives the best scalar eigenvalue estimate for any trial vector `v`.
///
/// Returns `f64::NAN` if `v` is the zero vector (the quotient is undefined).
///
/// # Examples
///
/// ```
/// use tpt_zero_eigen::rayleigh_quotient;
/// use tpt_zero_tensor::{Tensor, Tensor2};
///
/// // Identity matrix: every vector is an eigenvector with eigenvalue 1.
/// let a = Tensor2::from([[1.0, 0.0], [0.0, 1.0]]);
/// let v = Tensor::from([2.0, 3.0]);
/// assert!((rayleigh_quotient(&a, &v) - 1.0).abs() < 1e-12);
/// ```
#[must_use]
pub fn rayleigh_quotient<const N: usize>(a: &Tensor2<f64, N, N>, v: &Tensor<f64, N>) -> f64 {
    let av = mat_vec_mul(a, v);
    let mut num = 0.0;
    let mut den = 0.0;
    let mut i = 0;
    while i < N {
        num += v[i] * av[i];
        den += v[i] * v[i];
        i += 1;
    }
    if den == 0.0 {
        return f64::NAN;
    }
    num / den
}

/// Estimates the dominant (largest-magnitude) eigenvalue of an `N`-by-`N`
/// matrix `a` via the power iteration method.
///
/// Starting from a random-ish initial vector, the iteration repeatedly applies
/// `v â† a v` and normalizes, converging to the eigenvector associated with the
/// eigenvalue of largest magnitude. The returned eigenvalue is the Rayleigh
/// quotient of the final eigenvector estimate (so it equals `váµ€ a v / váµ€ v`).
///
/// Returns `(lambda, v)` where `lambda` is the dominant eigenvalue and `v` is a
/// corresponding unit eigenvector, satisfying `a v â‰ˆ lambda v`.
///
/// # Arguments
///
/// - `a` â€” the square matrix.
/// - `tol` â€” convergence tolerance on the change in the eigenvalue estimate.
/// - `max_iter` â€” upper bound on the number of iterations.
///
/// # Examples
///
/// ```
/// use tpt_zero_eigen::power_iteration;
/// use tpt_zero_tensor::Tensor2;
///
/// let a = Tensor2::from([[2.0, 1.0], [1.0, 2.0]]);
/// let (lambda, v) = power_iteration(&a, 1e-12, 1000);
/// // Dominant eigenvalue is 3.
/// assert!((lambda - 3.0).abs() < 1e-9);
/// // Verify A v â‰ˆ lambda v.
/// let av = tpt_zero_linalg::mat_vec_mul(&a, &v);
/// let lv = v.map(|x| x * lambda);
/// for i in 0..2 {
///     assert!((av[i] - lv[i]).abs() < 1e-8);
/// }
/// ```
#[must_use]
pub fn power_iteration<const N: usize>(
    a: &Tensor2<f64, N, N>,
    tol: f64,
    max_iter: usize,
) -> (f64, Tensor<f64, N>) {
    // Seed with a non-zero vector (all ones, which is robust for generic
    // matrices and symmetric so it cannot be orthogonal to the dominant
    // eigenspace in pathological ways).
    let mut v = Tensor::new(core::array::from_fn(|_| 1.0));
    let mut lambda = 0.0;

    let mut iter = 0;
    while iter < max_iter {
        let av = mat_vec_mul(a, &v);
        // Normalize via the L2 norm so we keep a unit eigenvector estimate.
        let norm = norm_l2(&av);
        if norm == 0.0 {
            // a v == 0: v is an eigenvector with eigenvalue 0.
            return (0.0, v);
        }
        let next = Tensor::new(core::array::from_fn(|i| av[i] / norm));
        let next_lambda = rayleigh_quotient(a, &next);
        // Converge on the change of the *eigenvector*: the Rayleigh quotient
        // can stabilise long before the vector does, so gating on the vector
        // keeps the returned pair `a v â‰ˆ lambda v` accurate to `tol`.
        let converged = iter > 0 && vec_inf_dist(&next, &v) < tol;
        v = next;
        lambda = next_lambda;
        if converged {
            break;
        }
        iter += 1;
    }

    (lambda, v)
}

/// Returns the infinity norm of `x - y` (`max_i |x[i] - y[i]|`).
fn vec_inf_dist<const N: usize>(x: &Tensor<f64, N>, y: &Tensor<f64, N>) -> f64 {
    let mut d = 0.0;
    let mut i = 0;
    while i < N {
        let e = (x[i] - y[i]).abs();
        if e > d {
            d = e;
        }
        i += 1;
    }
    d
}

/// Estimates the smallest-magnitude eigenvalue of an `N`-by-`N` matrix `a` via
/// inverse iteration.
///
/// Inverse iteration applies power iteration to `Aâ»Â¹`, whose dominant
/// eigenvalue is the reciprocal of `A`'s smallest-magnitude eigenvalue. Each
/// step solves `A w = v` (via [`tpt_zero_solver::solve_gaussian`]) and
/// normalizes `w`; this converges to the eigenvector of the eigenvalue closest
/// to zero. The returned eigenvalue is extracted with the Rayleigh quotient.
///
/// Returns `Some((lambda, v))` on success, or `None` if `A` is singular (so the
/// smallest-magnitude eigenvalue is exactly zero and the linear solve fails) or
/// if iteration does not start from a non-zero vector.
///
/// # Arguments
///
/// - `a` â€” the square matrix (must be non-singular).
/// - `tol` â€” convergence tolerance on the change in the eigenvalue estimate.
/// - `max_iter` â€” upper bound on the number of iterations.
///
/// # Examples
///
/// ```
/// use tpt_zero_eigen::inverse_iteration;
/// use tpt_zero_tensor::Tensor2;
///
/// let a = Tensor2::from([[2.0, 0.0], [0.0, 5.0]]);
/// let (lambda, v) = inverse_iteration(&a, 1e-12, 1000).unwrap();
/// // Smallest-magnitude eigenvalue is 2.
/// assert!((lambda - 2.0).abs() < 1e-8);
/// ```
#[must_use]
pub fn inverse_iteration<const N: usize>(
    a: &Tensor2<f64, N, N>,
    tol: f64,
    max_iter: usize,
) -> Option<(f64, Tensor<f64, N>)> {
    let mut v = Tensor::new(core::array::from_fn(|_| 1.0));
    let mut lambda = 0.0;

    let mut iter = 0;
    while iter < max_iter {
        // Solve A w = v; w is the candidate next vector.
        let w = solve_gaussian(a, &v)?;
        let norm = norm_l2(&w);
        if norm == 0.0 {
            return None;
        }
        let next = Tensor::new(core::array::from_fn(|i| w[i] / norm));
        let next_lambda = rayleigh_quotient(a, &next);
        let converged = iter > 0 && vec_inf_dist(&next, &v) < tol;
        v = next;
        lambda = next_lambda;
        if converged {
            break;
        }
        iter += 1;
    }

    Some((lambda, v))
}

/// Returns the two eigenvalues of a `2`-by-`2` matrix from the closed-form
/// roots of its characteristic equation.
///
/// For `A = [[a, b], [c, d]]`, the eigenvalues are the roots of
/// `Î»Â² âˆ’ (a+d) Î» + (ad âˆ’ bc) = 0`, given by
/// `Î» = (tr Â± sqrt(trÂ² âˆ’ 4 det)) / 2` where `tr = a+d` and `det = ad âˆ’ bc`.
/// The two returned values satisfy `Î»1 + Î»2 == tr` and `Î»1 * Î»2 == det`.
///
/// # Examples
///
/// ```
/// use tpt_zero_eigen::eigenvalues_2x2;
/// use tpt_zero_tensor::Tensor2;
///
/// let a = Tensor2::from([[2.0, 1.0], [1.0, 2.0]]);
/// let ev = eigenvalues_2x2(&a);
/// // 3 and 1, in either order.
/// let sum = ev[0] + ev[1];
/// let prod = ev[0] * ev[1];
/// assert!((sum - 4.0).abs() < 1e-12);
/// assert!((prod - 3.0).abs() < 1e-12);
/// ```
#[must_use]
pub fn eigenvalues_2x2(a: &Tensor2<f64, 2, 2>) -> [f64; 2] {
    let a00 = a[(0, 0)];
    let a01 = a[(0, 1)];
    let a10 = a[(1, 0)];
    let a11 = a[(1, 1)];

    let tr = a00 + a11;
    let det = a00 * a11 - a01 * a10;

    // Discriminant of Î»Â² âˆ’ tr Î» + det = 0.
    let disc = tr * tr - 4.0 * det;
    let root = sqrt(disc.max(0.0));

    // Stable ordering: larger root first.
    let hi = (tr + root) / 2.0;
    let lo = (tr - root) / 2.0;
    [hi, lo]
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_zero_tensor::Tensor2;

    const TOL: f64 = 1e-9;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() <= TOL
    }

    #[test]
    fn sqrt_matches_reference() {
        for &x in &[0.0, 1.0, 2.0, 4.0, 9.0, 25.0, 100.0, 1234.5] {
            assert!((sqrt(x) - x.sqrt()).abs() < 1e-9, "sqrt({x})");
        }
        assert!(sqrt(-1.0).is_nan());
        assert!(sqrt(f64::NAN).is_nan());
    }

    #[test]
    fn norm_and_normalize() {
        let v = Tensor::from([3.0, 4.0]);
        assert!(approx(norm_l2(&v), 5.0));
        let n = normalize(&v).unwrap();
        assert!(approx(norm_l2(&n), 1.0));
        let zero = Tensor::from([0.0, 0.0]);
        assert_eq!(normalize(&zero), None);
    }

    #[test]
    fn rayleigh_identity() {
        let a = Tensor2::from([[1.0, 0.0], [0.0, 1.0]]);
        let v = Tensor::from([2.0, 3.0]);
        assert!(approx(rayleigh_quotient(&a, &v), 1.0));
    }

    #[test]
    fn rayleigh_zero_vector_is_nan() {
        let a = Tensor2::from([[1.0, 0.0], [0.0, 1.0]]);
        let v = Tensor::from([0.0, 0.0]);
        assert!(rayleigh_quotient(&a, &v).is_nan());
    }

    #[test]
    fn power_iteration_diagonal() {
        // Dominant eigenvalue 5, eigenvector [0, 1].
        let a = Tensor2::from([[2.0, 0.0], [0.0, 5.0]]);
        let (lambda, v) = power_iteration(&a, 1e-12, 1000);
        assert!(approx(lambda, 5.0));
        let av = mat_vec_mul(&a, &v);
        let lv = v.map(|x| x * lambda);
        assert!(approx(av[0], lv[0]));
        assert!(approx(av[1], lv[1]));
    }

    #[test]
    fn power_iteration_symmetric() {
        let a = Tensor2::from([[2.0, 1.0], [1.0, 2.0]]);
        let (lambda, v) = power_iteration(&a, 1e-12, 2000);
        assert!(approx(lambda, 3.0));
        let av = mat_vec_mul(&a, &v);
        let lv = v.map(|x| x * lambda);
        assert!(approx(av[0], lv[0]));
        assert!(approx(av[1], lv[1]));
    }

    #[test]
    fn power_iteration_unit_vector() {
        // Eigenpair satisfies A v = lambda v to high accuracy.
        let a = Tensor2::from([[4.0, 1.0], [1.0, 3.0]]);
        let (lambda, v) = power_iteration(&a, 1e-12, 2000);
        let av = mat_vec_mul(&a, &v);
        let lv = v.map(|x| x * lambda);
        assert!(approx(av[0], lv[0]));
        assert!(approx(av[1], lv[1]));
    }

    #[test]
    fn inverse_iteration_diagonal() {
        // Smallest-magnitude eigenvalue 2, eigenvector [1, 0].
        let a = Tensor2::from([[2.0, 0.0], [0.0, 5.0]]);
        let (lambda, v) = inverse_iteration(&a, 1e-12, 1000).unwrap();
        assert!(approx(lambda, 2.0));
        let av = mat_vec_mul(&a, &v);
        let lv = v.map(|x| x * lambda);
        assert!(approx(av[0], lv[0]));
        assert!(approx(av[1], lv[1]));
    }

    #[test]
    fn inverse_iteration_unit_vector() {
        let a = Tensor2::from([[4.0, 1.0], [1.0, 3.0]]);
        let (lambda, v) = inverse_iteration(&a, 1e-12, 2000).unwrap();
        let av = mat_vec_mul(&a, &v);
        let lv = v.map(|x| x * lambda);
        assert!(approx(av[0], lv[0]));
        assert!(approx(av[1], lv[1]));
    }

    #[test]
    fn eigenvalues_2x2_identities() {
        let a = Tensor2::from([[2.0, 1.0], [1.0, 2.0]]);
        let ev = eigenvalues_2x2(&a);
        // trace = 4, det = 3 -> roots 3 and 1.
        assert!(approx(ev[0] + ev[1], 4.0));
        assert!(approx(ev[0] * ev[1], 3.0));
    }

    #[test]
    fn eigenvalues_2x2_off_diagonal() {
        let a = Tensor2::from([[1.0, 2.0], [2.0, 1.0]]);
        let ev = eigenvalues_2x2(&a);
        // trace = 2, det = -3 -> roots 3 and -1.
        assert!(approx(ev[0] + ev[1], 2.0));
        assert!(approx(ev[0] * ev[1], -3.0));
    }

    #[test]
    fn eigenvalues_2x2_repeated() {
        let a = Tensor2::from([[5.0, 0.0], [0.0, 5.0]]);
        let ev = eigenvalues_2x2(&a);
        assert!(approx(ev[0], 5.0));
        assert!(approx(ev[1], 5.0));
    }
}

#[cfg(test)]
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
mod proptests {
    use proptest::prelude::*;

    use super::*;
    use tpt_zero_tensor::Tensor2;

    const TOL: f64 = 1e-6;

    fn mat2() -> impl Strategy<Value = Tensor2<f64, 2, 2>> {
        proptest::array::uniform2(proptest::array::uniform2(-100i64..100))
            .prop_map(|rows| Tensor2::new(rows.map(|r| r.map(|x| x as f64))))
    }

    /// A 2x2 matrix whose discriminant `trÂ² âˆ’ 4 det` is non-negative, so its
    /// eigenvalues are real (power iteration then converges to a real eigenpair
    /// and the closed-form roots are real).
    fn mat2_real_eigs() -> impl Strategy<Value = Tensor2<f64, 2, 2>> {
        mat2().prop_filter("eigenvalues must be real", |m| {
            let tr = m[(0, 0)] + m[(1, 1)];
            let det = m[(0, 0)] * m[(1, 1)] - m[(0, 1)] * m[(1, 0)];
            tr * tr - 4.0 * det >= 0.0
        })
    }

    /// A 2x2 matrix with a *unique* dominant eigenvalue in magnitude, so power
    /// iteration converges. The eigenvalues must be real and one must strictly
    /// dominate the other in absolute value.
    fn mat2_unique_dominant() -> impl Strategy<Value = Tensor2<f64, 2, 2>> {
        mat2().prop_filter("needs real, uniquely dominant eigenvalue", |m| {
            let tr = m[(0, 0)] + m[(1, 1)];
            let det = m[(0, 0)] * m[(1, 1)] - m[(0, 1)] * m[(1, 0)];
            let disc = tr * tr - 4.0 * det;
            if disc < 0.0 {
                return false;
            }
            let root = sqrt(disc);
            let e1 = (tr + root) / 2.0;
            let e2 = (tr - root) / 2.0;
            e1.abs() > e2.abs() * (1.0 + 1e-9)
        })
    }

    proptest! {
        #[test]
        fn eigenvalues_satisfy_trace_det_identities(a in mat2_real_eigs()) {
            let ev = eigenvalues_2x2(&a);
            let tr = a[(0, 0)] + a[(1, 1)];
            let det = a[(0, 0)] * a[(1, 1)] - a[(0, 1)] * a[(1, 0)];
            prop_assert!((ev[0] + ev[1] - tr).abs() <= TOL, "trace: {} vs {}", ev[0] + ev[1], tr);
            prop_assert!((ev[0] * ev[1] - det).abs() <= TOL, "det: {} vs {}", ev[0] * ev[1], det);
        }

        #[test]
        fn power_iteration_is_eigenpair(a in mat2_unique_dominant()) {
            let (lambda, v) = power_iteration(&a, 1e-10, 5000);
            let av = mat_vec_mul(&a, &v);
            let lv = v.map(|x| x * lambda);
            prop_assert!((av[0] - lv[0]).abs() <= 1e-4, "row0: {} vs {}", av[0], lv[0]);
            prop_assert!((av[1] - lv[1]).abs() <= 1e-4, "row1: {} vs {}", av[1], lv[1]);
        }
    }
}
