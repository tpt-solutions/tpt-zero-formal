# tpt-zero-float

[![crates.io](https://img.shields.io/crates/v/tpt-zero-float.svg)](https://crates.io/crates/tpt-zero-float)
[![docs.rs](https://docs.rs/tpt-zero-float/badge.svg)](https://docs.rs/tpt-zero-float)
[![license](https://img.shields.io/crates/l/tpt-zero-float.svg)](#license)

Verified, `no_std` floating-point primitives: [`sqrt`], [`exp`], and [`ln`] for
`f64`. Part of the
[tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal) ecosystem.

These are implemented from scratch in `core` (no `libm`, no FPU-runtime
dependency) so they work on bare-metal targets without an FPU math runtime.
Unlike ad-hoc copies, they handle subnormals, overflow, and underflow correctly
and converge to full `f64` precision (relative tolerance, fixed iteration cap).

```rust
use tpt_zero_float::{sqrt, exp, ln};

assert!((sqrt(2.0) - core::f64::consts::SQRT_2).abs() < 1e-12);
assert!((exp(1.0) - core::f64::consts::E).abs() < 1e-12);
assert!((ln(core::f64::consts::E) - 1.0).abs() < 1e-12);
```

[`sqrt`]: https://docs.rs/tpt-zero-float/latest/tpt_zero_float/fn.sqrt.html
[`exp`]: https://docs.rs/tpt-zero-float/latest/tpt_zero_float/fn.exp.html
[`ln`]: https://docs.rs/tpt-zero-float/latest/tpt_zero_float/fn.ln.html

## Install

```sh
cargo add tpt-zero-float
```

## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
