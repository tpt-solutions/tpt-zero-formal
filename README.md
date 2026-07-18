# tpt-zero-formal

Runtime building blocks for formal verification, design-by-contract, and
probabilistic computing in Rust — the bridge between high-assurance TPT
languages (`tpt-telos`, `tpt-eidos`) and safe, mathematically sound Rust
execution, and the foundational math for `tpt-augur`.

Rust's borrow checker prevents memory-safety issues, but not logical ones:
dividing by zero, state-machine violations, out-of-bounds indices. And
probabilistic/AI systems need rigorous, deterministic, sound math primitives
that standard crates don't provide in a `no_std` context. `tpt-zero-formal`
provides lightweight, zero-external-dependency primitives that make
verification explicit, ergonomic, and mathematically sound.

Part of the wider `tpt-zero-*` ecosystem (`tpt-zero-bytes`, `tpt-zero-text`,
`tpt-zero-formal`) — search **`tpt-zero`** on [crates.io](https://crates.io/search?q=tpt-zero)
to see every crate in the family.

**Target audience**: developers using `tpt-eidos`/`tpt-telos`, safety-critical
systems engineers (DO-178C, ISO 26262), and architects building formally
verified microservices and AI systems.

## Design principles

- **Zero external production dependencies.** Every crate below depends only
  on `core` (and, opt-in, `alloc`) and on its sibling crates in this
  workspace — never on an outside crate.
- **`no_std` by default.** `std` is an opt-in feature per crate.
- **Panic-free by default.** Fallible operations return `Result`; violation
  checks in the design-by-contract crates are feature-gated so verification
  builds and production builds can differ deliberately, not accidentally.

## Crates

Every crate is fully implemented either way — the split below is about what ships to
crates.io under the `tpt-zero-` name versus what stays an internal-only,
`publish = false` implementation detail (prefixed `out-zero-` instead, precisely so
it's obvious at a glance in `crates/` which is which). A crate is published only if
it's a real, well-tested implementation **and** no existing crate already covers the
same concept *and* is zero-dependency *and* `no_std` — otherwise it's better served by
depending on that existing crate directly.

### Design-by-Contract & Verification

**Published:**

| Crate | Purpose |
|---|---|
| [`tpt-zero-invariant`](crates/tpt-zero-invariant) | Traits/macros asserting state-machine invariants |
| [`tpt-zero-fsm`](crates/tpt-zero-fsm) | Zero-allocation, strongly-typed Finite State Machine builder |
| [`tpt-zero-ghost`](crates/tpt-zero-ghost) | Ghost state markers / phantom types for verification state |
| [`tpt-zero-smt-lite`](crates/tpt-zero-smt-lite) | Basic SMT-style boolean/integer constraint helpers |

**Internal-only** (not published — a zero-dep/`no_std` equivalent already exists, or the crate is too thin to stand alone):

| Crate | Purpose | Closest existing alternative |
|---|---|---|
| [`out-zero-contract`](crates/out-zero-contract) | `requires!` / `ensures!` macros for design-by-contract programming | `contracts` |
| [`out-zero-bounded`](crates/out-zero-bounded) | Zero-cost wrapper types guaranteeing numeric bounds, e.g. `BoundedInt<0, 100>` | `bounded-integer` |
| [`out-zero-safe-cast`](crates/out-zero-safe-cast) | Panic-free, explicitly verified numeric casting traits | `az` |
| [`out-zero-precond`](crates/out-zero-precond) | Precondition checking with detailed error reporting | `contracts` |
| [`out-zero-postcond`](crates/out-zero-postcond) | Postcondition validation and result verification | `contracts` |
| [`out-zero-loop-inv`](crates/out-zero-loop-inv) | Loop invariant checking for iterative algorithms | (thin wrapper over `debug_assert!`) |

### Probabilistic & Statistical Computing

**Published** (all 8 — no zero-dep/`no_std` equivalent stack exists on crates.io):

| Crate | Purpose |
|---|---|
| [`tpt-zero-prob`](crates/tpt-zero-prob) | Lightweight probabilistic types (`Dist<f64>`) and statistical helpers |
| [`tpt-zero-stats`](crates/tpt-zero-stats) | Core statistical functions (mean, variance, std dev, percentiles) |
| [`tpt-zero-rand`](crates/tpt-zero-rand) | Deterministic, seedable PRNGs (Xorshift, PCG) |
| [`tpt-zero-dist`](crates/tpt-zero-dist) | Common probability distributions (Normal, Uniform, Bernoulli, Poisson) |
| [`tpt-zero-sampler`](crates/tpt-zero-sampler) | Efficient sampling algorithms for probabilistic inference |
| [`tpt-zero-bayes`](crates/tpt-zero-bayes) | Bayesian inference primitives and conjugate priors |
| [`tpt-zero-monte-carlo`](crates/tpt-zero-monte-carlo) | Monte Carlo simulation utilities with variance reduction |
| [`tpt-zero-markov`](crates/tpt-zero-markov) | Markov chain representations and transition matrix operations |

### Linear Algebra & Tensor Operations

**Published** (all 6 — `nalgebra`/`ndarray`/`num-dual` cover the same concepts but none are zero-dependency):

| Crate | Purpose |
|---|---|
| [`tpt-zero-tensor`](crates/tpt-zero-tensor) | Fixed-rank tensor types for multi-dimensional data |
| [`tpt-zero-linalg`](crates/tpt-zero-linalg) | Core linear algebra operations (dot/cross product, norms) |
| [`tpt-zero-decomp`](crates/tpt-zero-decomp) | Matrix decompositions (LU, QR, Cholesky) |
| [`tpt-zero-solver`](crates/tpt-zero-solver) | Linear system solvers (Gaussian elimination, iterative methods) |
| [`tpt-zero-eigen`](crates/tpt-zero-eigen) | Eigenvalue/eigenvector computation for small matrices |
| [`tpt-zero-grad`](crates/tpt-zero-grad) | Automatic differentiation primitives for gradient computation |

### Type-Level & Compile-Time Verification

**Published:**

| Crate | Purpose |
|---|---|
| [`tpt-zero-witness`](crates/tpt-zero-witness) | Witness types that carry proofs of properties |

**Internal-only** (not published — a zero-dep/`no_std` equivalent already exists, or the crate is too thin to stand alone):

| Crate | Purpose | Closest existing alternative |
|---|---|---|
| [`out-zero-type-level`](crates/out-zero-type-level) | Type-level arithmetic/constraints using const generics | `typenum` |
| [`out-zero-phantom`](crates/out-zero-phantom) | Advanced phantom-type patterns for zero-cost abstractions | (thin wrapper over `PhantomData`) |
| [`out-zero-refinement`](crates/out-zero-refinement) | Refinement types encoding predicates in the type system | `refinement` |
| [`out-zero-newtype`](crates/out-zero-newtype) | Safe newtype wrappers with automatic derivation | `derive_more` / `nutype` |
| [`out-zero-assert-const`](crates/out-zero-assert-const) | Compile-time assertion utilities for const contexts | `static_assertions` |

## Building

```sh
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Every crate also builds with `--no-default-features` (pure `core`, no `alloc`).

## MSRV

Rust 1.85 (edition 2024).

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at
your option.
