# tpt-zero-tensor

[![crates.io](https://img.shields.io/crates/v/tpt-zero-tensor.svg)](https://crates.io/crates/tpt-zero-tensor)
[![docs.rs](https://docs.rs/tpt-zero-tensor/badge.svg)](https://docs.rs/tpt-zero-tensor)
[![license](https://img.shields.io/crates/l/tpt-zero-tensor.svg)](#license)

Fixed-rank, const-generic tensor types backed by fixed-size arrays for `no_std`
with zero allocation. Part of the
[tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal) ecosystem.

## Quick example

```rust
use tpt_zero_tensor::{Tensor, Tensor2};

let a = Tensor::from_fn(|i| i as f64);
let b = Tensor::from_fn(|i| (i * 2) as f64);
let sum = a.add(&b);
assert_eq!(sum.get(1), Some(&2.0));

let m = Tensor2::from_fn(|r, c| (r * 10 + c) as i32);
let t = m.transpose();
assert_eq!(t.get(1, 0), Some(&1));
```

## Features

| Feature | Default | Enables |
|---|---|---|
| `alloc` | off | reserved for future alloc-dependent helpers |
| `std` | off | implies `alloc` |

This crate builds with `--no-default-features` (pure `core`, no `alloc`).

## Install

```sh
cargo add tpt-zero-tensor
```

## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
