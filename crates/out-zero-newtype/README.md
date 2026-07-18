# out-zero-newtype

[![crates.io](https://img.shields.io/crates/v/out-zero-newtype.svg)](https://crates.io/crates/out-zero-newtype)
[![docs.rs](https://docs.rs/out-zero-newtype/badge.svg)](https://docs.rs/out-zero-newtype)
[![license](https://img.shields.io/crates/l/out-zero-newtype.svg)](#license)

Macro-driven, `no_std` newtype wrappers with safe inner-type access and
trait derivation. Part of the
[tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal) ecosystem.

## Quick example

```rust
use out_zero_newtype::define_newtype;

define_newtype!(Meters, f64);

let m = Meters(3.0);
let d: f64 = m.into();
assert_eq!(d, 3.0);
```

`define_newtype!` declares a tuple struct wrapping an inner type and
auto-derives common traits, plus a `From<Inner>` conversion, `into_inner`
access, and optional `Deref`/`DerefMut`. The [`Newtype`] trait ties the
wrapper to its inner type.

## Features

| Feature | Default | Enables |
|---|---|---|
| `alloc` | off | reserved for future alloc-dependent helpers |
| `std` | off | implies `alloc` |

This crate builds with `--no-default-features` (pure `core`, no `alloc`).

## Install

```sh
cargo add out-zero-newtype
```

## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
