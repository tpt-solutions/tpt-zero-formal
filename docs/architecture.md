# Architecture

`tpt-zero-formal` is a workspace of small, `no_std`, zero-external-dependency
crates. The defining invariants are:

- **Zero external production dependencies.** A crate may depend only on `core`
  (and, behind an opt-in `alloc` feature, `alloc`) and on sibling crates in this
  workspace. Nothing pulls in `std`, `libm`, or a third-party crate at runtime.
- **`no_std` by default.** `std` is an opt-in feature per crate; the default
  build is pure `core`.
- **Panic-free by default.** Fallible operations return `Result`; contract
  checks are feature-gated so a verification build and a production build can
  differ deliberately.

## Layers

Crates are organized so each depends only on the layer(s) below it.

```
Layer 0  primitives
  out-zero-safe-cast      panic-free, verified numeric casts
  out-zero-assert-const   compile-time assertions in const contexts
  tpt-zero-smt-lite       boolean / pseudo-boolean constraint solver
  out-zero-float         verified sqrt / exp / ln (subnormal-safe)
  tpt-zero-witness       proof-carrying Witness<T, P> types
  out-zero-phantom       advanced PhantomData patterns
  out-zero-type-level    const-generic type-level arithmetic

Layer 1  proof-carrying types
  out-zero-bounded        BoundedInt<MIN, MAX> range-checked integers
  out-zero-precond        precondition checks (detailed errors)
  out-zero-postcond       postcondition / result verification
  tpt-zero-ghost          Ghost<Proven>/Ghost<Unproven> markers
  tpt-zero-refinement     Refined<T, Predicate> refinement types

Layer 2  data & numerics
  tpt-zero-rand           deterministic, seedable PRNGs (Xorshift, PCG)
  tpt-zero-stats          descriptive statistics (mean, variance, quantiles)
  tpt-zero-tensor         fixed-rank const-generic tensors
  tpt-zero-linalg         linear algebra (norms, products, decompositions helpers)
  tpt-zero-prob           Dist<f64> probabilistic container + helpers
  tpt-zero-sampler        sampling algorithms

Layer 3  algorithms
  tpt-zero-eigen          eigenvalues / eigenvectors (small matrices)
  tpt-zero-grad           automatic differentiation
  tpt-zero-solver         linear system solvers (Gaussian, LU, Cholesky, iterative)
  tpt-zero-decomp         LU / QR / Cholesky decompositions
  tpt-zero-monte-carlo    Monte Carlo utilities
  tpt-zero-bayes          conjugate-prior Bayesian inference
  tpt-zero-dist           distributions (Normal, Uniform, Bernoulli, Poisson)
  tpt-zero-markov         Markov chains / transition matrices
  tpt-zero-fsm            strongly-typed finite state machines
  out-zero-contract       requires!/ensures! design-by-contract macros
  tpt-zero-invariant      state-machine invariants
  out-zero-loop-inv       loop-invariant checks

Layer 4  umbrella
  tpt-zero-formal         re-exports every layer, grouped by feature
```

The `out-zero-*` crates are `publish = false` internal implementation details
(see [`crate-selection.md`](crate-selection.md)). The `tpt-zero-*` crates are
published.

## The facade

`tpt-zero-formal` re-exports each crate behind a short, namespaced module
(`tpt_zero_formal::prob`, `tpt_zero_formal::linalg`, ...) so types that share a
name across crates — `Distribution`, `Normal`, `Witness` — no longer collide.
Feature flags mirror the layers (`layer0` … `layer3`) and individual crates
(`prob`, `linalg`, ...). A small [`prelude`] re-exports the most common,
unambiguous items.
