# Choosing a crate

This is a problem → crate index. For the full published-vs-internal breakdown,
see [`crate-selection.md`](crate-selection.md).

## "I need to…"

- **…draw samples from a distribution** → `tpt-zero-rand` (PRNG) +
  `tpt-zero-dist` (`Normal`, `Uniform`, `Bernoulli`, `Poisson`) +
  `tpt-zero-sampler` (inference samplers).
- **…compute mean / variance / quantiles over a slice** → `tpt-zero-stats`.
- **…do fixed-size matrix / tensor math** → `tpt-zero-tensor` +
  `tpt-zero-linalg`; add `tpt-zero-decomp` / `tpt-zero-solver` /
  `tpt-zero-eigen` / `tpt-zero-grad` as needed.
- **…run a Bayesian update** → `tpt-zero-bayes` (Beta–Bernoulli,
  Normal–Normal, Gamma–Poisson conjugate pairs).
- **…simulate with Monte Carlo** → `tpt-zero-monte-carlo`.
- **…model a Markov chain** → `tpt-zero-markov`.
- **…encode a finite state machine with checked transitions** →
  `tpt-zero-fsm` + `tpt-zero-invariant`.
- **…carry a compile-time proof of a property** → `tpt-zero-witness`
  (with `tpt-zero-ghost` for ghost state, `tpt-zero-refinement` for
  refinement types, `out-zero-bounded` for range-bounded integers).
- **…write design-by-contract `requires!` / `ensures!`** → `out-zero-contract`.
- **…panic-free numeric casts** → `out-zero-safe-cast`.
- **…everything, namespaced behind one dependency** → `out-zero-formal`
  (umbrella) with `use out_zero_formal::prelude::*;`.

## "Which dependency should I pick?"

- **No heap, no `std`, no external crates.** Every crate here is `no_std` and
  zero-dependency by construction. If you target bare metal or a verified
  runtime, start here.
- **A single well-known alternative is fine if** you can accept its dependency
  footprint and `std` requirement (e.g. `nalgebra` for linear algebra,
  `rand` for RNG, `typenum` for type-level numbers). Use those when you are not
  constrained by `no_std` / zero-dep.
- **You need the formal-methods story** (witness types, ghost state,
  contracts). The `out-zero-*` family is purpose-built for that and has no
  equivalent on crates.io that is also zero-dependency / `no_std`.

## Quick start

```toml
[dependencies]
out-zero-formal = { version = "0.1", features = ["layer2", "layer3"] }
```

```rust
use out_zero_formal::prelude::*;

let v = Tensor::<f64, 3>::from_fn(|i| (i as f64) - 1.0);
assert_eq!(v.get(0), Some(&-1.0));

let n = Normal::standard();
let _ = n.pdf(0.0);

let posterior = Beta::new(1.0, 1.0).unwrap().posterior(7, 3);
assert!((posterior.mean() - 8.0 / 12.0).abs() < 1e-12);
```
