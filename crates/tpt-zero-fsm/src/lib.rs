#![doc = include_str!("../README.md")]

#![no_std]
#![warn(missing_docs)]
#![forbid(unsafe_code)]
// `Machine<S>` is a zero-sized `PhantomData` wrapper: `new` yields a value
// that only ever changes type, and `Default` is intentionally trivial, so
// these pedantic lints add noise rather than value here.
#![allow(clippy::new_without_default)]

use core::marker::PhantomData;

// Re-export the invariant tooling so downstream crates can assert machine
// well-formedness after a transition without a separate dependency.
pub use tpt_zero_invariant::{Invariant, invariant};

/// A marker trait for the *states* of a finite state machine.
///
/// Implement this for a zero-sized marker type to make it usable as the state
/// parameter of a [`Machine`]. States carry no runtime data; they exist only
/// at the type level.
///
/// # Examples
///
/// ```
/// use tpt_zero_fsm::State;
///
/// struct Idle;
/// impl State for Idle {}
/// ```
pub trait State {}

/// A marker trait for the *events* that drive a finite state machine.
///
/// Implement this for a zero-sized marker type to make it usable as the event
/// parameter of a [`Transition`]. Events carry no runtime data; they select a
/// declared transition at the type level.
///
/// # Examples
///
/// ```
/// use tpt_zero_fsm::Event;
///
/// struct Start;
/// impl Event for Start {}
/// ```
pub trait Event {}

/// A declared, legal transition from state `From` on event `Evt`.
///
/// Implement this trait once per allowed `(From, Evt)` pair, setting the
/// associated [`To`](Transition::To) type to the resulting state. The
/// implementing type is a *transition table* marker (often the unit type
/// `()`); it lets several independent tables coexist without conflicting
/// blanket impls.
///
/// Because [`Machine::transition`] requires `T: Transition<S, E>`, only pairs
/// with an impl can be used, so illegal transitions fail to compile.
///
/// # Examples
///
/// ```
/// use tpt_zero_fsm::{Event, State, Transition};
///
/// struct Off;
/// struct On;
/// impl State for Off {}
/// impl State for On {}
///
/// struct Toggle;
/// impl Event for Toggle {}
///
/// impl Transition<Off, Toggle> for () { type To = On; }
/// impl Transition<On, Toggle> for () { type To = Off; }
/// ```
pub trait Transition<From: State, Evt: Event> {
    /// The state the machine is in after this transition completes.
    type To: State;
}

/// A zero-sized finite state machine whose current state is the type
/// parameter `S`.
///
/// `Machine<S>` holds no runtime data â€” only a [`PhantomData<S>`] â€” so it is a
/// zero-sized type and every transition is a zero-cost change of type rather
/// than a mutation of stored data.
///
/// [`PhantomData<S>`]: core::marker::PhantomData
///
/// # Examples
///
/// ```
/// use tpt_zero_fsm::{Event, Machine, State, Transition};
///
/// struct A;
/// struct B;
/// impl State for A {}
/// impl State for B {}
///
/// struct Go;
/// impl Event for Go {}
/// impl Transition<A, Go> for () { type To = B; }
///
/// let m = Machine::<A>::new();
/// let _m: Machine<B> = m.transition::<Go, ()>();
/// ```
#[repr(transparent)]
pub struct Machine<S: State> {
    _state: PhantomData<S>,
}

