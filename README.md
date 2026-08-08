# tpt-zero-formal

Runtime building blocks for formal verification, design-by-contract, and
probabilistic computing in Rust — the bridge between high-assurance TPT languages
(`tpt-telos`, `tpt-eidos`) and safe, mathematically sound Rust execution, and
the foundational math for `tpt-augur`.

Rust's borrow checker prevents memory-safety issues, but not logical ones:
dividing by zero, state-machine violations, out-of-bounds indices. And
probabilistic / AI systems need rigorous, deterministic, sound math primitives
that standard crates don't provide in a `no_std` context. `tpt-zero-formal`
provides lightweight, zero-external-dependency primitives that make verification
explicit, ergonomic, and mathematically sound.

Part of the wider `tpt-zero-*` ecosystem — search **`tpt-zero`** on
[crates.io](https://crates.io/search?q=tpt-zero) to see every crate in the
family.

**Target audience**: developers using `tpt-eidos` / `tpt-telos`, safety-critical
systems engineers (DO-178C, ISO 26262), and architects building formally verified
microservices and AI systems.

## Quick start

Add the umbrella crate and pick the layers you need:

```sh
cargo add tpt-zero-formal --features "layer2,layer3"
```

```rust
use tpt_zero_formal::prelude::*;

// Fixed-size tensor math...
let v = Tensor::<f64, 3>::from_fn(|i| (i as f64) - 1.0); // [-1.0, 0.0, 1.0]
assert_eq!(v.get(0), Some(&-1.0));

// ...a standard-normal density...
let n = Normal::standard();
let _ = n.pdf(0.0);

// ...and a conjugate-prior Bayesian update — all with zero external dependencies.
let posterior = Beta::new(1.0, 1.0).unwrap().posterior(7, 3);
assert!((posterior.mean() - 8.0 / 12.0).abs() < 1e-12);
```

## Design principles

- **Zero external production dependencies.** Every crate depends only on `core`
  (and, opt-in, `alloc`) and on its sibling crates in this workspace — never on an
  outside crate.
- **`no_std` by default.** `std` is an opt-in feature per crate.
- **Panic-free by default.** Fallible operations return `Result`; violation
  checks in the design-by-contract crates are feature-gated so verification builds
  and production builds can differ deliberately, not accidentally.

## Crates

The workspace is layered so each crate depends only on the layer(s) below it. The
`out-zero-*` crates are internal, `publish = false` implementation details (a
zero-dep / `no_std` equivalent already exists, or the crate is too thin to stand
alone). The full published-vs-internal breakdown, with the closest
crates.io alternatives, lives in [`docs/crate-selection.md`](docs/crate-selection.md).

| Layer | Crates |
|---|---|
| **0 — primitives** | `safe-cast`, `assert-const`, `smt-lite`, `float`, `witness`, `phantom`, `type-level` |
| **1 — proof-carrying types** | `bounded`, `precond`, `postcond`, `ghost`, `refinement` |
| **2 — data & numerics** | `rand`, `stats`, `tensor`, `linalg`, `prob`, `sampler` |
| **3 — algorithms** | `eigen`, `grad`, `solver`, `decomp`, `monte-carlo`, `bayes`, `dist`, `markov`, `fsm`, `contract`, `invariant`, `loop-inv` |
| **4 — umbrella** | `tpt-zero-formal` |

## Building

```sh
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Every crate also builds with `--no-default-features` (pure `core`, no `alloc`),
and the whole workspace builds for bare-metal `no_std` targets
(`thumbv7em-none-eabihf`).

## MSRV

Rust 1.85 (edition 2024).

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your
option.
