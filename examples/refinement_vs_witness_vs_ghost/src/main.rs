//! Three closely related "value-with-proof" patterns and when to reach for
//! each, using the `refinement`, `witness`, and `ghost` crates.
use out_zero_formal::ghost::{Ghost, GhostProven, Unproven};
use out_zero_formal::refinement::{Predicate, Refined};
use out_zero_formal::witness::{Proof, Witness};

/// A decidable predicate: "this `i64` is strictly positive".
struct Positive;
impl Predicate<i64> for Positive {
    fn check(v: &i64) -> bool {
        *v > 0
    }
}

/// A proof type naming the same property at the type level.
struct IsPositive;
impl Proof for IsPositive {}

fn main() {
    // Refinement: the value is *checked* against a decidable predicate at
    // construction; out-of-range values are rejected with a `Result`.
    let refined = Refined::<i64, Positive>::new(10).expect("must be positive");
    println!("refined: {}", refined.get());

    // Witness: construction requires an explicit *proof value* minted by a
    // checker that just established the property.
    fn checked_positive(v: i64) -> Option<Witness<i64, IsPositive>> {
        if v > 0 {
            Some(Witness::from_proof(v, IsPositive))
        } else {
            None
        }
    }
    let witness = checked_positive(10).expect("must be positive");
    println!("witness: {}", witness.value());

    // Ghost: the property is *tracked* at the type level as provenance, erased
    // at runtime. Here we assert it (sound only where the caller guarantees it).
    let ghost: Ghost<i64, Unproven> = Ghost::new(10);
    let ghost: GhostProven<i64> = ghost.assume_proven();
    println!("ghost: {}", ghost.value());
}
