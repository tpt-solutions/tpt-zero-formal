# out-zero-contract

[![crates.io](https://img.shields.io/crates/v/out-zero-contract.svg)](https://crates.io/crates/out-zero-contract)
[![docs.rs](https://docs.rs/out-zero-contract/badge.svg)](https://docs.rs/out-zero-contract)
[![license](https://img.shields.io/crates/l/out-zero-contract.svg)](#license)

Design-by-contract precondition and postcondition macros for `no_std`, with
a `checked` mode for always-on assertions. Part of the
[tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal) ecosystem.

## Quick example

```rust
use out_zero_contract::{requires, ensures};

fn div(a: i32, b: i32) -> i32 {
    requires!(b != 0, "divisor must be non-zero");
    let q = a / b;
    ensures!(b * q == a - (a % b), "division identity holds");
    q
}

assert_eq!(div(10, 2), 5);
```

By default `requires!` and `ensures!` expand to `debug_assert!`, so they are
compiled away in release builds (zero-cost). Enable the `checked` feature to
make them use `assert!` instead, enforcing contracts even in release.

## Features

| Feature | Default | Enables |
|---|---|---|
| `checked` | off | makes `requires!`/`ensures!` always assert (via `assert!`) rather than only in debug |
| `alloc` | off | reserved for future alloc-dependent helpers |
| `std` | off | implies `alloc` |

This crate builds with `--no-default-features` (pure `core`, no `alloc`).

## Install

```sh
cargo add out-zero-contract
```

## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
