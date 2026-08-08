# tpt-zero-stats

[![crates.io](https://img.shields.io/crates/v/tpt-zero-stats.svg)](https://crates.io/crates/tpt-zero-stats)
[![docs.rs](https://docs.rs/tpt-zero-stats/badge.svg)](https://docs.rs/tpt-zero-stats)
[![license](https://img.shields.io/crates/l/tpt-zero-stats.svg)](#license)

`no_std` descriptive statistics — mean, variance, standard deviation, min,
max, median, and percentiles/quantiles — over `f64` slices and iterators.
Part of the
[tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal) ecosystem.

## Quick example

```rust
use tpt_zero_stats::{mean, variance, std_dev, percentile};

let data = [1.0, 2.0, 3.0, 4.0, 5.0];
assert_eq!(mean(&data), Some(3.0));
assert_eq!(variance(&data), Some(2.5));
assert!((std_dev(&data).unwrap() - sqrt(2.5)).abs() < 1e-12);

let mut scratch = [0.0; 5];
assert_eq!(percentile(&data, 0.5, &mut scratch), Some(3.0));
```

Order-based statistics (`median`, `percentile`, `quantile`) sort a
*caller-owned* mutable scratch buffer in place, so no heap allocation is
required in the `default` (`no_std`) configuration.

## Features

| Feature | Default | Enables |
|---|---|---|
| `alloc` | off | reserved for future alloc-dependent helpers |
| `std` | off | implies `alloc` |

This crate builds with `--no-default-features` (pure `core`, no `alloc`).

## Install

```sh
cargo add tpt-zero-stats
```

## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
