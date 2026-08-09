# tpt-zero-markov

[![crates.io](https://img.shields.io/crates/v/tpt-zero-markov.svg)](https://crates.io/crates/tpt-zero-markov)
[![docs.rs](https://docs.rs/tpt-zero-markov/badge.svg)](https://docs.rs/tpt-zero-markov)
[![license](https://img.shields.io/crates/l/tpt-zero-markov.svg)](#license)

Discrete-time Markov chains and transition-matrix operations for `no_std`,
over fixed-size tensors. Part of the
[tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal) ecosystem.

## Quick example

```rust
use tpt_zero_markov::Chain;
use tpt_zero_tensor::{Tensor, Tensor2};

let p = Tensor2::from([[0.9, 0.1], [0.5, 0.5]]);
let chain = Chain::new(p, Tensor::from([1.0, 0.0])).unwrap();

// One step of the initial distribution:
let next = chain.step(&chain.initial());
assert!((next[0] - 0.9).abs() < 1e-12);

// The stationary distribution satisfies pi P = pi.
let pi = chain.stationary().unwrap();
let balanced = chain.step(&pi);
for i in 0..2 {
    assert!((balanced[i] - pi[i]).abs() < 1e-9);
}
```

A [`Chain`] stores a row-stochastic transition matrix and an initial
distribution. Build it with [`Chain::new`] (which normalizes rows) or
[`Chain::checked_new`] (which rejects non-stochastic rows), then use
[`Chain::step`], [`Chain::n_step`], [`Chain::stationary`], and
`Chain::sample` (with the `alloc` feature).

## Features

| Feature | Default | Enables |
|---|---|---|
| `alloc` | off | `sample` returns a `Vec<usize>` trajectory |
| `std` | off | implies `alloc` |

This crate builds with `--no-default-features` (pure `core`, no `alloc`).

## Install

```sh
cargo add tpt-zero-markov
```

## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
