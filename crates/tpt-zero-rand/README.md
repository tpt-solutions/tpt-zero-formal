# tpt-zero-rand

[![crates.io](https://img.shields.io/crates/v/tpt-zero-rand.svg)](https://crates.io/crates/tpt-zero-rand)
[![docs.rs](https://docs.rs/tpt-zero-rand/badge.svg)](https://docs.rs/tpt-zero-rand)
[![license](https://img.shields.io/crates/l/tpt-zero-rand.svg)](#license)

Self-contained, seedable pseudo-random number generators for `no_std`. Part of
the [tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal)
ecosystem.

Rust's standard `rand` crate pulls in `std` and external dependencies. This
crate implements two small generators from scratch — [`XorShift64`] (xorshift)
and [`Pcg32`] (PCG-XSH-RR) — with zero production dependencies, so they work in
`no_std` and in `#![forbid(unsafe_code)]` contexts.

## Quick example

```rust
use tpt_zero_rand::{Pcg32, Rng, SeedableRng};

let mut rng = Pcg32::seed_from_u64(42);
let u: u32 = rng.next_u32();
let f: f64 = rng.next_f64();
assert!(f >= 0.0 && f < 1.0);
```

Both generators implement [`Rng`] (with `next_u32`, `next_u64`, `next_f64`, and
`fill_bytes`) and [`SeedableRng`] (with `from_seed` / `seed_from_u64`). The same
seed always produces the same stream, which makes these generators ideal for
reproducible tests and simulations.

## Features

| Feature | Default | Enables |
|---|---|---|
| `alloc` | off | reserved for future alloc-dependent helpers |
| `std` | off | implies `alloc` |

This crate builds with `--no-default-features` (pure `core`, no `alloc`).

## Install

```sh
cargo add tpt-zero-rand
```

## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
