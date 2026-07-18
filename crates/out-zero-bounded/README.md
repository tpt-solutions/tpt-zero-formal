# out-zero-bounded

[![crates.io](https://img.shields.io/crates/v/out-zero-bounded.svg)](https://crates.io/crates/out-zero-bounded)
[![docs.rs](https://docs.rs/out-zero-bounded/badge.svg)](https://docs.rs/out-zero-bounded)
[![license](https://img.shields.io/crates/l/out-zero-bounded.svg)](#license)

Zero-cost bounds-checked integers for `no_std`, built on
[out-zero-safe-cast](https://crates.io/crates/out-zero-safe-cast) and
[out-zero-assert-const](https://crates.io/crates/out-zero-assert-const). Part
of the [tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal)
ecosystem.

`BoundedInt<MIN, MAX>` wraps an `i64` and guarantees at compile time
(`MIN <= MAX`) and at construction (`MIN <= value <= MAX`) that the value
always stays inside the range. Out-of-range construction returns
`BoundsError`, and the arithmetic helpers saturate/clamp to the bounds instead
of overflowing.

## Quick example

```rust
use out_zero_bounded::{BoundedInt, BoundsError};

type Percent = BoundedInt<0, 100>;

let x = Percent::new(50).unwrap();
assert_eq!(x.value(), 50);

// Clamps instead of overflowing:
let clamped = x.saturating_add(200);
assert_eq!(clamped.value(), 100);

// An inverted range is a compile error (the type won't instantiate).
// type Bad = BoundedInt<100, 0>; // <- fails to compile

assert!(matches!(Percent::new(101), Err(BoundsError)));
```

A range with `MIN > MAX` is rejected at the point the type is instantiated
(via `assert_i64_le`/`const_assert!`), so it is a compile error rather than a
runtime panic.

## Features

| Feature | Default | Enables |
|---|---|---|
| `alloc` | off | reserved for future alloc-dependent helpers |
| `std` | off | implies `alloc` |

This crate builds with `--no-default-features` (pure `core`, no `alloc`).

## Install

```sh
cargo add out-zero-bounded
```

## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