impl<S: State> Machine<S> {
    /// Creates a machine positioned in state `S`.
    ///
    /// This is a `const fn` and performs no allocation; the returned value is
    /// zero-sized.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_fsm::{Machine, State};
    ///
    /// struct Ready;
    /// impl State for Ready {}
    ///
    /// let _m = Machine::<Ready>::new();
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self {
            _state: PhantomData,
        }
    }

    /// Consumes the machine and returns it in the state reached by applying
    /// event `E` through the transition table `T`.
    ///
    /// This compiles only when a `T: Transition<S, E>` impl exists, so
    /// undeclared transitions are rejected at compile time. The resulting
    /// state is `<T as Transition<S, E>>::To`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_fsm::{Event, Machine, State, Transition};
    ///
    /// struct Closed;
    /// struct Open;
    /// impl State for Closed {}
    /// impl State for Open {}
    ///
    /// struct OpenIt;
    /// impl Event for OpenIt {}
    /// impl Transition<Closed, OpenIt> for () { type To = Open; }
    ///
    /// let m = Machine::<Closed>::new();
    /// let _m: Machine<Open> = m.transition::<OpenIt, ()>();
    /// ```
    #[must_use]
    pub fn transition<E, T>(self) -> Machine<<T as Transition<S, E>>::To>
    where
        E: Event,
        T: Transition<S, E>,
    {
        Machine::new()
    }

    /// Returns the name of the current state type, useful for logging or
    /// debugging a machine whose state lives only in its type.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_fsm::{Machine, State};
    ///
    /// struct Booting;
    /// impl State for Booting {}
    ///
    /// let m = Machine::<Booting>::new();
    /// assert!(m.state_name().ends_with("Booting"));
    /// ```
    #[must_use]
    pub fn state_name(&self) -> &'static str {
        core::any::type_name::<S>()
    }
}

impl<S: State> Clone for Machine<S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S: State> Copy for Machine<S> {}

impl<S: State> core::fmt::Debug for Machine<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Machine")
            .field("state", &core::any::type_name::<S>())
            .finish()
    }
}

/// Every [`Machine`] is well-formed by construction: its state is encoded in
/// the type system, so there is no reachable ill-formed value.
///
/// The impl always returns `true`, letting a machine participate uniformly in
/// [`Invariant`]-based checks (for example via the re-exported
/// [`invariant!`] macro) after a transition.
impl<S: State> Invariant for Machine<S> {
    fn check(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Locked;
    struct Unlocked;
    impl State for Locked {}
    impl State for Unlocked {}

    struct Unlock;
    struct Lock;
    impl Event for Unlock {}
    impl Event for Lock {}

    impl Transition<Locked, Unlock> for () {
        type To = Unlocked;
    }
    impl Transition<Unlocked, Lock> for () {
        type To = Locked;
    }

    #[test]
    fn machine_is_zero_sized() {
        assert_eq!(core::mem::size_of::<Machine<Locked>>(), 0);
    }

    #[test]
    fn round_trip_transitions() {
        let m = Machine::<Locked>::new();
        let m: Machine<Unlocked> = m.transition::<Unlock, ()>();
        let _m: Machine<Locked> = m.transition::<Lock, ()>();
    }

    #[test]
    fn state_name_reflects_type() {
        let m = Machine::<Locked>::new();
        assert!(m.state_name().ends_with("Locked"));
        let m = m.transition::<Unlock, ()>();
        assert!(m.state_name().ends_with("Unlocked"));
    }

    #[test]
    fn machine_invariant_holds() {
        let m = Machine::<Locked>::new();
        assert!(m.check());
        invariant!(m.check());
    }

    #[test]
    fn machine_is_copy() {
        let m = Machine::<Locked>::new();
        let copy = m;
        // `m` is still usable after the copy because `Machine` is `Copy`.
        assert!(copy.check());
        assert!(m.check());
    }

    #[test]
    fn machine_debug_uses_writer() {
        use core::fmt::Write;

        // A tiny no_std sink to exercise the Debug impl without alloc.
        struct Sink {
            buf: [u8; 128],
            len: usize,
        }
        impl Write for Sink {
            fn write_str(&mut self, s: &str) -> core::fmt::Result {
                for &b in s.as_bytes() {
                    if self.len < self.buf.len() {
                        self.buf[self.len] = b;
                        self.len += 1;
                    }
                }
                Ok(())
            }
        }

        let m = Machine::<Locked>::new();
        let mut sink = Sink {
            buf: [0; 128],
            len: 0,
        };
        write!(sink, "{m:?}").unwrap();
        let written = core::str::from_utf8(&sink.buf[..sink.len]).unwrap();
        assert!(written.contains("Machine"));
    }
}
