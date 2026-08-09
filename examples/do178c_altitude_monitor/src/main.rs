//! A DO-178C-flavoured altitude monitor: every transition through the
//! certified flight envelope is guarded by a precondition/postcondition and
//! the resulting state is checked against an [`Invariant`].
use out_zero_formal::bounded::BoundedInt;
use out_zero_formal::contract::{ensures, requires};
use out_zero_formal::invariant::{Invariant, check_invariant};

/// Aircraft altitude held in a certified safe envelope [0, 41_000] ft.
#[derive(Clone, Copy, Debug)]
pub struct Altitude {
    feet: BoundedInt<0, 41000>,
}

impl Invariant for Altitude {
    fn check(&self) -> bool {
        let v = self.feet.value();
        (0i64..=41000).contains(&v)
    }
}

/// Climb by a non-negative `delta` feet, keeping the aircraft in its envelope.
pub fn climb(current: i64, delta: i64) -> Altitude {
    requires!(
        REQ = "SRS-ALT-014",
        delta >= 0,
        "climb rate must be non-negative"
    );
    let next = (current + delta).clamp(0, 41000);
    let a = Altitude {
        feet: BoundedInt::new_clamped(next),
    };
    ensures!(
        REQ = "SRS-ALT-015",
        a.check(),
        "altitude remains within certified envelope"
    );
    a
}

fn main() {
    let a = climb(1000, 5000);
    let a = check_invariant!(a);
    println!("altitude after climb: {} ft", a.feet.value());
    assert!(a.check());
}
