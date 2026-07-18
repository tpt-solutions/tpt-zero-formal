//! `no_std` matrix decompositions for small, fixed-size matrices. Part of the
//! [tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal)
//! ecosystem.
//!
//! This crate factors an `N`-by-`N` matrix `A` (given as a
//! [`Tensor2<f64, N, N>`](tpt_zero_tensor::Tensor2)) into canonical forms:
//!
//! - [`lu`] — Doolittle LU with partial pivoting: `P A = L U`, where `L` is
//!   unit-lower-triangular, `U` is upper-triangular, and `P` is a row
//!   permutation.
//! - [`qr`] — Householder QR: `A = Q R`, where `Q` is orthonormal and `R` is
//!   upper-triangular.
//! - [`cholesky`] — Cholesky: `A = L Lᵀ` for symmetric positive-definite `A`,
//!   returning `None` otherwise.
//!
//! It reuses the [`tpt_zero_linalg`] square root so it never relies on float
//! intrinsics that are unavailable in some `core`-only targets.
//!
//! ```
//! use tpt_zero_decomp::{cholesky, lu, qr};
//! use tpt_zero_tensor::Tensor2;
//!
//! let m = Tensor2::from([[4.0, 12.0, -16.0], [12.0, 37.0, -43.0], [-16.0, -43.0, 98.0]]);
//!
//! // P A = L U.
//! let (l, u, _p) = lu(&m);
//!
//! // A = Q R with Q orthonormal.
//! let (q, r) = qr(&m);
//! assert!((q[(0,0)].abs() - 1.0).abs() <= 1.0);
//!
//! // A = L L^T for symmetric positive-definite A.
//! let chol = cholesky(&m).unwrap();
//! ```
//!
//! # Verifying a decomposition
//!
//! [`lu`] returns `L`, `U`, and a permutation `p` such that applying `p` to the
//! rows of `A` gives `L U`. [`lu_reconstruct`] rebuilds that permuted input.

#![no_std]
#![warn(missing_docs)]
#![forbid(unsafe_code)]

// The float conversion helpers used in the normalisation and Householder
// implementations necessarily convert between `usize` and `f64` on
// data-sized counts that are never large enough to lose meaningful
// information, so the associated cast lints are allowed. Single-character
// index names (`i`, `c`, `k`, `r` ...) are idiomatic in numerical linear
// algebra loops, and the `match` form of `c < i`/`c == i` comparisons reads
// worse than the clear three-way `if`; both lints are allowed here.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::many_single_char_names,
    clippy::comparison_chain,
    clippy::uninlined_format_args
)]

use tpt_zero_linalg::sqrt;
use tpt_zero_tensor::Tensor2;

/// Computes the Doolittle LU decomposition with partial pivoting of a square
/// matrix `A`.
///
/// Returns `(L, U, p)` where:
///
/// - `L` is unit-lower-triangular (`L[(i, i)] == 1`),
/// - `U` is upper-triangular,
/// - `p` is a permutation array such that row `p[i]` of `A` equals row `i` of
///   `L U`. Equivalently, letting `P` be the permutation matrix that sends row
///   `i` to row `p[i]`, we have `P A = L U`, or `A[p[i], :] == (L U)[i, :]`.
///
/// Partial pivoting selects, at each step, the row with the largest available
/// pivot magnitude to improve numerical stability. The decomposition always
/// succeeds for a non-singular matrix; a zero pivot signals a singular matrix.
///
/// # Examples
///
/// ```
/// use tpt_zero_decomp::lu;
/// use tpt_zero_tensor::Tensor2;
///
/// let a = Tensor2::from([[2.0, 1.0], [1.0, 3.0]]);
/// let (l, u, _p) = lu(&a);
/// assert_eq!(l[(1, 0)], 0.5);
/// assert_eq!(u[(0, 0)], 2.0);
/// assert_eq!(u[(0, 1)], 1.0);
/// assert_eq!(u[(1, 1)], 2.5);
/// ```
#[must_use]
pub fn lu<const N: usize>(m: &Tensor2<f64, N, N>) -> (Tensor2<f64, N, N>, Tensor2<f64, N, N>, [usize; N]) {
    let mut a = *m;
    let mut p = [0usize; N];
    let mut i = 0;
    while i < N {
        p[i] = i;
        i += 1;
    }

    let mut row_of = [0usize; N];
    let mut j = 0;
    while j < N {
        row_of[j] = j;
        j += 1;
    }

    let mut k = 0;
    while k < N {
        // Partial pivoting: find the row with the largest magnitude in column k.
        let mut pivot_row = k;
        let mut pivot_val = a[(row_of[k], k)].abs();
        let mut r = k + 1;
        while r < N {
            let v = a[(row_of[r], k)].abs();
            if v > pivot_val {
                pivot_val = v;
                pivot_row = r;
            }
            r += 1;
        }

        if pivot_val == 0.0 {
            // Singular matrix: remaining submatrix is zero.
            k += 1;
            continue;
        }

        if pivot_row != k {
            row_of.swap(k, pivot_row);
        }

        // Eliminate entries below the pivot in column k.
        let mut r = k + 1;
        while r < N {
            let factor = a[(row_of[r], k)] / a[(row_of[k], k)];
            let mut c = k;
            while c < N {
                a[(row_of[r], c)] -= factor * a[(row_of[k], c)];
                c += 1;
            }
            a[(row_of[r], k)] = factor;
            r += 1;
        }

        k += 1;
    }

    // Build L (unit lower-triangular) and U (upper-triangular) from the merged
    // storage, accounting for the permutation via `row_of`.
    let l = Tensor2::from_fn(|i, c| {
        if c < i {
            a[(row_of[i], c)]
        } else if c == i {
            1.0
        } else {
            0.0
        }
    });

    let u = Tensor2::from_fn(|i, c| {
        if c >= i {
            a[(row_of[i], c)]
        } else {
            0.0
        }
    });

    // `p` maps output row -> original row: `A[p[i]] == (L U)[i]`.
    let mut perm = [0usize; N];
    let mut r = 0;
    while r < N {
        perm[r] = row_of[r];
        r += 1;
    }

    (l, u, perm)
}

