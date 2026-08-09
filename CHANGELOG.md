# Changelog

All notable changes to the crates in this workspace are documented here. This
project adheres to [Semantic Versioning](https://semver.org/).

## [0.1.0] - unreleased

### Added
- Initial public workspace: `no_std`, zero-production-dependency formal-methods
  and numerical toolkit.
- Layer 0 primitives: `safe-cast`, `assert-const`, `smt-lite`, `float`,
  `witness`, `phantom`, `type-level`.
- Proof-carrying types: `bounded`, `precond`, `postcond`, `ghost`, `refinement`.
- Numerics & data: `rand`, `stats`, `tensor`, `linalg`, `prob`, `sampler`.
- Algorithms: `eigen`, `grad`, `solver`, `decomp`, `monte-carlo`, `bayes`,
  `dist`, `markov`, `fsm`, `contract`, `invariant`, `loop-inv`.
- `out-zero-formal` umbrella crate with namespaced re-exports and layered
  feature flags.

### Fixed
- `tpt-zero-float` `ln` accuracy (correct `frexp` for normals/subnormals).
- `smt-lite` `1usize << n` overflow at 64 variables; re-scoped "linear integer"
  constraints as pseudo-boolean (`pb_eq`/`pb_leq`).
- `dist::Poisson::sample` infinite loop for large `lambda` (log-space Knuth).
- Replaced 14 hand-rolled `sqrt`/`exp`/`ln` copies with the verified
  `tpt-zero-float` implementation.
