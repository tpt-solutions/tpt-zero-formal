# Changelog

All notable changes to this crate are documented here. This crate adheres to
[Semantic Versioning](https://semver.org/).

## [0.1.0] - unreleased

- Initial release as part of the `tpt-zero-formal` workspace.
- Zero-cost bounds-checked integers (`BoundedInt<MIN, MAX>`) for `no_std`.
- This crate is internal to the workspace (`publish = false`) and is not released to
  crates.io on its own. The `out-zero-formal` umbrella crate re-exports its
  `no_std`/zero-dependency functionality for downstream users.

