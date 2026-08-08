# Comparison with other crates

`tpt-zero-formal` is not trying to replace `nalgebra`, `rand`, `typenum`, or
`contracts`. It occupies a specific niche: **zero-dependency, `no_std`,
panic-free primitives for formal methods and numerical work.** Below is an honest
comparison so you can decide.

## vs `contracts`

- `contracts` brings a `requires!` / `ensures!` macro pair (and more) but pulls
  in `syn` / `quote` proc-macro dependencies and is not `no_std`.
- `out-zero-contract` (internal here) provides `requires!` / `ensures!` with a
  `checked` feature, no proc-macro runtime dependency, and a `no_std` build. It is
  intentionally smaller in scope.

**Pick `contracts`** if you are on `std` and want the full macro suite.
**Pick `out-zero-contract`** when you must stay `no_std` / zero-dependency.

## vs `nalgebra` / `ndarray` / `num` / `num-dual`

- Those are powerful, general linear-algebra / numeric stacks, but they depend on
  `std` (and several crates) and are far larger than a bare-metal target wants.
- `tpt-zero-tensor` + `tpt-zero-linalg` + `tpt-zero-decomp` + `tpt-zero-solver`
  + `tpt-zero-eigen` cover the common fixed-size cases (`N`-by-`N`, const
  generics) with **no external dependencies** and a `no_std` build.

**Pick `nalgebra`** for large/dynamic matrices, GPU, or a rich ecosystem.
**Pick the `tpt-zero-*` numerics** for small, fixed-rank, verified, `no_std`
workloads.

## vs `rand`

- `rand` is the de-facto RNG ecosystem; it needs `std` and external crates.
- `tpt-zero-rand` is a tiny, deterministic, seedable PRNG (Xorshift / PCG) with a
  `no_std`, zero-dependency build.

**Pick `rand`** for production RNG with entropy sources and many backends.
**Pick `tpt-zero-rand`** for reproducible, dependency-free sampling in
`no_std`.

## vs `typenum` / `refinement` / `bounded-integer` / `az`

- `typenum` provides type-level integers via a macro-generated trait tower (large
  generated code, but very complete). `out-zero-type-level` uses const generics
  directly and is much smaller.
- `refinement` and `bounded-integer` provide refinement / bounded integer types
  but are `std`-based. `tpt-zero-refinement` and `out-zero-bounded` are
  `no_std` and zero-dependency.

**Pick the established crates** when you are on `std` and want their breadth.
**Pick the `out-zero-*` equivalents** for `no_std` / zero-dependency contexts.

## Summary

| Need | tpt-zero-formal | Established alternative |
|---|---|---|
| Design-by-contract | `out-zero-contract` | `contracts` (std) |
| Linear algebra (small, fixed) | `tpt-zero-*` numerics | `nalgebra` / `ndarray` (std) |
| RNG | `tpt-zero-rand` | `rand` (std) |
| Type-level numbers | `out-zero-type-level` | `typenum` |
| Bounded / refinement ints | `out-zero-bounded` / `tpt-zero-refinement` | `bounded-integer` / `refinement` (std) |
| Verified casts | `out-zero-safe-cast` | `az` |

The consistent theme: if `no_std` and zero external dependencies are requirements,
`tpt-zero-formal` is the option; otherwise the established crates are usually
more feature-complete.
