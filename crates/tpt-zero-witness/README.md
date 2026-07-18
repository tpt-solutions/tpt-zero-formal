# tpt-zero-witness

[![crates.io](https://img.shields.io/crates/v/tpt-zero-witness.svg)](https://crates.io/crates/tpt-zero-witness)
[![docs.rs](https://docs.rs/tpt-zero-witness/badge.svg)](https://docs.rs/tpt-zero-witness)
[![license](https://img.shields.io/crates/l/tpt-zero-witness.svg)](#license)

A value paired with a proof that some property holds, for `no_std`
dependently-typed-style APIs. Part of the
[tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal) ecosystem.

`Witness<T, P>` carries a value `T` together with a (zero-sized) proof `P`
that some property of `T` holds. The proof is a type-level witness: because
it appears in the type, the compiler only lets you construct a `Witness`
when you can *name* the proof type, which in practice means you have run the
construction logic that establishes the property.

## Quick example

```rust
use tpt_zero_witness::{Proof, Witness};

/// Proof that a `u32` is non-zero.
#[derive(Clone, Copy, Debug)]
struct NonZero;
impl Proof for NonZero {}

// Safe because the caller has already established `value != 0`.
fn checked_nonzero(value: u32) -> Option<Witness<u32, NonZero>> {
    if value == 0 {
        None
    } else {
        Some(Witness::new(value))
    }
}

let w = checked_nonzero(7).unwrap();
assert_eq!(*w.value(), 7);
```

## Features

| Feature | Default | Enables |
|---|---|---|
| `alloc` | off | reserved for future alloc-dependent helpers |
| `std` | off | implies `alloc` |

This crate builds with `--no-default-features` (pure `core`, no `alloc`).

## Install

```sh
cargo add tpt-zero-witness
```

## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
