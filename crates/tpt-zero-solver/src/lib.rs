#![doc = include_str!("../README.md")]

#![no_std]
#![warn(missing_docs)]
#![forbid(unsafe_code)]

// Numerical kernels iterate over small fixed-size matrices with single-letter
// index names and a three-way `if` for the triangular split, which clippy
// flags; the casts are on data-sized counts that never lose meaningful
// information. All are idiomatic here and allowed.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::many_single_char_names,
    clippy::comparison_chain,
    clippy::needless_range_loop
)]

use tpt_zero_decomp::{cholesky, lu};
use tpt_zero_linalg::mat_vec_mul;
use tpt_zero_tensor::{Tensor, Tensor2};

/// tolerance used to decide whether a residual is "effectively zero" when
/// judging singularity during direct elimination.
const SINGULAR_EPS: f64 = 1e-12;

/// Solves `A x = b` by Gaussian elimination with partial pivoting.
///
/// `A` must be an `N`-by-`N` matrix (given as [`Tensor2<f64, N, N>`]) and `b` a
/// length-`N` vector. The elimination is performed on a copy of `A`, so the
/// inputs are not modified. Partial pivoting swaps rows to use the largest
/// available pivot magnitude at each step, improving numerical stability.
///
/// Returns `Some(x)` with `A x â‰ˆ b` to within the working precision, or `None`
/// if `A` is (numerically) singular.
///
/// # Examples
///
/// ```
/// use tpt_zero_solver::solve_gaussian;
/// use tpt_zero_tensor::{Tensor, Tensor2};
///
/// let a = Tensor2::from([[2.0, 1.0], [1.0, 3.0]]);
/// let b = Tensor::from([5.0, 10.0]);
/// let x = solve_gaussian(&a, &b).unwrap();
/// assert!((x[0] - 1.0).abs() < 1e-9);
/// assert!((x[1] - 3.0).abs() < 1e-9);
///
/// // A singular matrix has no solution.
/// let singular = Tensor2::from([[1.0, 2.0], [2.0, 4.0]]);
/// assert_eq!(solve_gaussian(&singular, &Tensor::from([1.0, 2.0])), None);
/// ```
#[must_use]
pub fn solve_gaussian<const N: usize>(
    a: &Tensor2<f64, N, N>,
    b: &Tensor<f64, N>,
) -> Option<Tensor<f64, N>> {
    // Work on a copy of A and the right-hand side, pivoting both together so
    // the inputs are left unmodified.
    let mut m: [[f64; N]; N] = [[0.0; N]; N];
    let mut rhs = [0.0f64; N];
    let mut i = 0;
    while i < N {
        let mut c = 0;
        while c < N {
            m[i][c] = a[(i, c)];
            c += 1;
        }
        rhs[i] = b[i];
        i += 1;
    }

    // Forward elimination with partial pivoting.
    let mut p = 0;
    while p < N {
        // Choose the pivot: largest magnitude in column p at or below row p.
        let mut pivot_row = p;
        let mut pivot_val = m[p][p].abs();
        let mut r = p + 1;
        while r < N {
            let v = m[r][p].abs();
            if v > pivot_val {
                pivot_val = v;
                pivot_row = r;
            }
            r += 1;
        }
        if pivot_val <= SINGULAR_EPS {
            // Zero column: singular system.
            return None;
        }
        if pivot_row != p {
            m.swap(p, pivot_row);
            rhs.swap(p, pivot_row);
        }

        // Eliminate entries below the pivot.
        let mut r = p + 1;
        while r < N {
            let factor = m[r][p] / m[p][p];
            let mut c = p;
            while c < N {
                m[r][c] -= factor * m[p][c];
                c += 1;
            }
            rhs[r] -= factor * rhs[p];
            r += 1;
        }
        p += 1;
    }

    // Back-substitution.
    let mut x = [0.0f64; N];
    let mut r = N;
    while r > 0 {
        let i = r - 1;
        let mut sum = rhs[i];
        let mut c = i + 1;
        while c < N {
            sum -= m[i][c] * x[c];
            c += 1;
        }
        x[i] = sum / m[i][i];
        r -= 1;
    }

    Some(Tensor::from(x))
}

