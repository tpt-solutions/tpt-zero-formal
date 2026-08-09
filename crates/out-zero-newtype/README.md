# out-zero-newtype

[![license](https://img.shields.io/crates/l/out-zero-newtype.svg)](#license)

Macro-driven, `no_std` newtype wrappers with safe inner-type access and
trait derivation. Part of the
[tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal) ecosystem.

## Quick example

```rust
use out_zero_newtype::define_newtype;

define_newtype!(Meters, f64);

let m = Meters(3.0);
let d: f64 = m.into();
assert_eq!(d, 3.0);
```

`define_newtype!` declares a tuple struct wrapping an inner type and
auto-derives common traits, plus a `From<Inner>` conversion, `into_inner`
access, and optional `Deref`/`DerefMut`. The [`Newtype`] trait ties the
wrapper to its inner type.

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

**Use [`derive_more`](https://crates.io/crates/derive_more) or [`nutype`](https://crates.io/crates/nutype) instead.** These are mature newtype-derivation crates with far broader trait and validation coverage.

`out-zero-newtype` is kept internal to the workspace because it is a zero-dependency,
`no_std` building block used by other crates here. If you specifically need
`no_std` and zero external dependencies, depend on
[`out-zero-formal`](https://crates.io/crates/out-zero-formal) (which re-exports
this functionality) rather than adding this crate directly.


## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.