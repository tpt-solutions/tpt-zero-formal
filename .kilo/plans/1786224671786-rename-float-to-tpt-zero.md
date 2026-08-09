# Rename `out-zero-float` → `tpt-zero-float`

## Context

`out-zero-float` is the only `out-zero-*` crate with `publish = false` omitted,
because 11 released `tpt-zero-*` crates depend on it for verified `no_std`
`sqrt`/`exp`/`ln`. Per the project's own policy ("don't publish a crate unless
it's more useful than what exists"), a crate that *is* published should carry the
`tpt-zero-*` prefix like every other published crate. The user confirmed: rename
it to the `tpt-zero` prefix and move the directory accordingly.

New identities:
- crate name: `tpt-zero-float`
- lib (Rust) name: `tpt_zero_float` (matches published convention)
- directory: `crates/tpt-zero-float`

The umbrella `float` cargo feature name is **kept** (only the dependency it
enables is renamed); the public `tpt_zero_formal::float` re-export path is
unchanged for API stability.

## Affected boundaries

- Workspace uses `members = ["crates/*", ...]` (glob), so the directory rename
  automatically keeps the crate in the workspace — no root `Cargo.toml` member
  edit needed.
- Dependent crates (path deps): `tpt-zero-bayes`, `tpt-zero-dist`, `tpt-zero-eigen`,
  `tpt-zero-grad`, `tpt-zero-linalg`, `tpt-zero-monte-carlo`, `tpt-zero-prob`,
  `tpt-zero-sampler`, `tpt-zero-stats`, `tpt-zero-formal` (facade, optional dep).

## Steps

1. **Move the directory**
   - `git mv crates/out-zero-float crates/tpt-zero-float` (preserves its existing
     `CHANGELOG.md`/`README.md`/`src`).

2. **`crates/tpt-zero-float/Cargo.toml`**
   - `name = "out-zero-float"` → `name = "tpt-zero-float"`
   - `documentation = "https://docs.rs/out-zero-float"` → `https://docs.rs/tpt-zero-float`
   - `[lib] name = "out_zero_float"` → `name = "tpt_zero_float"`

3. **`crates/tpt-zero-float/README.md`**
   - `# out-zero-float` → `# tpt-zero-float`
   - badge URLs `out-zero-float` → `tpt-zero-float` (3: crates.io, docs.rs, license)
   - `use out_zero_float::{sqrt, exp, ln};` → `use tpt_zero_float::{...}`
   - doc-link URLs `/out_zero_float/` → `/tpt_zero_float/` (3)
   - `cargo add out-zero-float` → `cargo add tpt-zero-float`

4. **`crates/tpt-zero-float/src/lib.rs`**
   - doc-comment `use out_zero_float::...` → `use tpt_zero_float::...` (3 spots:
     ~lines 41, 127, 233). Verify no other `out_zero_float` intra-doc links remain.

5. **Facade `crates/tpt-zero-formal/Cargo.toml`**
   - `float = ["dep:out-zero-float"]` → `float = ["dep:tpt-zero-float"]`
   - `out-zero-float = { path = "../out-zero-float", version = "0.1.0", optional = true }`
     → `tpt-zero-float = { path = "../tpt-zero-float", version = "0.1.0", optional = true }`

6. **Facade `crates/tpt-zero-formal/src/lib.rs`**
   - `pub use out_zero_float as float;` → `pub use tpt_zero_float as float;`
   - `pub use out_zero_float::{exp, ln, sqrt};` → `pub use tpt_zero_float::{exp, ln, sqrt};`

7. **9 dependent `Cargo.toml` path deps** (facade handled in step 5)
   - In each, replace
     `out-zero-float = { path = "../out-zero-float", version = "0.1.0" }`
     → `tpt-zero-float = { path = "../tpt-zero-float", version = "0.1.0" }`:
     `tpt-zero-bayes`, `tpt-zero-dist`, `tpt-zero-eigen`, `tpt-zero-grad`,
     `tpt-zero-linalg`, `tpt-zero-monte-carlo`, `tpt-zero-prob`,
     `tpt-zero-sampler`, `tpt-zero-stats`.

8. **Rust `out_zero_float` → `tpt_zero_float` in dependent sources** (calls +
   `[`out_zero_float`]` / `[`out_zero_float::...`]` intra-doc links). Rename
   every occurrence in:
   - `tpt-zero-bayes/src/math.rs`
   - `tpt-zero-dist/src/math.rs`
   - `tpt-zero-eigen/src/lib.rs`
   - `tpt-zero-grad/src/lib.rs`
   - `tpt-zero-linalg/src/lib.rs`
   - `tpt-zero-monte-carlo/src/lib.rs`
   - `tpt-zero-prob/src/distributions.rs`
   - `tpt-zero-sampler/src/lib.rs`
   - `tpt-zero-stats/src/lib.rs`

9. **`crates/tpt-zero-stats/README.md`**
   - `use out_zero_float::sqrt;` → `use tpt_zero_float::sqrt;`

10. **Regenerate / update non-source references**
    - `Cargo.lock`: run `cargo generate-lockfile` (don't hand-edit).
    - `cert/certification_pack.json` (+ `cert/traceability_matrix.md`): re-run
      `cargo xtask certify` (or update `"name": "out-zero-float"` → `"tpt-zero-float"`).
    - `docs/architecture.md` (line ~24): `out-zero-float` → `tpt-zero-float`.
    - `docs/crate-selection.md`: `out-zero-float` → `tpt-zero-float` (lines ~12, 87);
      drop the "single exception … published" framing — `tpt-zero-float` is now a
      normal published `tpt-zero-*` crate. Keep/relabel the "Math primitives"
      published section.
    - `README.md` (root, line ~62): remove the "single exception is out-zero-float"
      special case; state internal `out-zero-*` crates are `publish = false` and
      `tpt-zero-float` is a regular published crate.
    - `CHANGELOG.md` (root, lines ~21, 26): `out-zero-float` → `tpt-zero-float`.
    - `TODO.md` (line ~284): `out-zero-float` → `tpt-zero-float`.
    - `template/cargo-generate.toml` (line ~10 prompt): `out-zero-float` → `tpt-zero-float`.

## Risk / edge cases

- **Intra-doc links**: every `[`out_zero_float`]` / `[`out_zero_float::sqrt`]`
  must become `[`tpt_zero_float`]`; a miss breaks `cargo doc` under
  `-D warnings`. Step 8 covers all `.rs` hits from the reference scan.
- **`float` feature**: keep the name; only repoint to `tpt-zero-float` (step 5)
  so the umbrella public API is unchanged.
- **Crates.io publish order**: `tpt-zero-float` must be published before its
  dependents (already true today for `out-zero-float`); `cargo xtask publish-order`
  should still topo-resolve.

## Validation

- `cargo build --workspace`
- `cargo test --workspace`
- `cargo xtask check-readmes` and `cargo xtask check-consistency`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo doc --workspace --no-deps` (confirm no broken links to the renamed crate)
- `cargo xtask publish-order` (confirm `tpt-zero-float` precedes dependents)
