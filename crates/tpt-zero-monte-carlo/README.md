# tpt-zero-monte-carlo

[![crates.io](https://img.shields.io/crates/v/tpt-zero-monte-carlo.svg)](https://crates.io/crates/tpt-zero-monte-carlo)
[![docs.rs](https://docs.rs/tpt-zero-monte-carlo/badge.svg)](https://docs.rs/tpt-zero-monte-carlo)
[![license](https://img.shields.io/crates/l/tpt-zero-monte-carlo.svg)](#license)

Monte Carlo simulation and variance reduction for `no_std`, zero external
dependencies. Part of the
[tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal) ecosystem.

## Quick example

```rust
use tpt_zero_rand::{Pcg32, Rng, SeedableRng};
use tpt_zero_monte_carlo::monte_carlo_integral;

let mut rng = Pcg32::seed_from_u64(7);
// ∫_0^1 x dx = 0.5
let estimate = monte_carlo_integral(|x| x, &mut rng, 10_000);
assert!((estimate - 0.5).abs() < 0.05);
```

The crate provides plain Monte Carlo integration over `[0, 1]`, a mean/standard
error estimator, antithetic-variate and control-variate variance reduction, and
a tool to measure the variance of the estimator itself across replications.

## Features

| Feature | Default | Enables |
|---|---|---|
| `alloc` | off | reserved for future alloc-dependent helpers |
| `std` | off | implies `alloc` |

This crate builds with `--no-default-features` (pure `core`, no `alloc`).

## Install

```sh
cargo add tpt-zero-monte-carlo
```

## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
