# out-zero-refinement

[![crates.io](https://img.shields.io/crates/v/out-zero-refinement.svg)](https://crates.io/crates/out-zero-refinement)
[![docs.rs](https://docs.rs/out-zero-refinement/badge.svg)](https://docs.rs/out-zero-refinement)
[![license](https://img.shields.io/crates/l/out-zero-refinement.svg)](#license)

Refinement types for `no_std`: a value of type `T` that is known to satisfy a
compile-time-named predicate `P`. Part of the
[tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal) ecosystem.

## Quick example

```rust
use out_zero_refinement::{Predicate, Refined};

/// Predicate: a `u32` that is non-zero.
struct NonZeroU32;
impl Predicate<u32> for NonZeroU32 {
    fn check(value: &u32) -> bool {
        *value != 0
    }
}

let ok = Refined::<u32, NonZeroU32>::new(7).unwrap();
assert_eq!(*ok.get(), 7);
assert_eq!(*ok, 7); // via Deref

assert!(Refined::<u32, NonZeroU32>::new(0).is_err());
```

A `Refined<T, P>` can only be built through [`Refined::new`] (or `TryFrom`),
which runs `P::check`; once constructed it is a proof-carrying wrapper whose
invariant holds for its whole lifetime. `P` is zero-sized, so `Refined<T, P>`
has the same size as `T`.

## Features

| Feature | Default | Enables |
|---|---|---|
| `alloc` | off | reserved for future alloc-dependent helpers |
| `std` | off | implies `alloc` |

This crate builds with `--no-default-features` (pure `core`, no `alloc`).

## Install

```sh
cargo add out-zero-refinement
```

## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
