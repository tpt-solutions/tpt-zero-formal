# tpt-zero-bayes

[![crates.io](https://img.shields.io/crates/v/tpt-zero-bayes.svg)](https://crates.io/crates/tpt-zero-bayes)
[![docs.rs](https://docs.rs/tpt-zero-bayes/badge.svg)](https://docs.rs/tpt-zero-bayes)
[![license](https://img.shields.io/crates/l/tpt-zero-bayes.svg)](#license)

Bayesian inference primitives and conjugate priors for `no_std`. Part of
the [tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal)
ecosystem.

Conjugate-prior pairs expose closed-form posterior updates, so inference
stays exact and allocation-free:

- **Beta–Bernoulli**: `Beta::new(alpha, beta)` with `posterior`, `mean`,
  and `pdf`.
- **Normal–Normal** (unknown mean, known variance):
  `GaussianKnownVar::new(prior_mean, prior_var, obs_var)` with `update`.
- **Gamma–Poisson** (rate): `Gamma::new(shape, rate)` with `posterior`.

The transcendental functions those formulas need (`sqrt`, `ln`, `exp`,
`gamma`) are reimplemented from scratch because this crate targets a
`core`-only environment where the `std` float methods are not available.

## Quick example

```rust
use tpt_zero_bayes::Beta;

// A uniform prior on a coin's bias: Beta(1, 1).
let prior = Beta::new(1.0, 1.0);
assert!((prior.mean() - 0.5).abs() < 1e-12);

// Observe 7 heads and 3 tails.
let posterior = prior.posterior(7, 3);
assert!((posterior.mean() - 0.8).abs() < 1e-12);
```

## Features

| Feature | Default | Enables |
|---|---|---|
| `alloc` | off | reserved for future alloc-dependent helpers |
| `std` | off | implies `alloc` |

This crate builds with `--no-default-features` (pure `core`, no `alloc`).

## Install

```sh
cargo add tpt-zero-bayes
```

## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
