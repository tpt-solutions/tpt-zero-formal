//! Integration test building a small turnstile-style machine and exercising
//! its declared transitions. Illegal transitions are covered by the
//! `compile_fail` doctest in the crate root; this file proves the legal
//! path type-checks and runs.

use tpt_zero_fsm::{Event, Invariant, Machine, State, Transition, invariant};

struct Locked;
struct Unlocked;
impl State for Locked {}
impl State for Unlocked {}

struct Unlock;
struct Lock;
impl Event for Unlock {}
impl Event for Lock {}

// The transition table: Locked --unlock--> Unlocked --lock--> Locked.
impl Transition<Locked, Unlock> for () {
    type To = Unlocked;
}
impl Transition<Unlocked, Lock> for () {
    type To = Locked;
}

#[test]
fn build_and_run_turnstile() {
    let m = Machine::<Locked>::new();
    assert!(m.state_name().ends_with("Locked"));

    let m: Machine<Unlocked> = m.transition::<Unlock, ()>();
    assert!(m.state_name().ends_with("Unlocked"));
    invariant!(m.check());

    let m: Machine<Locked> = m.transition::<Lock, ()>();
    assert!(m.state_name().ends_with("Locked"));
    assert!(m.check());
}

#[test]
fn machine_is_zero_sized_across_states() {
    assert_eq!(std::mem::size_of::<Machine<Locked>>(), 0);
    assert_eq!(std::mem::size_of::<Machine<Unlocked>>(), 0);
}

#[test]
fn many_cycles_stay_well_formed() {
    // Locked -> Unlocked -> Locked repeatedly; each hop is a type change with
    // no allocation. The invariant holds at every step.
    let mut locked = Machine::<Locked>::new();
    for _ in 0..1_000 {
        let unlocked = locked.transition::<Unlock, ()>();
        assert!(unlocked.check());
        locked = unlocked.transition::<Lock, ()>();
        assert!(locked.check());
    }
}

// The following, if uncommented, must NOT compile because there is no
// `Transition<Locked, Lock>` impl (a locked turnstile cannot be locked
// again). This documents the compile-time safety guarantee:
//
//     let _ = Machine::<Locked>::new().transition::<Lock, ()>();
