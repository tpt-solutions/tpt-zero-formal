# tpt-zero-formal — Build Checklist

Tracks the 30-crate build described in the approved plan
(`crates.io`-ready, `no_std`-first, real implementations). Work proceeds
strictly layer by layer (a layer's crates only depend on earlier layers),
since internal path-dependencies between sibling crates are allowed.

Each crate's 4 sub-items: **Scaffold** (Cargo.toml + lib.rs skeleton + README
stub) → **Implement** (real logic) → **Tests** (unit + doctests, + proptest
where noted) → **Docs** (polish README/rustdoc for docs.rs).

## Layer 0.5 — Workspace scaffolding

- [x] Root `Cargo.toml` (workspace.package, corrected shared dev-deps, lints)
- [x] `LICENSE-MIT` / `LICENSE-APACHE`
- [x] Root `README.md` (ecosystem pitch + crate table)
- [x] `.gitignore`, `rust-toolchain.toml`, `rustfmt.toml`
- [x] `crates/` directory populated with all 30 members (ongoing below)

## Layer 0 — Leaf crates (no internal deps)

### out-zero-contract
- [x] Scaffold
- [x] Implement (`requires!`/`ensures!` macros, `checked` feature)
- [x] Tests (unit + doctests)
- [x] Docs/README

### tpt-zero-invariant
- [x] Scaffold
- [x] Implement (state-machine invariant traits/macros)
- [x] Tests (unit + doctests)
- [x] Docs/README

### out-zero-safe-cast
- [x] Scaffold
- [x] Implement (panic-free verified numeric casts)
- [x] Tests (unit + doctests + proptest)
- [x] Docs/README

### tpt-zero-smt-lite
- [x] Scaffold
- [x] Implement (boolean/integer constraint helpers)
- [x] Tests (unit + doctests + proptest)
- [x] Docs/README

### out-zero-phantom
- [x] Scaffold
- [x] Implement (zero-cost marker-type/variance patterns)
- [x] Tests (unit + doctests)
- [x] Docs/README

### out-zero-type-level
- [x] Scaffold
- [x] Implement (type-level arithmetic/constraints via const generics)
- [x] Tests (unit + doctests)
- [x] Docs/README

### tpt-zero-witness
- [x] Scaffold
- [x] Implement (`Proof` marker trait + `Witness<T, P>`)
- [x] Tests (unit + doctests)
- [x] Docs/README

### out-zero-newtype
- [x] Scaffold
- [x] Implement (safe newtype wrappers, macro-based derivation)
- [x] Tests (unit + doctests)
- [x] Docs/README

### out-zero-assert-const
- [x] Scaffold
- [x] Implement (compile-time assertion utilities)
- [x] Tests (unit + doctests)
- [x] Docs/README

### tpt-zero-stats
- [x] Scaffold
- [x] Implement (mean, variance, std dev, percentiles)
- [x] Tests (unit + doctests + proptest)
- [x] Docs/README

### tpt-zero-rand
- [x] Scaffold
- [x] Implement (Xorshift, PCG seedable PRNGs)
- [x] Tests (unit + doctests + proptest)
- [x] Docs/README

### tpt-zero-tensor
- [x] Scaffold
- [x] Implement (fixed-rank tensor types)
- [x] Tests (unit + doctests + proptest)
- [x] Docs/README

**Layer 0 exit check**: `cargo build --workspace`, `cargo test --workspace`,
`cargo clippy --workspace --all-targets --all-features -- -D warnings` all clean.

## Layer 1 — depends only on Layer 0

### out-zero-bounded (needs: safe-cast, assert-const)
- [x] Scaffold
- [x] Implement (`BoundedInt<MIN, MAX>`)
- [x] Tests (unit + doctests + proptest)
- [x] Docs/README

### out-zero-precond (needs: contract)
- [x] Scaffold
- [x] Implement
- [x] Tests (unit + doctests)
- [x] Docs/README

### out-zero-postcond (needs: contract)
- [x] Scaffold
- [x] Implement
- [x] Tests (unit + doctests)
- [x] Docs/README

### tpt-zero-fsm (needs: invariant)
- [x] Scaffold
- [x] Implement (zero-allocation strongly-typed FSM builder)
- [x] Tests (unit + doctests + tests/ integration)
- [x] Docs/README

