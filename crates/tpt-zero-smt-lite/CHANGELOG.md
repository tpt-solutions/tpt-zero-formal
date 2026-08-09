# Changelog

All notable changes to this crate are documented here. This crate adheres to
[Semantic Versioning](https://semver.org/).

## [0.1.0] - unreleased

- Initial release as part of the `tpt-zero-formal` workspace.
- SMT-style boolean and pseudo-boolean constraint helpers (`boolean`, `pb_eq`/`pb_leq`) for `no_std`.
- Fixed `1usize << n` overflow at 64 variables; re-scoped "linear integer" constraints as pseudo-boolean (`pb_eq`/`pb_leq`).
- Published to crates.io as part of the layered `tpt-zero-formal` release (layer order
  L0 -> L4).

