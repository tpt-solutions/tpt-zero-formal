//! Integration tests for `tpt-zero-grad`. `std` is available here.

use tpt_zero_grad::{grad, Dual};
use tpt_zero_tensor::Tensor;

#[test]
fn grad_polynomial_matches_analytic() {
    // f(x) = x^4 - 3 x^2 + 2 x  =>  f'(x) = 4 x^3 - 6 x + 2
    let f = |x: Dual<f64>| {
        let x2 = x * x;
        let x4 = x2 * x2;
        x4 - Dual::constant(3.0) * x2 + Dual::constant(2.0) * x
    };
    for &x in &[-2.0, -0.5, 0.0, 1.0, 3.7] {
        let d = grad(f, x);
        let expected = 4.0 * x * x * x - 6.0 * x + 2.0;
        assert!((d - expected).abs() < 1e-9, "x={x}, got {d}, expected {expected}");
    }
}

#[test]
fn grad_tensor_matches_partials() {
    // f(v) = v[0].exp() + v[0]*v[1] + sin(v[1])
    // partial0 = e^{v0} + v1 ;  partial1 = v0 + cos(v1)
    let v = Tensor::from([1.0, 2.0]);
    let g = tpt_zero_grad::grad_tensor(
        |x: Tensor<Dual<f64>, 2>| x[0].exp() + x[0] * x[1] + x[1].sin(),
        &v,
    );
    assert!((g[0] - (v[0].exp() + v[1])).abs() < 1e-9);
    assert!((g[1] - (v[0] + v[1].cos())).abs() < 1e-9);
}

#[test]
fn chain_rule_nested() {
    // f(x) = ln(1 + x^2)  =>  f'(x) = 2x / (1 + x^2)
    let d = grad(|x| (Dual::constant(1.0) + x * x).ln(), 2.0);
    let expected = 4.0 / 5.0;
    assert!((d - expected).abs() < 1e-9, "got {d}, expected {expected}");
}
