//! Solve a small linear system with the no_std tensor + solver stack.
use tpt_zero_formal::prelude::*;
use tpt_zero_formal::solver;

fn main() {
    // Solve A x = b for a symmetric positive-definite A via Cholesky.
    let a = Tensor2::from_fn(|r, c| [[4.0, 1.0], [1.0, 3.0]][r][c]);
    let b = Tensor::<f64, 2>::from_fn(|i| [1.0, 2.0][i]);
    let x = solver::solve_cholesky(&a, &b).unwrap();
    println!(
        "x = [{}, {}]",
        x.get(0).copied().unwrap(),
        x.get(1).copied().unwrap()
    );
}
