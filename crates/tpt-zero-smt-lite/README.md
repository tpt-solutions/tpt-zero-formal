# tpt-zero-smt-lite

[![crates.io](https://img.shields.io/crates/v/tpt-zero-smt-lite.svg)](https://crates.io/crates/tpt-zero-smt-lite)
[![docs.rs](https://docs.rs/tpt-zero-smt-lite/badge.svg)](https://docs.rs/tpt-zero-smt-lite)
[![license](https://img.shields.io/crates/l/tpt-zero-smt-lite.svg)](#license)

Boolean and linear integer constraint helpers with a tiny `no_std` SMT-lite
solver for checking satisfiability at runtime. Part of the
[tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal) ecosystem.

## Quick example

```rust
use tpt_zero_smt_lite::{Var, constraint_set, boolean::{and, or}, linear_leq, solve};

let a = Var(0);
let b = Var(1);

let mut cs = constraint_set();
cs.add(and(or(a, b), boolean::not(a)));
cs.add(linear_leq(&[(1, a), (1, b)], 1));

assert!(solve(&cs).is_some());
```

`solve` returns a `Witness` mapping each [`Var`] to a `bool` (and, for
integer-variable problems, to an `i64`) when the constraint set is
satisfiable, or `None` otherwise.

## Features

| Feature | Default | Enables |
|---|---|---|
| `alloc` | off | reserved for future heap-backed enumeration |
| `std` | off | implies `alloc` |

This crate builds with `--no-default-features` (pure `core`, no `alloc`). The
solver performs bounded enumeration over a fixed set of boolean variables and
does not require a heap.

## Install

```sh
cargo add tpt-zero-smt-lite
```

## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
