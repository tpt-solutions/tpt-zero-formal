# out-zero-contract

[![license](https://img.shields.io/crates/l/out-zero-contract.svg)](#license)

Design-by-contract precondition and postcondition macros for `no_std`, with
a `checked` mode for always-on assertions. Part of the
[tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal) ecosystem.

## Quick example

```rust
use out_zero_contract::{requires, ensures};

fn div(a: i32, b: i32) -> i32 {
    requires!(b != 0, "divisor must be non-zero");
    let q = a / b;
    ensures!(b * q == a - (a % b), "division identity holds");
    q
}

assert_eq!(div(10, 2), 5);
```

By default `requires!` and `ensures!` expand to `debug_assert!`, so they are
compiled away in release builds (zero-cost). Enable the `checked` feature to
make them use `assert!` instead, enforcing contracts even in release.

## Requirement traceability

A precondition or postcondition can be tagged with a requirement identifier
using the `REQ=` syntax. The tag is ignored at runtime (so it is zero-cost)
but is extracted by `cargo xtask certify` to build a traceability matrix that
links source-level contracts back to system requirements.

```rust
use out_zero_contract::requires;

fn altitude_hold(altitude_ft: f64, armed: bool) {
    requires!(REQ = "SRS-ALT-014", altitude_ft >= 0.0 && armed, "must be armed and positive");
}
```

## MC/DC-oriented contracts

`mcdc_requires!` asserts each independent condition *separately*, so a failing
sub-condition is pinpointed instead of being masked by short-circuit
evaluation. Pair it with `rustc -Z coverage-options=mcdc` (nightly) to verify
every condition is independently exercised.

## Features

| Feature | Default | Enables |
|---|---|---|
| `checked` | off | makes `requires!`/`ensures!` always assert (via `assert!`) rather than only in debug |
| `formal` | off | the `formal` module: anchors and docs for discharging contracts with Kani / Creusot / Prusti |
| `alloc` | off | reserved for future alloc-dependent helpers |
| `std` | off | implies `alloc` |

This crate builds with `--no-default-features` (pure `core`, no `alloc`).

## Formal verification

With the `formal` feature, `out-zero-contract` exposes a `formal` module and a
`proof!` marker macro. These are documentation-only anchors (no runtime
effect); an external backend does the actual discharge:

- **Kani** — wrap a `requires!`/`ensures!`-guarded function in a Kani harness.
- **Creusot** — translates Rust to Why3; combine with `tpt-zero-smt-lite`'s
  `export` module (the `alloc` feature) to re-check the same constraints as a
  Why3 theory.
- **Prusti** — reads `requires`/`ensures` specification attributes; `proof!`
  marks the block whose invariants Prusti proves.

## Status: not published

This crate is **not published to crates.io**. The `tpt-zero-formal` project
does not publish a crate unless it is more useful than what is already
available, and for this need a more complete, widely-used alternative already
exists.

**Use [`contracts`](https://crates.io/crates/contracts) instead.** It is the de-facto design-by-contract crate, with a much larger `requires!`/`ensures!` macro suite and tooling.

`out-zero-contract` is kept internal to the workspace because it is a zero-dependency,
`no_std` building block used by other crates here. If you specifically need
`no_std` and zero external dependencies, depend on
[`out-zero-formal`](https://crates.io/crates/out-zero-formal) (which re-exports
this functionality) rather than adding this crate directly.


## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.