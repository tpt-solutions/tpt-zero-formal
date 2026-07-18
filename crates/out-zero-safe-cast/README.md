# out-zero-safe-cast

[![crates.io](https://img.shields.io/crates/v/out-zero-safe-cast.svg)](https://crates.io/crates/out-zero-safe-cast)
[![docs.rs](https://docs.rs/out-zero-safe-cast/badge.svg)](https://docs.rs/out-zero-safe-cast)
[![license](https://img.shields.io/crates/l/out-zero-safe-cast.svg)](#license)

Panic-free, explicitly verified numeric casting traits for `no_std`. Part of
the [tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal)
ecosystem.

Rust's `as` operator silently truncates, wraps, or saturates when a cast
doesn't fit. `SafeCast::safe_cast` returns `Err` instead, for every pair of
primitive integer and float types.

## Quick example

```rust
use out_zero_safe_cast::SafeCast;

let ok: Result<u8, _> = 200i32.safe_cast();
assert_eq!(ok, Ok(200u8));

let err: Result<u8, _> = 300i32.safe_cast();
assert!(err.is_err());

let float_ok: Result<i32, _> = 4.0f64.safe_cast();
assert_eq!(float_ok, Ok(4));

let float_err: Result<i32, _> = 4.5f64.safe_cast();
assert!(float_err.is_err());
```

## Features

| Feature | Default | Enables |
|---|---|---|
| `alloc` | off | reserved for future alloc-dependent helpers |
| `std` | off | implies `alloc` |

This crate builds with `--no-default-features` (pure `core`, no `alloc`).

## Install

```sh
cargo add out-zero-safe-cast
```

## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
