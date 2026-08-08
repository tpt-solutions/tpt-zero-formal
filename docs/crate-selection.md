# Crate selection guide

This workspace ships two kinds of crates, marked at a glance by their name
prefix:

- **`tpt-zero-*`** — published, general-purpose crates you can depend on from
  crates.io.
- **`out-zero-*`** — internal, `publish = false` implementation details. They are
  kept internal when a zero-dependency / `no_std` equivalent already exists on
  crates.io, or when the crate is too thin to stand alone. Prefer depending on the
  published alternative directly.

## Published vs internal

### Design-by-Contract & Verification

**Published:**

| Crate | Purpose |
|---|---|
| `tpt-zero-invariant` | Traits/macros asserting state-machine invariants |
| `tpt-zero-fsm` | Zero-allocation, strongly-typed Finite State Machine builder |
| `tpt-zero-ghost` | Ghost state markers / phantom types for verification state |
| `tpt-zero-smt-lite` | Basic SMT-style boolean / pseudo-boolean constraint helpers |

**Internal-only** (not published — a zero-dep / `no_std` equivalent already
exists, or the crate is too thin to stand alone):

| Crate | Purpose | Closest existing alternative |
|---|---|---|
| `out-zero-contract` | `requires!` / `ensures!` macros for design-by-contract programming | `contracts` |
| `out-zero-bounded` | Zero-cost wrapper types guaranteeing numeric bounds, e.g. `BoundedInt<0, 100>` | `bounded-integer` |
| `out-zero-safe-cast` | Panic-free, explicitly verified numeric casting traits | `az` |
| `out-zero-precond` | Precondition checking with detailed error reporting | `contracts` |
| `out-zero-postcond` | Postcondition validation and result verification | `contracts` |
| `out-zero-loop-inv` | Loop invariant checking for iterative algorithms | (thin wrapper over `debug_assert!`) |

### Probabilistic & Statistical Computing

**Published** (all 8 — no zero-dep / `no_std` equivalent stack exists on
crates.io):

| Crate | Purpose |
|---|---|
| `tpt-zero-prob` | Lightweight probabilistic types (`Dist<f64>`) and statistical helpers |
| `tpt-zero-stats` | Core statistical functions (mean, variance, std dev, percentiles) |
| `tpt-zero-rand` | Deterministic, seedable PRNGs (Xorshift, PCG) |
| `tpt-zero-dist` | Common probability distributions (Normal, Uniform, Bernoulli, Poisson) |
| `tpt-zero-sampler` | Efficient sampling algorithms for probabilistic inference |
| `tpt-zero-bayes` | Bayesian inference primitives and conjugate priors |
| `tpt-zero-monte-carlo` | Monte Carlo simulation utilities with variance reduction |
| `tpt-zero-markov` | Markov chain representations and transition matrix operations |

### Linear Algebra & Tensor Operations

**Published** (all 6 — `nalgebra` / `ndarray` / `num-dual` cover the same
concepts but none are zero-dependency):

| Crate | Purpose |
|---|---|
| `tpt-zero-tensor` | Fixed-rank tensor types for multi-dimensional data |
| `tpt-zero-linalg` | Core linear algebra operations (dot/cross product, norms) |
| `tpt-zero-decomp` | Matrix decompositions (LU, QR, Cholesky) |
| `tpt-zero-solver` | Linear system solvers (Gaussian elimination, iterative methods) |
| `tpt-zero-eigen` | Eigenvalue / eigenvector computation for small matrices |
| `tpt-zero-grad` | Automatic differentiation primitives for gradient computation |

### Type-Level & Compile-Time Verification

**Published:**

| Crate | Purpose |
|---|---|
| `tpt-zero-witness` | Witness types that carry proofs of properties |

**Internal-only** (not published — a zero-dep / `no_std` equivalent already
exists, or the crate is too thin to stand alone):

| Crate | Purpose | Closest existing alternative |
|---|---|---|
| `out-zero-type-level` | Type-level arithmetic / constraints using const generics | `typenum` |
| `out-zero-phantom` | Advanced phantom-type patterns for zero-cost abstractions | (thin wrapper over `PhantomData`) |
| `out-zero-refinement` | Refinement types encoding predicates in the type system | `refinement` |
| `out-zero-newtype` | Safe newtype wrappers with automatic derivation | `derive_more` / `nutype` |
| `out-zero-assert-const` | Compile-time assertion utilities for const contexts | `static_assertions` |

## Choosing a crate

- Need **probabilistic types / distributions**? Start with `tpt-zero-prob` and
  `tpt-zero-dist`; add `tpt-zero-bayes` for inference and `tpt-zero-sampler` /
  `tpt-zero-monte-carlo` for sampling.
- Need **linear algebra**? `tpt-zero-tensor` + `tpt-zero-linalg`, then
  `tpt-zero-decomp` / `tpt-zero-solver` / `tpt-zero-eigen` as needed.
- Need **verification scaffolding**? `tpt-zero-ghost` + `tpt-zero-witness` for
  proof-carrying state, `tpt-zero-invariant` / `tpt-zero-fsm` for state machines,
  and `out-zero-contract` / `out-zero-bounded` / `out-zero-safe-cast` for
  contract and bound checks (kept internal by design).
- Want everything namespaced behind one dependency? Use the `tpt-zero-formal`
  umbrella crate, which re-exports every layer and groups them by feature.
