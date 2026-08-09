# out-zero-precond

[![license](https://img.shields.io/crates/l/out-zero-precond.svg)](#license)

Precondition checking helpers built on design-by-contract for `no_std`.
Part of the
[tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal) ecosystem.

## Quick example

```rust
use out_zero_precond::{check, PreconditionError};

fn sqrt(x: f64) -> Result<f64, PreconditionError> {
    check(x >= 0.0, "x must be non-negative")?;
    Ok(x.sqrt())
}

assert!(sqrt(4.0).is_ok());
assert!(sqrt(-1.0).is_err());
```

Unlike the design-by-contract [`requires!`](https://docs.rs/out-zero-contract)
macro, which *asserts* a precondition (panicking in debug), `out-zero-precond`
*checks* a precondition, returning a recoverable [`PreconditionError`] from the
caller's function instead.

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

**Use [`contracts`](https://crates.io/crates/contracts) instead.** It is a more complete design-by-contract toolkit that also covers precondition checking.

`out-zero-precond` is kept internal to the workspace because it is a zero-dependency,
`no_std` building block used by other crates here. If you specifically need
`no_std` and zero external dependencies, depend on
[`out-zero-formal`](https://crates.io/crates/out-zero-formal) (which re-exports
this functionality) rather than adding this crate directly.


## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.