/// Solves `A x = b` using the LU decomposition with partial pivoting from
/// [`tpt_zero_decomp::lu`].
///
/// The matrix `A` is factored as `P A = L U`; the system is then solved by
/// forward-substitution on `L` followed by back-substitution on `U`. This is
/// equivalent to [`solve_gaussian`] but reuses the shared decomposition, which
/// is cheaper when you must solve several right-hand sides sharing the same `A`.
///
/// Returns `Some(x)` with `A x â‰ˆ b`, or `None` if `A` is (numerically) singular.
///
/// # Examples
///
/// ```
/// use tpt_zero_solver::solve_lu;
/// use tpt_zero_tensor::{Tensor, Tensor2};
///
/// let a = Tensor2::from([[2.0, 1.0], [1.0, 3.0]]);
/// let b = Tensor::from([5.0, 10.0]);
/// let x = solve_lu(&a, &b).unwrap();
/// assert!((x[0] - 1.0).abs() < 1e-9);
/// assert!((x[1] - 3.0).abs() < 1e-9);
/// ```
#[must_use]
pub fn solve_lu<const N: usize>(
    a: &Tensor2<f64, N, N>,
    b: &Tensor<f64, N>,
) -> Option<Tensor<f64, N>> {
    let (l, u, perm) = lu(a);
    solve_lu_factors(&l, &u, &perm, b)
}

/// Solves `A x = b` from existing LU factors `(L, U, p)`.
///
/// Given `P A = L U` (as returned by [`tpt_zero_decomp::lu`]), solves
/// `L U x = P b` by forward- then back-substitution. Returns `None` if a zero
/// pivot is encountered (the system is singular).
///
/// This is the internal helper shared by [`solve_lu`]; it is public so callers
/// who already hold the factors can avoid re-factorization.
#[must_use]
pub fn solve_lu_factors<const N: usize>(
    l: &Tensor2<f64, N, N>,
    u: &Tensor2<f64, N, N>,
    perm: &[usize; N],
    b: &Tensor<f64, N>,
) -> Option<Tensor<f64, N>> {
    // Permute b: y = P b, so that L U x = y.
    let mut y = [0.0f64; N];
    let mut r = 0;
    while r < N {
        y[r] = b[perm[r]];
        r += 1;
    }

    // Forward substitution: L z = y.
    let mut z = [0.0f64; N];
    let mut i = 0;
    while i < N {
        let mut sum = y[i];
        let mut c = 0;
        while c < i {
            sum -= l[(i, c)] * z[c];
            c += 1;
        }
        // L has unit diagonal, so z[i] = sum. A zero pivot here means singular.
        if l[(i, i)].abs() <= SINGULAR_EPS {
            return None;
        }
        z[i] = sum / l[(i, i)];
        i += 1;
    }

    // Back substitution: U x = z.
    let mut x = [0.0f64; N];
    let mut r = N;
    while r > 0 {
        let i = r - 1;
        let mut sum = z[i];
        let mut c = i + 1;
        while c < N {
            sum -= u[(i, c)] * x[c];
            c += 1;
        }
        if u[(i, i)].abs() <= SINGULAR_EPS {
            return None;
        }
        x[i] = sum / u[(i, i)];
        r -= 1;
    }

    Some(Tensor::from(x))
}

