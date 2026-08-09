# Contributing to tpt-zero-formal

Thanks for your interest in contributing! This workspace is a set of small,
`no_std`, zero-production-dependency crates that back formal-methods and
numerical work.

## Workspace layout

Crates are layered so each depends only on the layer(s) below it:

- **Layer 0** — `out-zero-*` primitives: `safe-cast`, `assert-const`, `smt-lite`,
  `float`, `witness`, `phantom`, `type-level`.
- **Layer 1** — proof-carrying types: `bounded`, `precond`, `postcond`, `ghost`,
  `refinement`.
- **Layer 2** — data + numerics: `rand`, `stats`, `tensor`, `linalg`, `prob`,
  `sampler`.
- **Layer 3** — algorithms: `eigen`, `grad`, `solver`, `decomp`, `monte-carlo`,
  `bayes`, `dist`, `markov`, `fsm`, `contract`, `invariant`, `loop-inv`.
- **Layer 4** — the `out-zero-formal` umbrella crate.

## Before opening a PR

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo build --workspace --no-default-features
cargo build --workspace --target thumbv7em-none-eabihf   # bare-metal / no_std
```

All crates build with `--no-default-features` (pure `core`) and the workspace
lints are intentionally strict (`all` + `pedantic` at `deny` for some
families). New `unsafe` code is forbidden outside an explicit, reviewed
`unsafe` block with a `// SAFETY:` comment.

## Adding a crate

Use the `xtask new-crate` helper (if present) or copy an existing leaf crate's
`Cargo.toml`. Keep it `no_std` unless there is a compelling reason; if you add a
production dependency, document why in the PR.

## Commit / release

Commits are squash-merged. Releases are published per-layer, L0 → L4, because a
published crate cannot depend on a yet-unpublished path crate. Let the maintainer
handle `cargo publish`.
