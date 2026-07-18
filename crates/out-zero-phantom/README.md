# out-zero-phantom

[![crates.io](https://img.shields.io/crates/v/out-zero-phantom.svg)](https://crates.io/crates/out-zero-phantom)
[![docs.rs](https://docs.rs/out-zero-phantom/badge.svg)](https://docs.rs/out-zero-phantom)
[![license](https://img.shields.io/crates/l/out-zero-phantom.svg)](#license)

Zero-cost phantom-type markers and variance patterns for `no_std`. Part of
the [tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal)
ecosystem.

## Quick example

```rust
use out_zero_phantom::{Phantom, Covariant, marker};

// A zero-sized wrapper that carries a phantom tag.
let tagged: Phantom<&str> = Phantom::new();
assert_eq!(core::mem::size_of::<Phantom<&str>>(), 0);

// Declare a named zero-sized marker type.
marker!(pub UserId);
let id = UserId::new();
assert_eq!(core::mem::size_of_val(&id), 0);

// Variance markers expose the covariance/invariance of a parameter.
let _cov: Covariant<&str> = Covariant::new();
```

A `Phantom<T>` carries the type `T` only at the type level: the value has
size `0` and no `T` is ever stored, so you can tag values and drive the
borrow checker without any runtime cost.

## Features

| Feature | Default | Enables |
|---|---|---|
| `alloc` | off | reserved for future alloc-dependent helpers |
| `std` | off | implies `alloc` |

This crate builds with `--no-default-features` (pure `core`, no `alloc`).

## Install

```sh
cargo add out-zero-phantom
```

## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
