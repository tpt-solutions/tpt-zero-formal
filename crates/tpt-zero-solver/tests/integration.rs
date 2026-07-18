//! Integration tests for `tpt_zero_solver`, exercising the public API the way
//! a downstream crate would (with `std` available).

use tpt_zero_solver::{
    gauss_seidel, jacobi, residual, solve_cholesky, solve_gaussian, solve_lu,
};
use tpt_zero_tensor::{Tensor, Tensor2};

const TOL: f64 = 1e-8;

fn assert_residual_small<const N: usize>(a: &Tensor2<f64, N, N>, x: &Tensor<f64, N>, b: &Tensor<f64, N>) {
    let r = residual(a, x, b);
    for i in 0..N {
        assert!(r[i].abs() < TOL, "residual[{i}] = {} too large", r[i]);
    }
}

#[test]
fn integration_gaussian_and_lu_agree() {
    let a = Tensor2::from([[4.0, 3.0, 2.0], [1.0, 5.0, 3.0], [2.0, 1.0, 6.0]]);
    let b = Tensor::from([3.0, 2.0, 1.0]);

    let g = solve_gaussian(&a, &b).expect("gaussian should solve a nonsingular system");
    let lu = solve_lu(&a, &b).expect("lu should solve a nonsingular system");

    assert_residual_small(&a, &g, &b);
    assert_residual_small(&a, &lu, &b);

    for i in 0..3 {
        assert!((g[i] - lu[i]).abs() < TOL, "direct solvers disagree at {i}");
    }
}

#[test]
fn integration_cholesky_spd() {
    let a = Tensor2::from([[4.0, 12.0, -16.0], [12.0, 37.0, -43.0], [-16.0, -43.0, 98.0]]);
    let b = Tensor::from([1.0, 2.0, 3.0]);
    let x = solve_cholesky(&a, &b).expect("spd matrix should have a cholesky factor");
    assert_residual_small(&a, &x, &b);
}

#[test]
fn integration_cholesky_rejects_indefinite() {
    let indef = Tensor2::from([[1.0, 2.0], [2.0, 1.0]]);
    assert_eq!(solve_cholesky(&indef, &Tensor::from([1.0, 1.0])), None);
}

#[test]
fn integration_iterative_converges_diagonally_dominant() {
    // Strictly diagonally dominant, so both Jacobi and Gauss-Seidel converge.
    let a = Tensor2::from([[10.0, 1.0, 2.0], [1.0, 9.0, 1.0], [2.0, 1.0, 8.0]]);
    let b = Tensor::from([1.0, 2.0, 3.0]);
    let x0 = Tensor::from([0.0, 0.0, 0.0]);

    let xj = jacobi(&a, &b, &x0, 1e-10, 5000);
    let xg = gauss_seidel(&a, &b, &x0, 1e-10, 5000);

    assert_residual_small(&a, &xj, &b);
    assert_residual_small(&a, &xg, &b);
}

#[test]
fn integration_iterative_matches_direct() {
    let a = Tensor2::from([[4.0, 1.0], [1.0, 3.0]]);
    let b = Tensor::from([5.0, 7.0]);
    let direct = solve_gaussian(&a, &b).unwrap();
    let x0 = Tensor::from([0.0, 0.0]);

    let xj = jacobi(&a, &b, &x0, 1e-10, 1000);
    let xg = gauss_seidel(&a, &b, &x0, 1e-10, 1000);

    for i in 0..2 {
        assert!((direct[i] - xj[i]).abs() < 1e-6, "jacobi disagrees at {i}");
        assert!((direct[i] - xg[i]).abs() < 1e-6, "gauss-seidel disagrees at {i}");
    }
}
