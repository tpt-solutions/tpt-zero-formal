//! Integration tests for `tpt-zero-eigen`. These run with the default features
//! (and `std` is available here, since integration tests link the standard
//! library regardless of the crate's own `no_std` core).

use tpt_zero_eigen::{eigenvalues_2x2, inverse_iteration, power_iteration, rayleigh_quotient};
use tpt_zero_linalg::mat_vec_mul;
use tpt_zero_tensor::{Tensor, Tensor2};

const TOL: f64 = 1e-7;

fn is_eigenpair<const N: usize>(a: &Tensor2<f64, N, N>, lambda: f64, v: &Tensor<f64, N>) -> bool {
    let av = mat_vec_mul(a, v);
    let lv = v.map(|x| x * lambda);
    let mut ok = true;
    let mut i = 0;
    while i < N {
        if (av[i] - lv[i]).abs() > TOL {
            ok = false;
        }
        i += 1;
    }
    ok
}

#[test]
fn power_iteration_eigenpair_2x2() {
    let a = Tensor2::from([[4.0, 1.0], [1.0, 3.0]]);
    let (lambda, v) = power_iteration(&a, 1e-12, 5000);
    assert!(is_eigenpair(&a, lambda, &v), "A v ≈ λ v failed");
}

#[test]
fn inverse_iteration_eigenpair_2x2() {
    let a = Tensor2::from([[4.0, 1.0], [1.0, 3.0]]);
    let (lambda, v) = inverse_iteration(&a, 1e-12, 5000).unwrap();
    assert!(is_eigenpair(&a, lambda, &v), "A v ≈ λ v failed");
}

#[test]
fn power_iteration_eigenpair_3x3() {
    // Symmetric 3x3; power iteration still converges to the dominant pair.
    let a = Tensor2::from([[2.0, 1.0, 0.0], [1.0, 2.0, 1.0], [0.0, 1.0, 2.0]]);
    let (lambda, v) = power_iteration(&a, 1e-12, 10000);
    assert!(is_eigenpair(&a, lambda, &v), "A v ≈ λ v failed");
}

#[test]
fn eigenvalues_2x2_trace_det_identity() {
    let a = Tensor2::from([[3.0, 1.0], [1.0, 1.0]]);
    let ev = eigenvalues_2x2(&a);
    let tr = a[(0, 0)] + a[(1, 1)];
    let det = a[(0, 0)] * a[(1, 1)] - a[(0, 1)] * a[(1, 0)];
    assert!((ev[0] + ev[1] - tr).abs() < 1e-12);
    assert!((ev[0] * ev[1] - det).abs() < 1e-12);
    // Each root must satisfy the characteristic equation det(A - λI) = 0.
    for &lambda in &ev {
        let char_eq = lambda * lambda - tr * lambda + det;
        assert!(char_eq.abs() < 1e-12, "λ={lambda} fails char eq ({char_eq})");
    }
}

#[test]
fn rayleigh_equals_eigenvalue_of_eigenvector() {
    // For an exact eigenvector, the Rayleigh quotient returns its eigenvalue.
    let a = Tensor2::from([[2.0, 0.0], [0.0, 5.0]]);
    let v = Tensor::from([0.0, 1.0]); // eigenvector for eigenvalue 5.
    assert!((rayleigh_quotient(&a, &v) - 5.0).abs() < 1e-12);
}