### out-zero-loop-inv (needs: invariant)
- [x] Scaffold
- [x] Implement (loop invariant checking)
- [x] Tests (unit + doctests)
- [x] Docs/README

### tpt-zero-ghost (needs: phantom)
- [x] Scaffold
- [x] Implement (`Ghost<Proven>` / `Ghost<Unproven>` markers)
- [x] Tests (unit + doctests)
- [x] Docs/README

### out-zero-refinement (needs: witness)
- [x] Scaffold
- [x] Implement (`Refined<T, Predicate>`)
- [x] Tests (unit + doctests)
- [x] Docs/README

### tpt-zero-prob (needs: stats)
- [x] Scaffold
- [x] Implement (`Distribution` trait, generic `Dist<f64>` sample container)
- [x] Tests (unit + doctests + proptest)
- [x] Docs/README

### tpt-zero-linalg (needs: tensor)
- [x] Scaffold
- [x] Implement (dot/cross product, norms)
- [x] Tests (unit + doctests + proptest)
- [x] Docs/README

**Layer 1 exit check**: `cargo build --workspace`, `cargo test --workspace`,
`cargo clippy --workspace --all-targets --all-features -- -D warnings` all clean.

## Layer 2 — depends on Layer ≤1

### tpt-zero-dist (needs: rand, stats, prob)
- [x] Scaffold
- [x] Implement (Normal, Uniform, Bernoulli, Poisson)
- [x] Tests (unit + doctests + proptest)
- [x] Docs/README

### tpt-zero-decomp (needs: linalg)
- [x] Scaffold
- [x] Implement (LU, QR, Cholesky)
- [x] Tests (unit + doctests + proptest)
- [x] Docs/README

### tpt-zero-grad (needs: tensor, linalg)
- [x] Scaffold
- [x] Implement (automatic differentiation primitives)
- [x] Tests (unit + doctests + proptest)
- [x] Docs/README

### tpt-zero-markov (needs: linalg, rand, stats)
- [x] Scaffold
- [x] Implement (Markov chains, transition matrix ops)
- [x] Tests (unit + doctests + proptest)
- [x] Docs/README

### tpt-zero-sampler (needs: rand, prob)
- [x] Scaffold
- [x] Implement (sampling algorithms for probabilistic inference)
- [x] Tests (unit + doctests + proptest)
- [x] Docs/README

**Layer 2 exit check**: `cargo build --workspace`, `cargo test --workspace`,
`cargo clippy --workspace --all-targets --all-features -- -D warnings` all clean.

## Layer 3 — depends on Layer ≤2

### tpt-zero-solver (needs: linalg, decomp)
- [x] Scaffold
- [x] Implement (Gaussian elimination, iterative methods)
- [x] Tests (unit + doctests + proptest)
- [x] Docs/README

### tpt-zero-bayes (needs: dist, prob, stats)
- [x] Scaffold
- [x] Implement (Bayesian inference primitives, conjugate priors)
- [x] Tests (unit + doctests + proptest + tests/ integration)
- [x] Docs/README

### tpt-zero-monte-carlo (needs: rand, sampler, stats)
- [x] Scaffold
- [x] Implement (Monte Carlo simulation, variance reduction)
- [x] Tests (unit + doctests + proptest)
- [x] Docs/README

**Layer 3 exit check**: `cargo build --workspace`, `cargo test --workspace`,
`cargo clippy --workspace --all-targets --all-features -- -D warnings` all clean.

## Layer 4 — depends on Layer ≤3

### tpt-zero-eigen (needs: linalg, decomp, solver)
- [x] Scaffold
- [x] Implement (eigenvalue/eigenvector computation for small matrices)
- [x] Tests (unit + doctests + proptest + tests/ integration)
- [x] Docs/README

**Layer 4 exit check / workspace done**: `cargo build --workspace`,
`cargo test --workspace`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
`cargo doc --workspace --no-deps` all clean.

## Deferred / stretch (not blocking crate completion)

- [ ] Criterion benchmarks (candidates first: `rand`, `linalg`, `tensor`)
- [ ] `cargo-deny` config (`deny.toml`)
- [ ] `CONTRIBUTING.md`
- [ ] `git init` + GitHub Actions CI (build/test/clippy/fmt across stable + no_std targets)
- [ ] crates.io publish — **must publish in layer order L0 → L4** (path deps
      need their target versions live on crates.io before a dependent crate
      can publish)
