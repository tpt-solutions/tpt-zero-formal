# tpt-zero-decomp

[![crates.io](https://img.shields.io/crates/v/tpt-zero-decomp.svg)](https://crates.io/crates/tpt-zero-decomp)
[![docs.rs](https://docs.rs/tpt-zero-decomp/badge.svg)](https://docs.rs/tpt-zero-decomp)
[![license](https://img.shields.io/crates/l/tpt-zero-decomp.svg)](#license)

`no_std` matrix decompositions for small, fixed-size matrices: LU with partial
pivoting, QR via Householder reflections, and Cholesky. Part of the
[tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal) ecosystem.

The routines operate on the compile-time-sized matrices from
[`tpt_zero_tensor`]: [`Tensor2<f64, N, N>`](tpt_zero_tensor::Tensor2). Because
`f64::sqrt` is unavailable on some `core`-only targets, the crate provides its
own Newton-iteration square root.

## Quick example

```rust
use tpt_zero_decomp::{cholesky, lu, qr};
use tpt_zero_tensor::Tensor2;

let m = Tensor2::from([[4.0, 12.0, -16.0], [12.0, 37.0, -43.0], [-16.0, -43.0, 98.0]]);

// LU with partial pivoting: P A = L U.
let (l, u, _p) = lu(&m);

// QR factorisation: A = Q R with Q orthonormal.
let (q, r) = qr(&m);

// Cholesky: A = L L^T for symmetric positive-definite A.
let chol = cholesky(&m).unwrap();
```

## Features

| Feature | Default | Enables |
|---|---|---|
| `alloc` | off | reserved for future alloc-dependent helpers |
| `std` | off | implies `alloc` |

This crate builds with `--no-default-features` (pure `core`, no `alloc`).

## Install

```sh
cargo add tpt-zero-decomp
```

## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
