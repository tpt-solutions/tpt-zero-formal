# tpt-zero-linalg

[![crates.io](https://img.shields.io/crates/v/tpt-zero-linalg.svg)](https://crates.io/crates/tpt-zero-linalg)
[![docs.rs](https://docs.rs/tpt-zero-linalg/badge.svg)](https://docs.rs/tpt-zero-linalg)
[![license](https://img.shields.io/crates/l/tpt-zero-linalg.svg)](#license)

Linear algebra for `no_std`: dot product, cross product, vector norms
([L1](https://en.wikipedia.org/wiki/Taxicab_geometry), [L2](https://en.wikipedia.org/wiki/Euclidean_distance),
max), vector normalization, and small matrix helpers (trace, Frobenius norm,
matrix-vector product) — all over the fixed-size
[`tpt-zero-tensor`](https://crates.io/crates/tpt-zero-tensor) types. Part of
the [tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal)
ecosystem.

## Quick example

```rust
use tpt_zero_linalg::{cross, dot, normalize, norm_l2};
use tpt_zero_tensor::Tensor;

let a = Tensor::from([1.0, 0.0, 0.0]);
let b = Tensor::from([0.0, 1.0, 0.0]);

// Cross product of orthogonal unit vectors is the third basis vector.
let c = cross(&a, &b);
assert_eq!(c.as_ref(), &[0.0, 0.0, 1.0]);

// Dot product of orthogonal vectors is zero.
assert_eq!(dot(&a, &b), 0.0);

// Normalizing a unit vector keeps it a unit vector.
let n = normalize(&a).unwrap();
assert!((norm_l2(&n) - 1.0).abs() < 1e-12);
```

All operations are const-generic and allocation-free: sizes are known at
compile time and storage is plain `[f64; N]` / `[[f64; C]; R]` arrays. A
Newton-iteration `sqrt` is provided so the crate needs no `std` float
intrinsics.

## Features

| Feature | Default | Enables |
|---|---|---|
| `alloc` | off | reserved for future alloc-dependent helpers |
| `std` | off | implies `alloc` |

This crate builds with `--no-default-features` (pure `core`, no `alloc`).

## Install

```sh
cargo add tpt-zero-linalg
```

## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
