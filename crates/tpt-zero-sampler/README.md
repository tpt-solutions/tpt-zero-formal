# tpt-zero-sampler

[![crates.io](https://img.shields.io/crates/v/tpt-zero-sampler.svg)](https://crates.io/crates/tpt-zero-sampler)
[![docs.rs](https://docs.rs/tpt-zero-sampler/badge.svg)](https://docs.rs/tpt-zero-sampler)
[![license](https://img.shields.io/crates/l/tpt-zero-sampler.svg)](#license)

`no_std` sampling algorithms for probabilistic inference: rejection sampling,
inverse-transform sampling, and a simple Metropolis-Hastings walker. Part of the
[tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal) ecosystem.

## Quick example

```rust
use tpt_zero_rand::{Pcg32, Rng, SeedableRng};
use tpt_zero_sampler::inverse_transform_sample;

let mut rng = Pcg32::seed_from_u64(7);
// The inverse CDF of Uniform(0,1) is the identity, so this yields uniform draws.
let x = inverse_transform_sample(|u| u, &mut rng);
assert!(x >= 0.0 && x < 1.0);
```

All algorithms are generic over any [`Rng`] from
[`tpt-zero-rand`](https://docs.rs/tpt-zero-rand) and operate purely on `f64`, so
they build with `--no-default-features` (pure `core`).

## Features

| Feature | Default | Enables |
|---|---|---|
| `alloc` | off | reserved for future alloc-dependent helpers |
| `std` | off | implies `alloc` |

This crate builds with `--no-default-features` (pure `core`, no `alloc`).

## Install

```sh
cargo add tpt-zero-sampler
```

## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
