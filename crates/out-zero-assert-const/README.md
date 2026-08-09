# out-zero-assert-const

[![license](https://img.shields.io/crates/l/out-zero-assert-const.svg)](#license)

Compile-time assertion macros and `const fn` helpers for `no_std` const
contexts. Part of the
[tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal) ecosystem.

## Quick example

```rust
use out_zero_assert_const::{const_assert, const_assert_eq, assert_i64_le};

const_assert!(core::mem::size_of::<u32>() == 4);
const_assert_eq!(core::mem::size_of::<u8>(), 1);

const MAX: i64 = assert_i64_le(0, 100);
assert_eq!(MAX, 100);
```

A failing `const_assert!` is a compile error, not a runtime panic — the
condition is checked once, at build time, for every concrete instantiation
that is actually used (e.g. every distinct `BoundedInt<MIN, MAX>` pair).

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

**Use [`static_assertions`](https://crates.io/crates/static_assertions) instead.** It provides a broader, actively maintained set of compile-time assertion macros (`const_assert!`, `assert_eq_size!`, type and trait assertions, and more).

`out-zero-assert-const` is kept internal to the workspace because it is a zero-dependency,
`no_std` building block used by other crates here. If you specifically need
`no_std` and zero external dependencies, depend on
[`out-zero-formal`](https://crates.io/crates/out-zero-formal) (which re-exports
this functionality) rather than adding this crate directly.


## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.