/// Reconstructs the input matrix `A` from its [`lu`] factors `(L, U, p)`,
/// yielding `A` with rows permuted: row `p[i]` of `A` equals row `i` of
/// `L U`.
///
/// This is the inverse of the `(L, U)` part of [`lu`] and is used to verify the
/// decomposition: `lu_reconstruct(lu(&A))[i] == A[p[i]]` for every `i`.
///
/// # Examples
///
/// ```
/// use tpt_zero_decomp::{lu, lu_reconstruct};
/// use tpt_zero_tensor::Tensor2;
///
/// let a = Tensor2::from([[2.0, 1.0], [1.0, 3.0]]);
/// let (l, u, p) = lu(&a);
/// let recon = lu_reconstruct(&l, &u, &p);
/// assert!((recon[(p[0], 0)] - a[(p[0], 0)]).abs() < 1e-12);
/// assert!((recon[(p[1], 1)] - a[(p[1], 1)]).abs() < 1e-12);
/// ```
#[must_use]
pub fn lu_reconstruct<const N: usize>(
    l: &Tensor2<f64, N, N>,
    u: &Tensor2<f64, N, N>,
    p: &[usize; N],
) -> Tensor2<f64, N, N> {
    let mut out = Tensor2::default();
    let mut i = 0;
    while i < N {
        let mut c = 0;
        while c < N {
            let mut acc = 0.0;
            let mut k = 0;
            while k < N {
                acc += l[(i, k)] * u[(k, c)];
                k += 1;
            }
            out[(p[i], c)] = acc;
            c += 1;
        }
        i += 1;
    }
    out
}

