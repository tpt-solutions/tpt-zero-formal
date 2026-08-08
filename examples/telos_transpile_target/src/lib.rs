//! A minimal `no_std`, zero-alloc target demonstrating the primitive surface
//! that the `tpt-telos` transpiler lowers high-level contract / refinement code
//! into: design-by-contract assertions plus refinement types.
#![no_std]

use tpt_zero_formal::bounded::BoundedInt;
use tpt_zero_formal::contract::{ensures, requires};
use tpt_zero_formal::refinement::{Predicate, Refined};

/// Predicate: a device port is "open" (its value is non-zero).
pub struct PortOpen;
impl Predicate<u8> for PortOpen {
    fn check(v: &u8) -> bool {
        *v != 0
    }
}

/// A transpiled device-driver entry point with lowered contracts.
///
/// Returns `None` if the value fails the refinement, mirroring the runtime
/// failure mode the transpiler emits for a violated postcondition.
pub fn set_port(pin: u8, value: u8) -> Option<Refined<u8, PortOpen>> {
    requires!(pin < 64, "pin index within device range");
    let r = match Refined::<u8, PortOpen>::new(value) {
        Ok(r) => r,
        Err(_) => return None,
    };
    ensures!(r.get() == &value, "value written unchanged");
    Some(r)
}

/// A transpiled bounded register write lowered to a `BoundedInt`.
pub fn write_register(value: i64) -> BoundedInt<0, 4095> {
    BoundedInt::new_clamped(value)
}
