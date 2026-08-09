# out-zero-postcond

[![license](https://img.shields.io/crates/l/out-zero-postcond.svg)](#license)

Postcondition checking helpers for `no_std`. Part of the
[tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal) ecosystem.

## Quick example

```rust
use out_zero_postcond::{postcondition, PostconditionError};

fn checked_add(a: u32, b: u32) -> Result<u32, PostconditionError> {
    let sum = a.checked_add(b).ok_or_else(|| PostconditionError::new("no overflow"))?;
    postcondition!(sum >= a, "addition did not wrap")?;
    Ok(sum)
}

assert_eq!(checked_add(2, 3), Ok(5));
```

[`postcondition!`] is zero-cost in release builds (it expands to
`debug_assert!`-style behaviour there, returning `Ok(())`), but returns
`Err(PostconditionError)` in `debug_assertions` builds when the condition is
violated. [`ensure_postcond!`] performs an early-return of the error at the
end of a function, and [`check`] always evaluates regardless of build profile.

## Features

| Feature | Default | Enables |
|---|---|---|
| `alloc` | off | reserved for future alloc-dependent helpers |
| `std` | off | implies `alloc`; also implements `std::error::Error` for `PostconditionError` |

This crate builds with `--no-default-features` (pure `core`, no `alloc`).

## Status: not published

This crate is **not published to crates.io**. The `tpt-zero-formal` project
does not publish a crate unless it is more useful than what is already
available, and for this need a more complete, widely-used alternative already
exists.

**Use [`contracts`](https://crates.io/crates/contracts) instead.** It is a more complete design-by-contract toolkit that also covers postcondition checking.

`out-zero-postcond` is kept internal to the workspace because it is a zero-dependency,
`no_std` building block used by other crates here. If you specifically need
`no_std` and zero external dependencies, depend on
[`out-zero-formal`](https://crates.io/crates/out-zero-formal) (which re-exports
this functionality) rather than adding this crate directly.


## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.