/// Computes the Householder QR decomposition of a square matrix `A`.
///
/// Returns `(Q, R)` such that `A = Q R`, where `Q` is orthonormal
/// (`Qᵀ Q = I`) and `R` is upper-triangular. The implementation applies `N`
/// successive Householder reflections that zero the sub-diagonal entries of
/// each column.
///
/// # Examples
///
/// ```
/// use tpt_zero_decomp::qr;
/// use tpt_zero_tensor::Tensor2;
///
/// let a = Tensor2::from([[12.0, -51.0, 4.0], [6.0, 167.0, -68.0], [-4.0, 24.0, -41.0]]);
/// let (q, r) = qr(&a);
/// assert!((r[(1, 0)]).abs() < 1e-9);
/// assert!((r[(2, 0)]).abs() < 1e-9);
/// assert!((r[(2, 1)]).abs() < 1e-9);
/// ```
#[must_use]
pub fn qr<const N: usize>(m: &Tensor2<f64, N, N>) -> (Tensor2<f64, N, N>, Tensor2<f64, N, N>) {
    // R is reduced in place; Q starts as the identity.
    let mut r = *m;
    let mut q = identity::<N>();

    let mut k = 0;
    while k < N {
        // Build the Householder vector for column k below the diagonal.
        // x = column k of R from row k down; reflect to -||x|| e_k.
        let mut norm_x = 0.0;
        let mut i = k;
        while i < N {
            norm_x += r[(i, k)] * r[(i, k)];
            i += 1;
        }
        norm_x = sqrt(norm_x);
        if norm_x == 0.0 {
            k += 1;
            continue;
        }

        // Sign ensures numerical stability (reflect toward -sign(x_k)).
        let sign = if r[(k, k)] < 0.0 { -1.0 } else { 1.0 };
        let mut v = [0.0f64; N];
        v[k] = r[(k, k)] + sign * norm_x;
        let mut i = k + 1;
        while i < N {
            v[i] = r[(i, k)];
            i += 1;
        }

        let mut norm_v = 0.0;
        let mut i = k;
        while i < N {
            norm_v += v[i] * v[i];
            i += 1;
        }
        norm_v = sqrt(norm_v);
        if norm_v == 0.0 {
            k += 1;
            continue;
        }
        let mut i = k;
        while i < N {
            v[i] /= norm_v;
            i += 1;
        }

        // Apply H = I - 2 v vᵀ to R: R <- (I - 2 v vᵀ) R.
        let mut c = 0;
        while c < N {
            let mut coeff = 0.0;
            let mut i = k;
            while i < N {
                coeff += v[i] * r[(i, c)];
                i += 1;
            }
            coeff *= 2.0;
            let mut i = k;
            while i < N {
                r[(i, c)] -= coeff * v[i];
                i += 1;
            }
            c += 1;
        }

        // Accumulate Q: Q <- Q Hᵀ = Q (I - 2 v vᵀ) = Q - 2 (Q v) vᵀ.
        let mut col = 0;
        while col < N {
            let mut coeff = 0.0;
            let mut i = k;
            while i < N {
                coeff += q[(col, i)] * v[i];
                i += 1;
            }
            coeff *= 2.0;
            let mut i = k;
            while i < N {
                q[(col, i)] -= coeff * v[i];
                i += 1;
            }
            col += 1;
        }

        k += 1;
    }

    (q, r)
}

/// Returns the `N`-by-`N` identity matrix.
#[must_use]
fn identity<const N: usize>() -> Tensor2<f64, N, N> {
    Tensor2::from_fn(|i, c| if i == c { 1.0 } else { 0.0 })
}

