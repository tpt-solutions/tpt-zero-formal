# tpt-zero-invariant

[![crates.io](https://img.shields.io/crates/v/tpt-zero-invariant.svg)](https://crates.io/crates/tpt-zero-invariant)
[![docs.rs](https://docs.rs/tpt-zero-invariant/badge.svg)](https://docs.rs/tpt-zero-invariant)
[![license](https://img.shields.io/crates/l/tpt-zero-invariant.svg)](#license)

State-machine invariant traits and zero-cost assertion macros for `no_std`.
Part of the
[tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal) ecosystem.

## Quick example

```rust
use tpt_zero_invariant::{Invariant, invariant};

struct Counter { count: u32, max: u32 }

impl Invariant for Counter {
    fn check(&self) -> bool {
        self.count <= self.max
    }
}

let mut c = Counter { count: 0, max: 10 };
c.count += 1;
invariant!(c.count <= c.max);
```

A failing `invariant!` is a `debug_assert!`, so the check is compiled away
in release builds and costs nothing in optimized code. The same property can
be checked through the [`Invariant`] trait via [`check_invariant!`] or the
[`assert_invariant`] / [`assert_invariant_mut`] helpers.

## Features

| Feature | Default | Enables |
|---|---|---|
| `alloc` | off | reserved for future alloc-dependent helpers |
| `std` | off | implies `alloc` |

This crate builds with `--no-default-features` (pure `core`, no `alloc`).

## Install

```sh
cargo add tpt-zero-invariant
```

## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
