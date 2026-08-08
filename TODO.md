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
- [x] Tests (unit + doctests) — proptest listed but unused
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
`cargo clippy --workspace --all-targets --all-features -- -D warnings` all clean;
and `cargo build --workspace --no-default-features` plus
`cargo build --workspace --all-features` succeed.

## Layer 1 — depends only on Layer 0

### out-zero-bounded (needs: safe-cast, assert-const)
- [x] Scaffold
- [x] Implement (`BoundedInt<MIN, MAX>`)
- [x] Tests (unit + doctests + proptest)
- [x] Docs/README

### out-zero-precond (no internal deps; uses `out-zero-contract` macros by re-export, but does not list it as a `[dependencies]`)
- [x] Scaffold
- [x] Implement
- [x] Tests (unit + doctests)
- [x] Docs/README

### out-zero-postcond (no internal deps; uses `out-zero-contract` macros by re-export, but does not list it as a `[dependencies]`)
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

### tpt-zero-ghost (needs: witness; does NOT depend on `out-zero-phantom`)
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
- [x] Tests (unit + doctests) — proptest listed but unused
- [x] Docs/README

### tpt-zero-linalg (needs: tensor)
- [x] Scaffold
- [x] Implement (dot/cross product, norms)
- [x] Tests (unit + doctests + proptest)
- [x] Docs/README

**Layer 1 exit check**: `cargo build --workspace`, `cargo test --workspace`,
`cargo clippy --workspace --all-targets --all-features -- -D warnings` all clean;
and `cargo build --workspace --no-default-features` plus
`cargo build --workspace --all-features` succeed.

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
- [x] Tests (unit + doctests) — proptest listed but unused
- [x] Docs/README

**Layer 2 exit check**: `cargo build --workspace`, `cargo test --workspace`,
`cargo clippy --workspace --all-targets --all-features -- -D warnings` all clean;
and `cargo build --workspace --no-default-features` plus
`cargo build --workspace --all-features` succeed.

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
`cargo clippy --workspace --all-targets --all-features -- -D warnings` all clean;
and `cargo build --workspace --no-default-features` plus
`cargo build --workspace --all-features` succeed.

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
- [x] `git init` (repository already initialized locally)
- [ ] GitHub Actions CI (build/test/clippy/fmt across stable + no_std targets)
- [ ] crates.io publish — **must publish in layer order L0 → L4** (path deps
      need their target versions live on crates.io before a dependent crate
      can publish)

---

# Review Action Items (added 2026-08-08)

Results of a platform-wide review (soundness, numerics, packaging, adoption).
Severity: 🔴 = ship-blocker, 🟠 = should-fix, 🟡 = nice-to-have.

## P0 — Ship-blockers (do before any publish)

### Soundness of proof-carrying types
- [x] **witness**: seal the `Proof` forge — delete the no-obligation
      `Witness::new`; keep `from_proof` (consumes a proof value). Update doctests.
- [x] **witness**: hand-write `Clone`/`Copy`/`Debug` so they do not add a
      `P: Clone`/`P: Copy` bound (currently `Witness<u32, NoClone>` is not `Clone`).
- [x] **ghost**: make `Ghost::new` construct `Unproven` only (move into
      `impl<T> Ghost<T, Unproven>`); fix doctests that build `GhostProven` via `new`.
- [x] **ghost**: `map` must return `Ghost<U, Unproven>` (it currently launders
      `Proven` onto an arbitrary new value).
- [x] **ghost**: gate `prove` behind a named `witness` feature; fix `Cargo.toml`
      (`default = ["witness"]`, `witness = ["dep:tpt-zero-witness"]`) so
      `--no-default-features` builds; fix the `std`-gated test (use `witness`).
- [x] **refinement**: make `into_witness` sound — it currently mints an
      *arbitrary* `Proof`. Return `Witness<T, RefinedProof<P>>` (local proof
      type, only obtainable from a validated `Refined`) instead of a caller-chosen `Pr`.