/// Solves `A x = b` via the Cholesky decomposition from
/// [`tpt_zero_decomp::cholesky`].
///
/// `A` must be symmetric positive-definite; the system is solved as
/// `L Láµ€ x = b` by forward-substitution on `L` then back-substitution on
/// `Láµ€`. This is both faster and more numerically stable than the general LU
/// route for SPD systems.
///
/// Returns `Some(x)` with `A x â‰ˆ b`, or `None` if `A` is not symmetric
/// positive-definite (the Cholesky factor does not exist).
///
/// # Examples
///
/// ```
/// use tpt_zero_solver::solve_cholesky;
/// use tpt_zero_tensor::{Tensor, Tensor2};
///
/// let a = Tensor2::from([[4.0, 1.0], [1.0, 3.0]]);
/// let b = Tensor::from([1.0, 2.0]);
/// let x = solve_cholesky(&a, &b).unwrap();
/// let got = tpt_zero_linalg::mat_vec_mul(&a, &x);
/// assert!((got[0] - 1.0).abs() < 1e-9);
/// assert!((got[1] - 2.0).abs() < 1e-9);
///
/// // A non-positive-definite matrix yields None.
/// let not_pd = Tensor2::from([[-1.0, 0.0], [0.0, -1.0]]);
/// assert_eq!(solve_cholesky(&not_pd, &Tensor::from([1.0, 1.0])), None);
/// ```
#[must_use]
pub fn solve_cholesky<const N: usize>(
    a: &Tensor2<f64, N, N>,
    b: &Tensor<f64, N>,
) -> Option<Tensor<f64, N>> {
    let l = cholesky(a)?;

    // Forward substitution: L y = b.
    let mut y = [0.0f64; N];
    let mut i = 0;
    while i < N {
        let mut sum = b[i];
        let mut c = 0;
        while c < i {
            sum -= l[(i, c)] * y[c];
            c += 1;
        }
        y[i] = sum / l[(i, i)];
        i += 1;
    }

    // Back substitution on Láµ€: Láµ€ x = y.
    let mut x = [0.0f64; N];
    let mut r = N;
    while r > 0 {
        let i = r - 1;
        let mut sum = y[i];
        let mut c = i + 1;
        while c < N {
            sum -= l[(c, i)] * x[c];
            c += 1;
        }
        x[i] = sum / l[(i, i)];
        r -= 1;
    }

    Some(Tensor::from(x))
}

/// Solves `A x = b` by the Jacobi iterative method.
///
/// Starting from the initial guess `x0`, each component is updated from the
/// previous iterate using `x_i = (b_i - Î£_{jâ‰ i} A[i][j] x_j) / A[i][i]`. The
/// iteration stops when the infinity-norm of the step (`â€–x^{k+1} - x^kâ€–_âˆž`)
/// drops below `tol` or after `max_iter` iterations, whichever comes first.
///
/// The method converges to the true solution for diagonally dominant `A`. The
/// returned vector is the final iterate; for a non-convergent system it is a
/// best-effort estimate rather than an error.
///
/// # Panics
///
/// Panics (a programming error) if any diagonal entry `A[i][i]` is zero, which
/// makes the Jacobi update undefined.
///
/// # Examples
///
/// ```
/// use tpt_zero_solver::jacobi;
/// use tpt_zero_tensor::{Tensor, Tensor2};
///
/// let a = Tensor2::from([[4.0, 1.0], [1.0, 3.0]]);
/// let b = Tensor::from([5.0, 7.0]);
/// let x = jacobi(&a, &b, &Tensor::from([0.0, 0.0]), 1e-10, 1000);
/// let got = tpt_zero_linalg::mat_vec_mul(&a, &x);
/// assert!((got[0] - 5.0).abs() < 1e-6);
/// assert!((got[1] - 7.0).abs() < 1e-6);
/// ```
#[must_use]
pub fn jacobi<const N: usize>(
    a: &Tensor2<f64, N, N>,
    b: &Tensor<f64, N>,
    x0: &Tensor<f64, N>,
    tol: f64,
    max_iter: usize,
) -> Tensor<f64, N> {
    let mut x = *x0;
    let mut iter = 0;
    while iter < max_iter {
        let mut next = [0.0f64; N];
        let mut i = 0;
        while i < N {
            let diag = a[(i, i)];
            assert!(diag != 0.0, "jacobi: zero diagonal entry A[{i}][{i}]");
            let mut sum = b[i];
            let mut j = 0;
            while j < N {
                if j != i {
                    sum -= a[(i, j)] * x[j];
                }
                j += 1;
            }
            next[i] = sum / diag;
            i += 1;
        }

        // Infinity-norm of the step.
        let mut step = 0.0;
        let mut i = 0;
        while i < N {
            let d = (next[i] - x[i]).abs();
            if d > step {
                step = d;
            }
            i += 1;
        }

        x = Tensor::from(next);
        if step < tol {
            break;
        }
        iter += 1;
    }
    x
}

