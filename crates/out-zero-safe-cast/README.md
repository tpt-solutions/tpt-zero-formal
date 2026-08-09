# out-zero-safe-cast

[![license](https://img.shields.io/crates/l/out-zero-safe-cast.svg)](#license)

Panic-free, explicitly verified numeric casting traits for `no_std`. Part of
the [tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal)
ecosystem.

Rust's `as` operator silently truncates, wraps, or saturates when a cast
doesn't fit. `SafeCast::safe_cast` returns `Err` instead, for every pair of
primitive integer and float types.

## Quick example

```rust
use out_zero_safe_cast::SafeCast;

let ok: Result<u8, _> = 200i32.safe_cast();
assert_eq!(ok, Ok(200u8));

let err: Result<u8, _> = 300i32.safe_cast();
assert!(err.is_err());

let float_ok: Result<i32, _> = 4.0f64.safe_cast();
assert_eq!(float_ok, Ok(4));

let float_err: Result<i32, _> = 4.5f64.safe_cast();
assert!(float_err.is_err());
```

## Features

| Feature | Default | Enables |
|---|---|---|
| `alloc` | off | reserved for future alloc-dependent helpers |
| `std` | off | implies `alloc` |

This crate builds with `--no-default-features` (pure `core`, no `alloc`).

## Status: not published

This crate is **not published to crates.io**. The `tpt-zero-formal` project
does not publish a crate unless it is more useful than what is already
available, and for this need a more complete, widely-used alternative already
exists.

**Use [`az`](https://crates.io/crates/az) instead.** It is a mature, well-tested casting crate (`Cast`/`CheckedAs`/`SaturatingAs`) covering the same cases.

`out-zero-safe-cast` is kept internal to the workspace because it is a zero-dependency,
`no_std` building block used by other crates here. If you specifically need
`no_std` and zero external dependencies, depend on
[`out-zero-formal`](https://crates.io/crates/out-zero-formal) (which re-exports
this functionality) rather than adding this crate directly.


## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.