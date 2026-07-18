# out-zero-loop-inv

[![crates.io](https://img.shields.io/crates/v/out-zero-loop-inv.svg)](https://crates.io/crates/out-zero-loop-inv)
[![docs.rs](https://docs.rs/out-zero-loop-inv/badge.svg)](https://docs.rs/out-zero-loop-inv)
[![license](https://img.shields.io/crates/l/out-zero-loop-inv.svg)](#license)

Loop-invariant checking helpers: zero-cost per-iteration assertions for
`no_std`. Part of the
[tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal) ecosystem.

## Quick example

```rust
use out_zero_loop_inv::loop_invariant;

let mut sum: u64 = 0;
for i in 0..10u64 {
    loop_invariant!(sum == i * (i + 1) / 2);
    sum += i + 1;
}
assert_eq!(sum, 10 * 11 / 2);
```

A `loop_invariant!` is checked on every iteration with `debug_assert!`, so it
is compiled away in release builds and costs nothing in the final binary. The
[`LoopInvariant`] trait additionally separates the *entry*
([`LoopInvariant::establish`]) and *per-iteration*
([`LoopInvariant::maintain`]) predicates so both phases can be verified
independently — this reuses the [`tpt-zero-invariant`] crate's `Invariant`
machinery via the `check_loop_entry!` / `check_loop_invariant!` macros.

## Features

| Feature | Default | Enables |
|---|---|---|
| `alloc` | off | reserved for future alloc-dependent helpers |
| `std` | off | implies `alloc` |

This crate builds with `--no-default-features` (pure `core`, no `alloc`).

## Install

```sh
cargo add out-zero-loop-inv
```

## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
