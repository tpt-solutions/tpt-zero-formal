# Security Policy

## Supported versions

Only the latest `0.1.x` release line of each published crate is supported with
security fixes.

## Reporting a vulnerability

Please report security issues **privately**. Do not open a public GitHub issue.

- Email the maintainers at security@tpt-solutions.example (replace with the real
  address before publishing).
- Include a description, affected crate(s)/version(s), and a minimal
  reproduction.

You will receive an acknowledgement within a few business days. Once the issue is
confirmed and a fix is prepared, we will coordinate a disclosure timeline with
you.

## Formal-methods caveats

These crates provide *assistance* for verification (ghost state, proof carrying,
design-by-contract, SMT-lite solving). They are **not** a verified compiler or a
proof assistant. `unsafe` is forbidden in the workspace; soundness-critical
constructs (e.g. `Witness`, `Ghost`) must preserve their invariants by
construction and are covered by unit + doctests.
