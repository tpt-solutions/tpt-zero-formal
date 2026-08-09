# Changelog

All notable changes to this crate are documented here. This crate adheres to
[Semantic Versioning](https://semver.org/).

## [0.1.0] - unreleased

- Initial release as part of the `tpt-zero-formal` workspace.
- Verified `no_std` `sqrt`/`exp`/`ln` (subnormal-safe).
- Fixed `ln` accuracy (correct `frexp` for normals/subnormals); unified verified `sqrt`/`exp`/`ln` used across the workspace.
- Published to crates.io as part of the layered `tpt-zero-formal` release (layer order
  L0 -> L4).

