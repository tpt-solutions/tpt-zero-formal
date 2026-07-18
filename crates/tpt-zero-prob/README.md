# tpt-zero-prob

[![crates.io](https://img.shields.io/crates/v/tpt-zero-prob.svg)](https://crates.io/crates/tpt-zero-prob)
[![docs.rs](https://docs.rs/tpt-zero-prob/badge.svg)](https://docs.rs/tpt-zero-prob)
[![license](https://img.shields.io/crates/l/tpt-zero-prob.svg)](#license)

Probability building blocks for `no_std`: a `Distribution` trait that captures
the analytic properties of a probability distribution, and a `Dist` container
that holds observed samples and derives empirical statistics from them. Part
of the
[tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal) ecosystem.

## Quick example

```rust
use tpt_zero_prob::{Dist, Distribution};

// An empirical sample viewed over a borrowed slice (no allocation).
let samples = [0.1, 0.5, 0.9, 0.2, 0.8];
let dist = Dist::new(&samples);
assert!(dist.empirical_mean().is_some());

// An analytic distribution.
let u = tpt_zero_prob::distributions::Uniform::new(0.0, 1.0).unwrap();
assert_eq!(u.mean(), Some(0.5));
assert!((u.variance().unwrap() - 1.0 / 12.0).abs() < 1e-12);
```

`Dist` borrows its samples from a `&[f64]` in the default configuration (no
allocation), and the empirical statistics are delegated to
[`tpt-zero-stats`](https://docs.rs/tpt-zero-stats), so the crate has zero
external production dependencies and builds in pure `core`.

## Features

| Feature | Default | Enables |
|---|---|---|
| `alloc` | off | owned `Dist` storage via `Vec<f64>` |
| `std` | off | implies `alloc` |

This crate builds with `--no-default-features` (pure `core`, samples held as a
borrowed `&[f64]`).

## Install

```sh
cargo add tpt-zero-prob
```

## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
