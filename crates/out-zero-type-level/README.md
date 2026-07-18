# out-zero-type-level

[![crates.io](https://img.shields.io/crates/v/out-zero-type-level.svg)](https://crates.io/crates/out-zero-type-level)
[![docs.rs](https://docs.rs/out-zero-type-level/badge.svg)](https://docs.rs/out-zero-type-level)
[![license](https://img.shields.io/crates/l/out-zero-type-level.svg)](#license)

Type-level unsigned integer arithmetic and constraints, built on `const`
generics, for `no_std` targets. Part of the
[tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal) ecosystem.

## Quick example

```rust
use out_zero_type_level::{U, Add, Sum};

type Three = U<3>;
type Four = U<4>;
type Seven = <Three as Add<Four>>::Output;

assert_eq!(<Seven as Add<U<0>>>::VALUE, 7);
```

A violation of a type-level constraint is a compile error, not a runtime
panic:

```rust,compile_fail
use out_zero_type_level::{U, const_assert_le};

// 5 is not <= 3, so this fails to compile.
const_assert_le::<U<5>, U<3>>();
```

## Features

| Feature | Default | Enables |
|---|---|---|
| `alloc` | off | reserved for future alloc-dependent helpers |
| `std` | off | implies `alloc` |

This crate builds with `--no-default-features` (pure `core`, no `alloc`).

## Install

```sh
cargo add out-zero-type-level
```

## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