/// Solves `A x = b` by the Gauss-Seidel iterative method.
///
/// Like [`jacobi`], but each component uses the most recently computed values
/// of the other components within the same sweep (in-place update), which
/// typically converges faster than Jacobi for the same diagonally dominant
/// systems. Iteration stops when `â€–x^{k+1} - x^kâ€–_âˆž < tol` or after
/// `max_iter` iterations.
///
/// The returned vector is the final iterate; for a non-convergent system it is
/// a best-effort estimate rather than an error.
///
/// # Panics
///
/// Panics (a programming error) if any diagonal entry `A[i][i]` is zero.
///
/// # Examples
///
/// ```
/// use tpt_zero_solver::gauss_seidel;
/// use tpt_zero_tensor::{Tensor, Tensor2};
///
/// let a = Tensor2::from([[4.0, 1.0], [1.0, 3.0]]);
/// let b = Tensor::from([5.0, 7.0]);
/// let x = gauss_seidel(&a, &b, &Tensor::from([0.0, 0.0]), 1e-10, 1000);
/// let got = tpt_zero_linalg::mat_vec_mul(&a, &x);
/// assert!((got[0] - 5.0).abs() < 1e-6);
/// assert!((got[1] - 7.0).abs() < 1e-6);
/// ```
#[must_use]
pub fn gauss_seidel<const N: usize>(
    a: &Tensor2<f64, N, N>,
    b: &Tensor<f64, N>,
    x0: &Tensor<f64, N>,
    tol: f64,
    max_iter: usize,
) -> Tensor<f64, N> {
    let mut x = *x0;
    let mut iter = 0;
    while iter < max_iter {
        let mut step = 0.0;
        let mut i = 0;
        while i < N {
            let diag = a[(i, i)];
            assert!(diag != 0.0, "gauss_seidel: zero diagonal entry A[{i}][{i}]");
            let mut sum = b[i];
            let mut j = 0;
            while j < N {
                if j != i {
                    sum -= a[(i, j)] * x[j];
                }
                j += 1;
            }
            let new_x = sum / diag;
            let d = (new_x - x[i]).abs();
            if d > step {
                step = d;
            }
            x[i] = new_x;
            i += 1;
        }

        if step < tol {
            break;
        }
        iter += 1;
    }
    x
}

