# tpt-zero-ghost

[![crates.io](https://img.shields.io/crates/v/tpt-zero-ghost.svg)](https://crates.io/crates/tpt-zero-ghost)
[![docs.rs](https://docs.rs/tpt-zero-ghost/badge.svg)](https://docs.rs/tpt-zero-ghost)
[![license](https://img.shields.io/crates/l/tpt-zero-ghost.svg)](#license)

Separation-logic-style ghost-state markers (`Proven`/`Unproven`) for
`no_std`. Part of the
[tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal) ecosystem.

## Quick example

```rust
use tpt_zero_ghost::{Ghost, GhostProven, Proven, Unproven};

// A value whose property has not yet been established.
let raw: Ghost<u32, Unproven> = Ghost::new(42);

// The caller asserts the invariant; this is a ghost operation and is
// unsound unless the caller actually maintains it.
let proven: GhostProven<u32> = raw.assume_proven();

// The value is carried unchanged, at zero runtime cost.
assert_eq!(*proven.value(), 42);
```

A `Ghost<T, P>` has exactly the same size, alignment, and layout as `T`
itself — the provenance marker `P` is stored as `PhantomData` and is erased
at runtime, leaving zero cost behind.

## Features

| Feature | Default | Enables |
|---|---|---|
| `witness` | on | sound `Ghost::prove` witness integration with `tpt-zero-witness` |
| `alloc` | off | reserved for future alloc-dependent helpers |
| `std` | off | implies `alloc` |

This crate builds with `--no-default-features` (pure `core`, no `alloc`): with
the default `witness` feature enabled `Ghost::prove` is available; with
`--no-default-features` only the unsound `assume_proven` remains.

## Install

```sh
cargo add tpt-zero-ghost
```

## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