/// Computes the Cholesky decomposition `L Lᵀ` of a symmetric
/// positive-definite matrix `A`.
///
/// Returns `Some(L)`, where `L` is lower-triangular and `L Lᵀ == A`, or `None`
/// if `A` is not symmetric positive-definite (for example if a non-positive
/// pivot is encountered). The upper triangle of `A` is ignored; only the lower
/// triangle is read.
///
/// # Examples
///
/// ```
/// use tpt_zero_decomp::cholesky;
/// use tpt_zero_tensor::Tensor2;
///
/// let a = Tensor2::from([[4.0, 12.0, -16.0], [12.0, 37.0, -43.0], [-16.0, -43.0, 98.0]]);
/// let l = cholesky(&a).unwrap();
/// assert_eq!(l[(0, 0)], 2.0);
/// assert_eq!(l[(1, 0)], 6.0);
/// assert_eq!(l[(2, 0)], -8.0);
///
/// // A non-positive-definite matrix yields None.
/// let not_pd = Tensor2::from([[-1.0, 0.0], [0.0, -1.0]]);
/// assert_eq!(cholesky(&not_pd), None);
/// ```
#[must_use]
pub fn cholesky<const N: usize>(m: &Tensor2<f64, N, N>) -> Option<Tensor2<f64, N, N>> {
    let mut l = Tensor2::default();

    let mut i = 0;
    while i < N {
        let mut j = 0;
        while j <= i {
            let mut sum = 0.0;
            let mut k = 0;
            while k < j {
                sum += l[(i, k)] * l[(j, k)];
                k += 1;
            }

            if i == j {
                let d = m[(i, i)] - sum;
                if d <= 0.0 {
                    return None;
                }
                l[(i, j)] = sqrt(d);
            } else {
                l[(i, j)] = (m[(i, j)] - sum) / l[(j, j)];
            }
            j += 1;
        }
        i += 1;
    }

    Some(l)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_zero_tensor::Tensor2;

    const TOL: f64 = 1e-9;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() <= TOL
    }

    fn lu_reconstruct_check<const N: usize>(m: &Tensor2<f64, N, N>) {
        let (l, u, p) = lu(m);
        let recon = lu_reconstruct(&l, &u, &p);
        let mut i = 0;
        while i < N {
            let mut c = 0;
            while c < N {
                assert!(
                    approx_eq(recon[(p[i], c)], m[(p[i], c)]),
                    "LU mismatch at ({}, {}): {} vs {}",
                    p[i],
                    c,
                    recon[(p[i], c)],
                    m[(p[i], c)]
                );
                c += 1;
            }
            i += 1;
        }
    }

    #[test]
    fn lu_2x2() {
        let a = Tensor2::from([[2.0, 1.0], [1.0, 3.0]]);
        let (l, u, _p) = lu(&a);
        assert!(approx_eq(l[(1, 0)], 0.5));
        assert!(approx_eq(u[(0, 0)], 2.0));
        assert!(approx_eq(u[(0, 1)], 1.0));
        assert!(approx_eq(u[(1, 1)], 2.5));
        lu_reconstruct_check(&a);
    }

    #[test]
    fn lu_requires_pivoting() {
        // Row 0 is smaller in magnitude; pivoting must swap rows.
        let a = Tensor2::from([[0.0, 1.0], [1.0, 1.0]]);
        let (l, u, p) = lu(&a);
        lu_reconstruct_check(&a);
        assert!(p[0] != 0);
        let _ = (l, u);
    }

    #[test]
    fn lu_3x3_reconstructs() {
        let a = Tensor2::from([[4.0, 3.0, 2.0], [1.0, 5.0, 3.0], [2.0, 1.0, 6.0]]);
        lu_reconstruct_check(&a);
    }

    #[test]
    fn lu_4x4_reconstructs() {
        let a = Tensor2::from([
            [5.0, 2.0, 1.0, 3.0],
            [1.0, 4.0, 2.0, 0.0],
            [2.0, 1.0, 6.0, 1.0],
            [3.0, 0.0, 1.0, 7.0],
        ]);
        lu_reconstruct_check(&a);
    }

    #[test]
    fn qr_3x3_properties() {
        let a = Tensor2::from([[12.0, -51.0, 4.0], [6.0, 167.0, -68.0], [-4.0, 24.0, -41.0]]);
        let (q, r) = qr(&a);

        // R is upper-triangular.
        assert!(approx_eq(r[(1, 0)], 0.0));
        assert!(approx_eq(r[(2, 0)], 0.0));
        assert!(approx_eq(r[(2, 1)], 0.0));

        // Q is orthonormal: Qᵀ Q = I.
        let qtq = q.transpose().mul(&q);
        let i3 = identity::<3>();
        let mut i = 0;
        while i < 3 {
            let mut c = 0;
            while c < 3 {
                assert!(approx_eq(qtq[(i, c)], i3[(i, c)]), "Q not orthonormal at ({}, {})", i, c);
                c += 1;
            }
            i += 1;
        }

        // A = Q R.
        let prod = q.mul(&r);
        let mut i = 0;
        while i < 3 {
            let mut c = 0;
            while c < 3 {
                assert!(approx_eq(prod[(i, c)], a[(i, c)]), "QR mismatch at ({}, {})", i, c);
                c += 1;
            }
            i += 1;
        }
    }

    #[test]
    fn qr_4x4_properties() {
        let a = Tensor2::from([
            [2.0, 0.0, 1.0, 3.0],
            [1.0, 3.0, 2.0, 0.0],
            [0.0, 1.0, 4.0, 1.0],
            [3.0, 0.0, 1.0, 2.0],
        ]);
        let (q, r) = qr(&a);
        let qtq = q.transpose().mul(&q);
        let i4 = identity::<4>();
        let mut i = 0;
        while i < 4 {
            let mut c = 0;
            while c < 4 {
                assert!(approx_eq(qtq[(i, c)], i4[(i, c)]));
                c += 1;
            }
            i += 1;
        }
        let prod = q.mul(&r);
        let mut i = 0;
        while i < 4 {
            let mut c = 0;
            while c < 4 {
                assert!(approx_eq(prod[(i, c)], a[(i, c)]));
                c += 1;
            }
            i += 1;
        }
        // Upper-triangular check.
        assert!(approx_eq(r[(1, 0)], 0.0));
        assert!(approx_eq(r[(2, 0)], 0.0));
        assert!(approx_eq(r[(3, 0)], 0.0));
        assert!(approx_eq(r[(2, 1)], 0.0));
        assert!(approx_eq(r[(3, 1)], 0.0));
        assert!(approx_eq(r[(3, 2)], 0.0));
    }

    #[test]
    fn cholesky_reconstructs() {
        let a = Tensor2::from([[4.0, 12.0, -16.0], [12.0, 37.0, -43.0], [-16.0, -43.0, 98.0]]);
        let l = cholesky(&a).unwrap();
        let ll = l.mul(&l.transpose());
        let mut i = 0;
        while i < 3 {
            let mut c = 0;
            while c < 3 {
                assert!(approx_eq(ll[(i, c)], a[(i, c)]), "Cholesky mismatch at ({}, {})", i, c);
                c += 1;
            }
            i += 1;
        }
    }

    #[test]
    fn cholesky_rejects_non_pd() {
        let neg = Tensor2::from([[-1.0, 0.0], [0.0, -1.0]]);
        assert_eq!(cholesky(&neg), None);

        // Symmetric but indefinite (eigenvalues of opposite sign).
        let indef = Tensor2::from([[1.0, 2.0], [2.0, 1.0]]);
        assert_eq!(cholesky(&indef), None);
    }

    #[test]
    fn cholesky_identity() {
        let id = identity::<3>();
        let l = cholesky(&id).unwrap();
        assert!(approx_eq(l[(0, 0)], 1.0));
        assert!(approx_eq(l[(1, 1)], 1.0));
        assert!(approx_eq(l[(2, 2)], 1.0));
        assert!(approx_eq(l[(1, 0)], 0.0));
    }
}