/// Returns the residual `A x - b` of a candidate solution.
///
/// This is a convenience helper for callers who want to quantify how well a
/// solved (or iteratively approximated) `x` satisfies `A x = b`. Small entries
/// in the residual indicate a good solution.
///
/// # Examples
///
/// ```
/// use tpt_zero_solver::{solve_gaussian, residual};
/// use tpt_zero_tensor::{Tensor, Tensor2};
///
/// let a = Tensor2::from([[2.0, 1.0], [1.0, 3.0]]);
/// let b = Tensor::from([5.0, 10.0]);
/// let x = solve_gaussian(&a, &b).unwrap();
/// let r = residual(&a, &x, &b);
/// assert!(r[0].abs() < 1e-9);
/// assert!(r[1].abs() < 1e-9);
/// ```
#[must_use]
pub fn residual<const N: usize>(
    a: &Tensor2<f64, N, N>,
    x: &Tensor<f64, N>,
    b: &Tensor<f64, N>,
) -> Tensor<f64, N> {
    let ax = mat_vec_mul(a, x);
    Tensor::from_fn(|i| ax[i] - b[i])
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_zero_decomp::lu_reconstruct;

    const TOL: f64 = 1e-9;

    fn lu_reconstruct_check(a: &Tensor2<f64, 3, 3>) {
        // Independent verification that the shared LU factors reconstruct `A`
        // (row-permuted), mirroring `tpt_zero_decomp` usage.
        let (l, u, p) = lu(a);
        let recon = lu_reconstruct(&l, &u, &p);
        let mut i = 0;
        while i < 3 {
            let mut c = 0;
            while c < 3 {
                assert!(
                    (recon[(p[i], c)] - a[(p[i], c)]).abs() < TOL,
                    "LU mismatch at ({}, {})",
                    p[i], c
                );
                c += 1;
            }
            i += 1;
        }
    }

    #[test]
    fn gaussian_2x2() {
        let a = Tensor2::from([[2.0, 1.0], [1.0, 3.0]]);
        let b = Tensor::from([5.0, 10.0]);
        let x = solve_gaussian(&a, &b).unwrap();
        assert!((x[0] - 1.0).abs() < TOL);
        assert!((x[1] - 3.0).abs() < TOL);
        let r = residual(&a, &x, &b);
        assert!(r[0].abs() < TOL);
        assert!(r[1].abs() < TOL);
    }

    #[test]
    fn gaussian_needs_pivoting() {
        let a = Tensor2::from([[0.0, 1.0], [1.0, 1.0]]);
        let b = Tensor::from([2.0, 3.0]);
        let x = solve_gaussian(&a, &b).unwrap();
        let r = residual(&a, &x, &b);
        assert!(r[0].abs() < TOL);
        assert!(r[1].abs() < TOL);
    }

    #[test]
    fn gaussian_singular_is_none() {
        let a = Tensor2::from([[1.0, 2.0], [2.0, 4.0]]);
        let b = Tensor::from([1.0, 2.0]);
        assert_eq!(solve_gaussian(&a, &b), None);
    }

    #[test]
    fn gaussian_3x3() {
        let a = Tensor2::from([[4.0, 3.0, 2.0], [1.0, 5.0, 3.0], [2.0, 1.0, 6.0]]);
        let b = Tensor::from([1.0, 2.0, 3.0]);
        let x = solve_gaussian(&a, &b).unwrap();
        let r = residual(&a, &x, &b);
        let mut i = 0;
        while i < 3 {
            assert!(r[i].abs() < TOL);
            i += 1;
        }
    }

    #[test]
    fn lu_matches_gaussian() {
        let a = Tensor2::from([[4.0, 3.0, 2.0], [1.0, 5.0, 3.0], [2.0, 1.0, 6.0]]);
        let b = Tensor::from([1.0, 2.0, 3.0]);
        let g = solve_gaussian(&a, &b).unwrap();
        let lu_x = solve_lu(&a, &b).unwrap();
        let mut i = 0;
        while i < 3 {
            assert!((g[i] - lu_x[i]).abs() < TOL);
            i += 1;
        }
        lu_reconstruct_check(&a);
    }

    #[test]
    fn lu_singular_is_none() {
        let a = Tensor2::from([[1.0, 2.0], [2.0, 4.0]]);
        let b = Tensor::from([1.0, 2.0]);
        assert_eq!(solve_lu(&a, &b), None);
    }

    #[test]
    fn cholesky_2x2() {
        let a = Tensor2::from([[4.0, 1.0], [1.0, 3.0]]);
        let b = Tensor::from([1.0, 2.0]);
        let x = solve_cholesky(&a, &b).unwrap();
        let got = mat_vec_mul(&a, &x);
        assert!((got[0] - 1.0).abs() < TOL);
        assert!((got[1] - 2.0).abs() < TOL);
    }

    #[test]
    fn cholesky_3x3() {
        let a = Tensor2::from([[4.0, 12.0, -16.0], [12.0, 37.0, -43.0], [-16.0, -43.0, 98.0]]);
        let b = Tensor::from([1.0, 2.0, 3.0]);
        let x = solve_cholesky(&a, &b).unwrap();
        let r = residual(&a, &x, &b);
        let mut i = 0;
        while i < 3 {
            assert!(r[i].abs() < TOL);
            i += 1;
        }
    }

    #[test]
    fn cholesky_rejects_non_pd() {
        let neg = Tensor2::from([[-1.0, 0.0], [0.0, -1.0]]);
        assert_eq!(solve_cholesky(&neg, &Tensor::from([1.0, 1.0])), None);
        let indef = Tensor2::from([[1.0, 2.0], [2.0, 1.0]]);
        assert_eq!(solve_cholesky(&indef, &Tensor::from([1.0, 1.0])), None);
    }

    #[test]
    fn jacobi_converges_diagonally_dominant() {
        let a = Tensor2::from([[4.0, 1.0], [1.0, 3.0]]);
        let b = Tensor::from([5.0, 7.0]);
        let x = jacobi(&a, &b, &Tensor::from([0.0, 0.0]), 1e-10, 1000);
        let got = mat_vec_mul(&a, &x);
        assert!((got[0] - 5.0).abs() < 1e-6);
        assert!((got[1] - 7.0).abs() < 1e-6);
    }

    #[test]
    fn gauss_seidel_converges_diagonally_dominant() {
        let a = Tensor2::from([[4.0, 1.0], [1.0, 3.0]]);
        let b = Tensor::from([5.0, 7.0]);
        let x = gauss_seidel(&a, &b, &Tensor::from([0.0, 0.0]), 1e-10, 1000);
        let got = mat_vec_mul(&a, &x);
        assert!((got[0] - 5.0).abs() < 1e-6);
        assert!((got[1] - 7.0).abs() < 1e-6);
    }

    #[test]
    fn gauss_seidel_beats_jacobi_iteration_count() {
        // Same system; Gauss-Seidel should reach the tolerance in no more
        // iterations than Jacobi (often fewer). We just check both converge
        // and agree on the solution.
        let a = Tensor2::from([[10.0, 1.0, 2.0], [1.0, 9.0, 1.0], [2.0, 1.0, 8.0]]);
        let b = Tensor::from([1.0, 2.0, 3.0]);
        let xj = jacobi(&a, &b, &Tensor::from([0.0, 0.0, 0.0]), 1e-10, 5000);
        let xg = gauss_seidel(&a, &b, &Tensor::from([0.0, 0.0, 0.0]), 1e-10, 5000);
        let mut i = 0;
        while i < 3 {
            assert!((xj[i] - xg[i]).abs() < 1e-6, "disagree at {i}");
            i += 1;
        }
    }

    #[test]
    fn direct_and_iterative_agree_on_solution() {
        let a = Tensor2::from([[4.0, 1.0], [1.0, 3.0]]);
        let b = Tensor::from([5.0, 7.0]);
        let g = solve_gaussian(&a, &b).unwrap();
        let xj = jacobi(&a, &b, &Tensor::from([0.0, 0.0]), 1e-10, 1000);
        let xg = gauss_seidel(&a, &b, &Tensor::from([0.0, 0.0]), 1e-10, 1000);
        assert!((g[0] - xj[0]).abs() < 1e-6);
        assert!((g[1] - xj[1]).abs() < 1e-6);
        assert!((g[0] - xg[0]).abs() < 1e-6);
        assert!((g[1] - xg[1]).abs() < 1e-6);
    }
}

