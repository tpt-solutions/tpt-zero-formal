# tpt-zero-dist

[![crates.io](https://img.shields.io/crates/v/tpt-zero-dist.svg)](https://crates.io/crates/tpt-zero-dist)
[![docs.rs](https://docs.rs/tpt-zero-dist/badge.svg)](https://docs.rs/tpt-zero-dist)
[![license](https://img.shields.io/crates/l/tpt-zero-dist.svg)](#license)

Concrete probability distributions for `no_std`: `Uniform`, `Normal`,
`Bernoulli`, and `Poisson`, each with analytic PDF/PMF, CDF, mean, and
variance, plus random sampling from any `tpt-zero-rand` generator. Part of the
[tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal) ecosystem.

## Quick example

```rust
use tpt_zero_dist::{Distribution, Normal};
use tpt_zero_rand::{Pcg32, SeedableRng};

let n = Normal::new(0.0, 1.0).unwrap();
assert_eq!(n.mean(), 0.0);
assert_eq!(n.variance(), 1.0);

let mut rng = Pcg32::seed_from_u64(42);
let x = n.sample(&mut rng);
assert!(x.is_finite());
```

Sampling uses textbook algorithms — inverse-transform scaling for `Uniform`,
the Box–Muller transform for `Normal`, a Bernoulli trial for `Bernoulli`, and
Knuth's algorithm for `Poisson`. The transcendental functions these need
(`sqrt`, `ln`, `exp`, `floor`) are reimplemented from scratch in the `math`
module because this crate targets a `core`-only environment, so it has zero
external production dependencies beyond its `tpt-zero-*` siblings and builds in
pure `core`.

## Features

| Feature | Default | Enables |
|---|---|---|
| `alloc` | off | reserved for future alloc-dependent helpers |
| `std` | off | implies `alloc` |

This crate builds with `--no-default-features` (pure `core`, no `alloc`).

## Install

```sh
cargo add tpt-zero-dist
```

## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