#[cfg(test)]
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
mod proptests {
    use proptest::prelude::*;

    use super::*;
    use tpt_zero_tensor::Tensor2;

    const TOL: f64 = 1e-6;

    fn mat3() -> impl Strategy<Value = Tensor2<f64, 3, 3>> {
        proptest::array::uniform3(proptest::array::uniform3(-100i64..100)).prop_map(|rows| {
            Tensor2::new(rows.map(|r| r.map(|x| x as f64)))
        })
    }

    fn mat4() -> impl Strategy<Value = Tensor2<f64, 4, 4>> {
        proptest::array::uniform4(proptest::array::uniform4(-100i64..100)).prop_map(|rows| {
            Tensor2::new(rows.map(|r| r.map(|x| x as f64)))
        })
    }

    proptest! {
        #[test]
        fn lu_reconstruction_3x3(a in mat3()) {
            let (l, u, p) = lu(&a);
            let recon = lu_reconstruct(&l, &u, &p);
            let mut i = 0;
            while i < 3 {
                let mut c = 0;
                while c < 3 {
                    prop_assert!(
                        (recon[(p[i], c)] - a[(p[i], c)]).abs() <= TOL,
                        "LU mismatch at ({}, {})",
                        p[i], c
                    );
                    c += 1;
                }
                i += 1;
            }
        }

        #[test]
        fn lu_reconstruction_4x4(a in mat4()) {
            let (l, u, p) = lu(&a);
            let recon = lu_reconstruct(&l, &u, &p);
            let mut i = 0;
            while i < 4 {
                let mut c = 0;
                while c < 4 {
                    prop_assert!(
                        (recon[(p[i], c)] - a[(p[i], c)]).abs() <= TOL,
                        "LU mismatch at ({}, {})",
                        p[i], c
                    );
                    c += 1;
                }
                i += 1;
            }
        }

        #[test]
        fn qr_orthonormal_q_3x3(a in mat3()) {
            let (q, r) = qr(&a);
            let qtq = q.transpose().mul(&q);
            let i3 = identity::<3>();
            let mut i = 0;
            while i < 3 {
                let mut c = 0;
                while c < 3 {
                    prop_assert!(
                        (qtq[(i, c)] - i3[(i, c)]).abs() <= TOL,
                        "Q not orthonormal at ({}, {})",
                        i, c
                    );
                    c += 1;
                }
                i += 1;
            }
            // R upper-triangular.
            prop_assert!((r[(1, 0)]).abs() <= TOL);
            prop_assert!((r[(2, 0)]).abs() <= TOL);
            prop_assert!((r[(2, 1)]).abs() <= TOL);
        }

        #[test]
        fn qr_reconstructs_a_3x3(a in mat3()) {
            let (q, r) = qr(&a);
            let prod = q.mul(&r);
            let mut i = 0;
            while i < 3 {
                let mut c = 0;
                while c < 3 {
                    prop_assert!(
                        (prod[(i, c)] - a[(i, c)]).abs() <= TOL,
                        "QR mismatch at ({}, {})",
                        i, c
                    );
                    c += 1;
                }
                i += 1;
            }
        }
    }
}
