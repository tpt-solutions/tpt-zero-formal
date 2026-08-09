# out-zero-formal

[![crates.io](https://img.shields.io/crates/v/out-zero-formal.svg)](https://crates.io/crates/out-zero-formal)
[![docs.rs](https://docs.rs/out-zero-formal/badge.svg)](https://docs.rs/out-zero-formal)
[![license](https://img.shields.io/crates/l/out-zero-formal.svg)](#license)

Zero-dependency, `no_std` formal-methods and numerical toolkit — the umbrella
crate for the [tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal)
ecosystem.

Every layer is re-exported behind a short, namespaced module, so types that
share a name across crates (for example `Distribution`, `Normal`, and
`Witness`) no longer collide at the top level.

```rust
use out_zero_formal::prelude::*;

let v = Tensor::<f64, 3>::from_fn(|i| i as f64);
let _ = (v.get(0), Normal::standard());
```

## Feature groups

| Feature   | Crates (re-exported as modules) |
|-----------|--------------------------------|
| `layer0`  | `safe_cast`, `assert_const`, `smt_lite`, `float`, `witness`, `phantom`, `type_level` |
| `layer1`  | `bounded`, `precond`, `postcond`, `ghost`, `refinement` |
| `layer2`  | `rand`, `stats`, `tensor`, `linalg`, `prob`, `sampler` |
| `layer3`  | `eigen`, `grad`, `solver`, `decomp`, `monte_carlo`, `bayes`, `dist`, `markov`, `fsm`, `contract`, `invariant`, `loop_inv` |

The modules are also individually selectable (e.g. `prob`, `linalg`).

## Install

```sh
cargo add out-zero-formal --features "layer2,layer3"
```

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your
option.
