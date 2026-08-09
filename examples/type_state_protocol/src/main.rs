//! A connection protocol modelled as a type-state machine: illegal transitions
//! are rejected at compile time by the [`out_zero_formal::fsm`] builder.
use out_zero_formal::fsm::{Event, Machine, State, Transition};

struct Disconnected;
struct Connected;
struct Authenticated;
impl State for Disconnected {}
impl State for Connected {}
impl State for Authenticated {}

struct Connect;
struct Authenticate;
struct Disconnect;
impl Event for Connect {}
impl Event for Authenticate {}
impl Event for Disconnect {}

impl Transition<Disconnected, Connect> for () {
    type To = Connected;
}
impl Transition<Connected, Authenticate> for () {
    type To = Authenticated;
}
impl Transition<Connected, Disconnect> for () {
    type To = Disconnected;
}
impl Transition<Authenticated, Disconnect> for () {
    type To = Disconnected;
}

fn main() {
    let m = Machine::<Disconnected>::new();
    let m: Machine<Connected> = m.transition::<Connect, ()>();
    println!("after connect: {}", m.state_name());
    let m: Machine<Authenticated> = m.transition::<Authenticate, ()>();
    println!("after auth: {}", m.state_name());
    let m: Machine<Disconnected> = m.transition::<Disconnect, ()>();
    println!("after disconnect: {}", m.state_name());
}
