# tpt-zero-eigen

[![crates.io](https://img.shields.io/crates/v/tpt-zero-eigen.svg)](https://crates.io/crates/tpt-zero-eigen)
[![docs.rs](https://docs.rs/tpt-zero-eigen/badge.svg)](https://docs.rs/tpt-zero-eigen)
[![license](https://img.shields.io/crates/l/tpt-zero-eigen.svg)](#license)

Eigenvalue and eigenvector computation for small, fixed-size matrices in
`no_std`, with zero external dependencies. Part of the
[tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal) ecosystem.

## Quick example

```rust
use tpt_zero_eigen::{eigenvalues_2x2, power_iteration};
use tpt_zero_tensor::Tensor2;

let a = Tensor2::from([[2.0, 1.0], [1.0, 2.0]]);

// Closed-form eigenvalues of a 2x2 matrix.
let ev = eigenvalues_2x2(&a);
assert!((ev[0] + ev[1] - 4.0).abs() < 1e-12); // trace identity
assert!((ev[0] * ev[1] - 3.0).abs() < 1e-12); // determinant identity

// Power iteration recovers the dominant eigenpair.
let (lambda, v) = power_iteration(&a, 1e-12, 1000);
let av = tpt_zero_linalg::mat_vec_mul(&a, &v);
let lv = v.map(|x| x * lambda);
for i in 0..2 {
    assert!((av[i] - lv[i]).abs() < 1e-8);
}
```

## Features

| Feature | Default | Enables |
|---|---|---|
| `alloc` | off | reserved for future alloc-dependent helpers |
| `std` | off | implies `alloc` |

This crate builds with `--no-default-features` (pure `core`, no `alloc`).

## Install

```sh
cargo add tpt-zero-eigen
```

## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
