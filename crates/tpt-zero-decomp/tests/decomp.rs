//! Integration tests for `tpt-zero-decomp` (std is available here).

#![allow(clippy::uninlined_format_args, clippy::float_cmp)]

use tpt_zero_decomp::{cholesky, lu, lu_reconstruct, qr};
use tpt_zero_tensor::Tensor2;

const TOL: f64 = 1e-9;

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() <= TOL
}

#[test]
fn lu_reconstructs_input() {
    let a = Tensor2::from([[4.0, 3.0, 2.0], [1.0, 5.0, 3.0], [2.0, 1.0, 6.0]]);
    let (l, u, p) = lu(&a);
    assert_eq!(l[(0, 0)], 1.0);
    assert_eq!(l[(1, 1)], 1.0);
    assert_eq!(l[(2, 2)], 1.0);
    let recon = lu_reconstruct(&l, &u, &p);
    for i in 0..3 {
        for c in 0..3 {
            assert!(approx_eq(recon[(p[i], c)], a[(p[i], c)]), "LU at ({}, {})", p[i], c);
        }
    }
}

#[test]
fn qr_q_is_orthonormal_and_reconstructs() {
    let a = Tensor2::from([[12.0, -51.0, 4.0], [6.0, 167.0, -68.0], [-4.0, 24.0, -41.0]]);
    let (q, r) = qr(&a);
    let qtq = q.transpose().mul(&q);
    let id = Tensor2::from([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]);
    for i in 0..3 {
        for c in 0..3 {
            assert!(approx_eq(qtq[(i, c)], id[(i, c)]), "orthonormal at ({}, {})", i, c);
        }
    }
    for i in 1..3 {
        for c in 0..i {
            assert!(approx_eq(r[(i, c)], 0.0), "R upper at ({}, {})", i, c);
        }
    }
    let prod = q.mul(&r);
    for i in 0..3 {
        for c in 0..3 {
            assert!(approx_eq(prod[(i, c)], a[(i, c)]), "reconstruct at ({}, {})", i, c);
        }
    }
}

#[test]
fn cholesky_reconstructs_spd() {
    let a = Tensor2::from([[4.0, 12.0, -16.0], [12.0, 37.0, -43.0], [-16.0, -43.0, 98.0]]);
    let l = cholesky(&a).expect("symmetric positive-definite");
    let ll = l.mul(&l.transpose());
    for i in 0..3 {
        for c in 0..3 {
            assert!(approx_eq(ll[(i, c)], a[(i, c)]), "cholesky at ({}, {})", i, c);
        }
    }
}

#[test]
fn cholesky_none_for_indefinite() {
    let indefinite = Tensor2::from([[1.0, 2.0], [2.0, 1.0]]);
    assert_eq!(cholesky(&indefinite), None);
}
