//! Design-by-contract basics: preconditions, postconditions, and invariants
//! expressed with the zero-dependency `contract` and `bounded` crates.
//!
//! By default the checks compile to `debug_assert!` (zero cost in release).
//! Build the `contract` crate with its `checked` feature to make them always-on.
use tpt_zero_formal::bounded::BoundedInt;
use tpt_zero_formal::contract::{ensures, requires};

/// Integer division guarded by a precondition on the divisor.
fn checked_div(a: i64, b: i64) -> i64 {
    requires!(b != 0, "divisor must be non-zero");
    let q = a / b;
    ensures!(b * q == a - (a % b), "division identity must hold");
    q
}

/// A clamped accumulator that never leaves a validated range.
fn record(value: i64) -> BoundedInt<-100, 100> {
    BoundedInt::new_clamped(value)
}

fn main() {
    println!("checked_div(10, 3) = {}", checked_div(10, 3));
    assert_eq!(checked_div(10, 3), 3);

    let lo = record(-500);
    let hi = record(9000);
    println!("clamped -500 -> {}", lo.value());
    println!("clamped 9000 -> {}", hi.value());
    assert_eq!(lo.value(), -100);
    assert_eq!(hi.value(), 100);
}
