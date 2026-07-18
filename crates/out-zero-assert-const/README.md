# out-zero-assert-const

[![crates.io](https://img.shields.io/crates/v/out-zero-assert-const.svg)](https://crates.io/crates/out-zero-assert-const)
[![docs.rs](https://docs.rs/out-zero-assert-const/badge.svg)](https://docs.rs/out-zero-assert-const)
[![license](https://img.shields.io/crates/l/out-zero-assert-const.svg)](#license)

Compile-time assertion macros and `const fn` helpers for `no_std` const
contexts. Part of the
[tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal) ecosystem.

## Quick example

```rust
use out_zero_assert_const::{const_assert, const_assert_eq, assert_i64_le};

const_assert!(core::mem::size_of::<u32>() == 4);
const_assert_eq!(core::mem::size_of::<u8>(), 1);

const MAX: i64 = assert_i64_le(0, 100);
assert_eq!(MAX, 100);
```

A failing `const_assert!` is a compile error, not a runtime panic — the
condition is checked once, at build time, for every concrete instantiation
that is actually used (e.g. every distinct `BoundedInt<MIN, MAX>` pair).

## Features

| Feature | Default | Enables |
|---|---|---|
| `alloc` | off | reserved for future alloc-dependent helpers |
| `std` | off | implies `alloc` |

This crate builds with `--no-default-features` (pure `core`, no `alloc`).

## Install

```sh
cargo add out-zero-assert-const
```

## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