- [x] **bounded**: force the `MIN <= MAX` compile check to actually run by
      evaluating `ASSERT_RANGE` from every constructor (associated consts are
      lazily monomorphized, so today `BoundedInt<100,0>` compiles and `clamp` lies).
- [x] **bounded**: `new_unchecked` is a safe fn that breaks the invariant in
      release — rename to `new_clamped` and clamp in all profiles (or delete).
- [x] **bounded**: `saturating_add`/`saturating_sub` narrow `i128`→`i64` *before*
      clamping (so `100.saturating_add(i64::MAX)` returns `0`). Clamp in `i128`.

### Contract `checked` feature is broken across crates
- [x] **contract**: `requires!`/`ensures!` put `#[cfg(feature="checked")]` *inside*
      the exported macro, so it reads the **caller's** features and does nothing.
      Emit a `#[doc(hidden)]` helper macro at definition site instead; add
      `[lints.rust] unexpected_cfgs` check-cfg so the class of bug is caught.

### Hand-rolled `core_math` is wrong (6×`sqrt`, 5×`exp`, 3×`ln`)
- [x] Create `out-zero-float` with one verified `sqrt`/`exp`/`ln` (subnormal-safe,
      relative tolerance, fixed iteration cap). Replace the 14 copies:
      stats, linalg, eigen, grad, monte-carlo (sqrt); dist, bayes, prob, sampler (exp);
      bayes, dist, grad (ln).
- [x] Fix downstream breakage: `qr`/`cholesky` non-orthonormal Q; `grad::ln(1e-13)`
      hang; `Poisson::sample(λ≳709)` infinite loop; `power_iteration`/`eigenvalues_2x2`
      cancellation; `norm_l2`/`normalize` for tiny/huge vectors.

### `smt-lite` correctness
- [x] `1usize << n` at `num_vars == 64` panics in debug / returns UNSAT in release
      (`>=` should be `>=` guard + `checked_shl`).
- [x] "Linear **integer** constraints" are actually 0/1 pseudo-boolean — rename/re-scope
      honestly (`tpt-zero-pbsat`) or implement real integer variables.
- [x] Rewrite the non-compiling README example (fabricated `constraint_set`,
      `boolean::{and,or}`, `linear_leq` API).

### `--no-default-features` / `--all-features` build fixes
- [x] **ghost**: fixed above (named `witness` feature).
- [x] **prob**: `--all-features` doctest fails (`alloc` not in scope) — add
      `# extern crate alloc;` to the `from_vec` doctest.

### Macro hygiene — double evaluation
- [x] **invariant** + **loop-inv**: `check_invariant!`/loop-inv macros evaluate
       `$value` twice in debug, once in release. Bind once into a `let` (affects
       `read_sensor()`-style side effects and debug≠release behaviour).

## P1 — Before announcing

- [x] **publish metadata**: add `rust-version`, `homepage`, `include` (license
      files), `[package.metadata.docs.rs]` + `doc(cfg)` to all crates; copy
      `LICENSE-MIT`/`LICENSE-APACHE` into each crate dir.
- [x] **ghost README**: remove false "builds with `--no-default-features`" / "`std`
      enables `Ghost::prove`" claims.
- [x] **tensor README**: delete lines 1–24 (raw `//!` doc-comment source pasted in).
- [x] **dead features**: drop inert `alloc`/`std` from the crates whose `src/`
       never reads them (use `core::error::Error` instead of a `std` gate). The 4
       crates with genuine `alloc` usage (`newtype`, `dist`, `markov`, `prob`) keep
       their feature; the 3 `std`-gated `Error` impls (`postcond`, `precond`,
       `refinement`) now use `core::error::Error`.
- [x] Add `tpt-zero-formal` **facade crate** (feature groups + prelude) and claim
      the name on crates.io; namespace away `Distribution`/`Normal`/`Witness` collisions.
- [x] Rewrite root README around one runnable end-to-end example; move the two
      "Internal-only / closest alternative" tables to `docs/crate-selection.md`.
- [x] `#![doc = include_str!("../README.md")]` in all 30 `lib.rs` so README examples
      become doctests; fix the broken ones (only `tpt-zero-bayes` needed a fix:
      `Beta::new` returns `Option` and the posterior mean is `8/12`, not `0.8`).
