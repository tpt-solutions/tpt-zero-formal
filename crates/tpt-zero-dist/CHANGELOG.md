# Changelog

All notable changes to this crate are documented here. This crate adheres to
[Semantic Versioning](https://semver.org/).

## [0.1.0] - unreleased

- Initial release as part of the `tpt-zero-formal` workspace.
- Common probability distributions: Normal, Uniform, Bernoulli, Poisson.
- Fixed `Poisson::sample` infinite loop for large `lambda` (log-space Knuth).
- Published to crates.io as part of the layered `tpt-zero-formal` release (layer order
  L0 -> L4).

