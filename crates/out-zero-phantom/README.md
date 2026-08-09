# out-zero-phantom

[![license](https://img.shields.io/crates/l/out-zero-phantom.svg)](#license)

Zero-cost phantom-type markers and variance patterns for `no_std`. Part of
the [tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal)
ecosystem.

## Quick example

```rust
use out_zero_phantom::{Phantom, Covariant, marker};

// A zero-sized wrapper that carries a phantom tag.
let tagged: Phantom<&str> = Phantom::new();
assert_eq!(core::mem::size_of::<Phantom<&str>>(), 0);

// Declare a named zero-sized marker type.
marker!(pub UserId);
let id = UserId::new();
assert_eq!(core::mem::size_of_val(&id), 0);

// Variance markers expose the covariance/invariance of a parameter.
let _cov: Covariant<&str> = Covariant::new();
```

A `Phantom<T>` carries the type `T` only at the type level: the value has
size `0` and no `T` is ever stored, so you can tag values and drive the
borrow checker without any runtime cost.

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

**Use [`core::marker::PhantomData`](https://doc.rust-lang.org/core/marker/struct.PhantomData.html) from the standard library instead.** This crate is a thin convenience wrapper over `PhantomData`; the standard type covers the zero-cost marker use case directly.

`out-zero-phantom` is kept internal to the workspace because it is a zero-dependency,
`no_std` building block used by other crates here. If you specifically need
`no_std` and zero external dependencies, depend on
[`out-zero-formal`](https://crates.io/crates/out-zero-formal) (which re-exports
this functionality) rather than adding this crate directly.


## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.