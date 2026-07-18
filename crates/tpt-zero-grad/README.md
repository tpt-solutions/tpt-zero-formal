# tpt-zero-grad

[![crates.io](https://img.shields.io/crates/v/tpt-zero-grad.svg)](https://crates.io/crates/tpt-zero-grad)
[![docs.rs](https://docs.rs/tpt-zero-grad/badge.svg)](https://docs.rs/tpt-zero-grad)
[![license](https://img.shields.io/crates/l/tpt-zero-grad.svg)](#license)

Forward-mode automatic differentiation via dual numbers for `no_std`, with zero
external production dependencies. Part of the
[tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal) ecosystem.

## Quick example

```rust
use tpt_zero_grad::{grad, Dual};

// d/dx x^2 = 2x
let d = grad(|x| x * x, 3.0);
assert!((d - 6.0).abs() < 1e-12);

// d/dx sin(x) = cos(x)
let d = grad(|x| x.sin(), 1.0);
assert!((d - 1.0f64.cos()).abs() < 1e-12);
```

A [`Dual<T>`](crate::Dual) carries a primal value and its derivative with
respect to a single independent variable. Arithmetic operators apply the
dual-number rules, so evaluating any differentiable expression also produces
its derivative. [`grad`](crate::grad) seeds the derivative to `1.0` and returns
the derivative part at a point. Transcendental operations (`exp`, `ln`, `sin`,
`cos`, `powi`) are included, with self-contained `core`-only implementations so
the crate never relies on float intrinsics missing from some bare-metal
targets.

## Features

| Feature | Default | Enables |
|---|---|---|
| `alloc` | off | reserved for future alloc-dependent helpers |
| `std` | off | implies `alloc` |

This crate builds with `--no-default-features` (pure `core`, no `alloc`).

## Install

```sh
cargo add tpt-zero-grad
```

## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