- [ ] Add `cargo-generate` templates + `examples/` (start: contracts_basics,
       do178c_altitude_monitor, baremetal_thumbv7, telos_transpile_target,
       type_state_protocol, refinement_vs_witness_vs_ghost, kalman_filter_nostd,
       migrating_from_rand).
        - **examples/**: all 10 added — `migrating_from_rand`, `bayesian_update`,
          `tensor_linalg_solve`, `contracts_basics`, `do178c_altitude_monitor`,
          `baremetal_thumbv7`, `telos_transpile_target`, `type_state_protocol`,
          `refinement_vs_witness_vs_ghost`, `kalman_filter_nostd` (all build; the
          4 std binaries run, the 3 no_std libs build + cross-compile to
          `thumbv7em-none-eabihf`, `kalman_filter_nostd` has a convergence test).
        - **cargo-generate templates**: not started.
- [x] Add `docs/choosing.md` (problem→crate index), `docs/architecture.md`
      (layer diagram from `cargo metadata`), `docs/comparison.md` (vs contracts/nalgebra/rand).
- [x] **CI**: `fmt`, `clippy -D`, `test`, `--no-default-features`, `--all-features`,
      bare-metal (`thumbv7em-none-eabihf`), `doc`; split proptest budget so `cargo test`
      isn't 10 min. (Direct cargo steps; `xtask check-readmes`/`check-consistency`
      are tracked under the xtask item below.)
- [x] **xtask**: `new-crate`, `check-consistency`, `check-readmes`, `check-nostd`,
       `publish-order`, `publish`, `gen-graph`, `gen-type-level` (the `gen-type-level`
       generator regenerates `out-zero-type-level/src/generated.rs` and now emits
       `AssertLe` impls only for `A <= B`, fixing the `compile_fail` doctest).
- [x] Reconsider publish decisions for `out-zero-contract`, `-precond`, `-postcond`,
       `-refinement`: decision is to **publish** all four (they are zero-dep or
       low-dep and load-bearing for the `tpt-telos` transpile story; `contracts`
       is not zero-dep, but these four are). Actual `cargo publish` is deferred to
       the layered crates.io flow under "Deferred / stretch" below.
- [x] Remove dead `criterion` dev-dep and the 4 unused `proptest` dev-deps
      (`safe-cast`, `fsm`, `prob`, `sampler`); fix `rustfmt.toml` (`imports_granularity`
      is nightly-only, silently ignored on stable). (`libt.rmeta` not present.)
- [x] Add `CONTRIBUTING.md`, `SECURITY.md`, `CHANGELOG.md`, `deny.toml`,
      `CODE_OF_CONDUCT.md`.
- [ ] Update `Cargo.lock` (currently gitignored).

## P2 — Differentiation (innovation)

- [ ] Requirement traceability: `requires!(REQ="SRS-ALT-014", cond)` + `xtask certify`
      → traceability matrix.
- [ ] Panic-freedom proof (`panic_handler` fail build) per crate/feature set.
- [ ] `xtask certify` certification artifact pack (deps/unsafe/MSRV/coverage/contract inventory).
- [ ] Kani / Creusot / Prusti backend for `requires!`/`ensures!`.
- [ ] MC/DC-aware contract macros (rustc `-Z coverage-options=mcdc`).
- [ ] Contract-derived test generation (`requires!` → proptest at boundaries).
- [ ] SMT-LIB / Why3 spec export from `smt-lite` + contracts.

## P3 — Documentation accuracy (TODO.md corrections)

- [x] Mark `proptest` claims false where unused (`safe-cast`, `prob`, `sampler`);
      mark `precond`/`postcond` "(needs: contract)" false (no `[dependencies]`);
      mark `ghost` "(needs: phantom)" false (depends on `witness`).
- [x] Split the stale `git init + CI` item (git done; CI missing).
- [x] Update layer exit checks: feature matrix (`--no-default-features`,
      `--all-features`) not just default features.