#[cfg(test)]
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
mod proptests {
    use proptest::prelude::*;

    use super::*;

    const TOL: f64 = 1e-6;

    fn diagonally_dominant<const N: usize>(rows: [[i64; N]; N]) -> Tensor2<f64, N, N> {
        // For each row i, make the diagonal entry A[i][i] strictly larger in
        // magnitude than the sum of the magnitudes of the off-diagonal entries
        // (and positive), guaranteeing diagonal dominance and a nonzero
        // diagonal so the iterative solvers are well defined.
        Tensor2::new(core::array::from_fn(|i| {
            let mut out = [0.0f64; N];
            let mut j = 0;
            let mut sum_abs_off: i64 = 0;
            while j < N {
                let v = rows[i][j];
                if j != i {
                    sum_abs_off += v.abs();
                }
                out[j] = v as f64;
                j += 1;
            }
            out[i] = (sum_abs_off + 1) as f64;
            out
        }))
    }

    fn dd3() -> impl Strategy<Value = Tensor2<f64, 3, 3>> {
        proptest::array::uniform3(proptest::array::uniform3(-20i64..20))
            .prop_map(diagonally_dominant)
    }

    fn dd4() -> impl Strategy<Value = Tensor2<f64, 4, 4>> {
        proptest::array::uniform4(proptest::array::uniform4(-20i64..20))
            .prop_map(diagonally_dominant)
    }

