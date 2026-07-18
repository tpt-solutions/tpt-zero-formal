# tpt-zero-solver

[![crates.io](https://img.shields.io/crates/v/tpt-zero-solver.svg)](https://crates.io/crates/tpt-zero-solver)
[![docs.rs](https://docs.rs/tpt-zero-solver/badge.svg)](https://docs.rs/tpt-zero-solver)
[![license](https://img.shields.io/crates/l/tpt-zero-solver.svg)](#license)

`no_std` linear system solvers: Gaussian elimination, LU/Cholesky direct
solves, and Jacobi/Gauss-Seidel iteration over fixed-size tensors. Part of the
[tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal) ecosystem.

## Quick example

```rust
use tpt_zero_solver::{solve_gaussian, solve_cholesky};
use tpt_zero_tensor::{Tensor, Tensor2};

let a = Tensor2::from([[2.0, 1.0], [1.0, 3.0]]);
let b = Tensor::from([5.0, 10.0]);

let x = solve_gaussian(&a, &b).unwrap();
assert!((x[0] - 1.0).abs() < 1e-9);

let spd = Tensor2::from([[4.0, 1.0], [1.0, 3.0]]);
let y = solve_cholesky(&spd, &Tensor::from([1.0, 2.0])).unwrap();
let got = spd.mat_vec_mul(&y);
assert!((got[0] - 1.0).abs() < 1e-9);
```

The crate exposes two families of solvers: **direct** methods
([`solve_gaussian`], [`solve_lu`], [`solve_cholesky`]) that terminate in a fixed
number of operations, and **iterative** methods ([`jacobi`], [`gauss_seidel`])
that refine an initial guess and converge for diagonally dominant systems. The
[`residual`] helper quantifies how well a candidate `x` satisfies `A x = b`.

## Features

| Feature | Default | Enables |
|---|---|---|
| `alloc` | off | reserved for future alloc-dependent helpers |
| `std` | off | implies `alloc` |

This crate builds with `--no-default-features` (pure `core`, no `alloc`).

## Install

```sh
cargo add tpt-zero-solver
```

## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
