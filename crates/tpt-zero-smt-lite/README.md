# tpt-zero-smt-lite

[![crates.io](https://img.shields.io/crates/v/tpt-zero-smt-lite.svg)](https://crates.io/crates/tpt-zero-smt-lite)
[![docs.rs](https://docs.rs/tpt-zero-smt-lite/badge.svg)](https://docs.rs/tpt-zero-smt-lite)
[![license](https://img.shields.io/crates/l/tpt-zero-smt-lite.svg)](#license)

Boolean and pseudo-boolean (0/1) linear constraint helpers with a tiny `no_std`
SMT-lite solver for checking satisfiability at runtime. Part of the
[tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal) ecosystem.

## Quick example

```rust
use tpt_zero_smt_lite::{Var, ConstraintSet};

let a = Var(0);
let b = Var(1);

let cs = ConstraintSet::new(2);
let expr = cs.and(cs.or(cs.var(a), cs.var(b)), cs.not(cs.var(a)));
cs.add_bool(expr);
// Pseudo-boolean (0/1) constraint: at most one of a, b is true.
cs.pb_leq(&[(1, a), (1, b)], 1);

let witness = cs.solve().expect("the constraints are satisfiable");
assert_eq!(witness.bool_of(a), Some(false));
assert_eq!(witness.bool_of(b), Some(true));
```

`solve` returns a `Witness` mapping each [`Var`] to a `bool` (and, via
[`Witness::int_of`], to `0`/`1`) when the constraint set is satisfiable, or
`None` otherwise.

## A note on "linear" constraints

The `pb_eq` / `pb_leq` constraints are **pseudo-boolean** constraints, not
general integer-linear constraints. Every variable is a boolean and is read as
the integer `1` (when true) or `0` (when false), so they describe linear
relations over the `0`/`1` assignment. There are no arbitrary `i64` variables.

## Export to external provers

With the `alloc` feature, the [`export`](https://docs.rs/tpt-zero-smt-lite)
module translates a [`ConstraintSet`] into an SMT-LIB 2.6 script
(`export::to_smtlib`) or a Why3 module (`export::to_why3`). The same
constraints can then be re-checked independently by an SMT solver or a
deductive proof assistant — useful when pairing with the formal-verification
backends described by `out-zero-contract`'s `formal` feature.

## Features

| Feature | Default | Enables |
|---|---|---|
| `alloc` | off | the `export` module (SMT-LIB / Why3 serialisation) |
| `std` | off | implies `alloc` |

This crate builds with `--no-default-features` (pure `core`, no `alloc`). The
solver performs bounded enumeration over a fixed set of boolean variables and
does not require a heap. Because the enumeration visits `2^num_vars`
assignments, it scales only to small problems (up to 63 variables on a 64-bit
target).

## Install

```sh
cargo add tpt-zero-smt-lite
```

## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