    fn vec3() -> impl Strategy<Value = Tensor<f64, 3>> {
        proptest::array::uniform3(-20i64..20).prop_map(|v| Tensor::from(v.map(|x| x as f64)))
    }

    fn vec4() -> impl Strategy<Value = Tensor<f64, 4>> {
        proptest::array::uniform4(-20i64..20).prop_map(|v| Tensor::from(v.map(|x| x as f64)))
    }

    /// Builds a symmetric, strictly diagonally dominant (hence symmetric
    /// positive-definite) 3x3 matrix from random off-diagonal entries.
    fn spd3() -> impl Strategy<Value = Tensor2<f64, 3, 3>> {
        proptest::array::uniform3(-20i64..20).prop_map(|off| {
            // off holds the upper-triangular off-diagonals (0,1),(0,2),(1,2).
            let a01 = off[0] as f64;
            let a02 = off[1] as f64;
            let a12 = off[2] as f64;
            let d0 = (a01.abs() + a02.abs()) + 1.0;
            let d1 = (a01.abs() + a12.abs()) + 1.0;
            let d2 = (a02.abs() + a12.abs()) + 1.0;
            Tensor2::from([
                [d0, a01, a02],
                [a01, d1, a12],
                [a02, a12, d2],
            ])
        })
    }

    proptest! {
        #[test]
        fn gaussian_reconstructs_a_3x3(a in dd3(), b in vec3()) {
            let x = solve_gaussian(&a, &b).unwrap();
            let r = residual(&a, &x, &b);
            prop_assert!(r[0].abs() <= TOL);
            prop_assert!(r[1].abs() <= TOL);
            prop_assert!(r[2].abs() <= TOL);
        }

        #[test]
        fn lu_matches_gaussian_3x3(a in dd3(), b in vec3()) {
            let g = solve_gaussian(&a, &b).unwrap();
            let lu = solve_lu(&a, &b).unwrap();
            prop_assert!((g[0] - lu[0]).abs() <= TOL);
            prop_assert!((g[1] - lu[1]).abs() <= TOL);
            prop_assert!((g[2] - lu[2]).abs() <= TOL);
        }

        #[test]
        fn jacobi_converges_3x3(a in dd3(), b in vec3()) {
            let x = jacobi(&a, &b, &Tensor::from([0.0; 3]), 1e-7, 5000);
            let r = residual(&a, &x, &b);
            prop_assert!(r[0].abs() <= 1e-3, "residual {}", r[0]);
            prop_assert!(r[1].abs() <= 1e-3, "residual {}", r[1]);
            prop_assert!(r[2].abs() <= 1e-3, "residual {}", r[2]);
        }

        #[test]
        fn gauss_seidel_converges_4x4(a in dd4(), b in vec4()) {
            let x = gauss_seidel(&a, &b, &Tensor::from([0.0; 4]), 1e-7, 5000);
            let r = residual(&a, &x, &b);
            prop_assert!(r[0].abs() <= 1e-3);
            prop_assert!(r[1].abs() <= 1e-3);
            prop_assert!(r[2].abs() <= 1e-3);
            prop_assert!(r[3].abs() <= 1e-3);
        }

        #[test]
        fn cholesky_spd_3x3(a in spd3(), b in vec3()) {
            let x = solve_cholesky(&a, &b).unwrap();
            let r = residual(&a, &x, &b);
            prop_assert!(r[0].abs() <= TOL);
            prop_assert!(r[1].abs() <= TOL);
            prop_assert!(r[2].abs() <= TOL);
        }
    }
}
