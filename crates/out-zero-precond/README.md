# out-zero-precond

[![crates.io](https://img.shields.io/crates/v/out-zero-precond.svg)](https://crates.io/crates/out-zero-precond)
[![docs.rs](https://docs.rs/out-zero-precond/badge.svg)](https://docs.rs/out-zero-precond)
[![license](https://img.shields.io/crates/l/out-zero-precond.svg)](#license)

Precondition checking helpers built on design-by-contract for `no_std`.
Part of the
[tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal) ecosystem.

## Quick example

```rust
use out_zero_precond::{check, PreconditionError};

fn sqrt(x: f64) -> Result<f64, PreconditionError> {
    check(x >= 0.0, "x must be non-negative")?;
    Ok(x.sqrt())
}

assert!(sqrt(4.0).is_ok());
assert!(sqrt(-1.0).is_err());
```

Unlike the design-by-contract [`requires!`](https://docs.rs/out-zero-contract)
macro, which *asserts* a precondition (panicking in debug), `out-zero-precond`
*checks* a precondition, returning a recoverable [`PreconditionError`] from the
caller's function instead.

## Features

| Feature | Default | Enables |
|---|---|---|
| `alloc` | off | reserved for future alloc-dependent helpers |
| `std` | off | implies `alloc` |

This crate builds with `--no-default-features` (pure `core`, no `alloc`).

## Install

```sh
cargo add out-zero-precond
```

## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
