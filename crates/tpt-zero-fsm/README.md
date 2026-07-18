# tpt-zero-fsm

[![crates.io](https://img.shields.io/crates/v/tpt-zero-fsm.svg)](https://crates.io/crates/tpt-zero-fsm)
[![docs.rs](https://docs.rs/tpt-zero-fsm/badge.svg)](https://docs.rs/tpt-zero-fsm)
[![license](https://img.shields.io/crates/l/tpt-zero-fsm.svg)](#license)

Zero-allocation, strongly-typed finite state machine builder with
compile-time transition checking for `no_std`. Part of the
[tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal) ecosystem.

## Quick example

```rust
use tpt_zero_fsm::{Event, Machine, State, Transition};

// States and events are zero-sized marker types.
struct Locked;
struct Unlocked;
impl State for Locked {}
impl State for Unlocked {}

struct PushUnlock;
struct PushLock;
impl Event for PushUnlock {}
impl Event for PushLock {}

// Declare the only legal transitions at compile time.
impl Transition<Locked, PushUnlock> for () { type To = Unlocked; }
impl Transition<Unlocked, PushLock> for () { type To = Locked; }

let m = Machine::<Locked>::new();
let m = m.transition::<PushUnlock, ()>(); // Machine<Unlocked>
let _m = m.transition::<PushLock, ()>();  // Machine<Locked>
```

Only declared transitions compile. Attempting an undeclared transition — for
example `Machine::<Locked>::new().transition::<PushLock, ()>()` — is a
compile error, because no `Transition<Locked, PushLock>` impl exists.

The `Machine<S>` newtype carries the current state purely in its type
parameter (`PhantomData`), so it is zero-sized and every transition is a
zero-cost, allocation-free type change.

## Invariants

The machine integrates with
[tpt-zero-invariant](https://crates.io/crates/tpt-zero-invariant): every
`Machine<S>` implements `Invariant`, and the `invariant!` macro is
re-exported so callers can assert well-formedness after a transition.

## Features

| Feature | Default | Enables |
|---|---|---|
| `alloc` | off | reserved for future alloc-dependent helpers |
| `std` | off | implies `alloc` |

This crate builds with `--no-default-features` (pure `core`, no `alloc`).

## Install

```sh
cargo add tpt-zero-fsm
```

## License

Dual-licensed under [MIT](../../LICENSE-MIT) or
[Apache-2.0](../../LICENSE-APACHE), at your option.
