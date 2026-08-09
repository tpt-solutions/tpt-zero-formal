//! Mirrors the familiar `rand` workflow but with a deterministic, `no_std`,
//! zero-dependency PRNG and distribution types.
use out_zero_formal::prelude::*;
use out_zero_formal::dist::Distribution;
use out_zero_formal::rand::{Rng, XorShift64};
use out_zero_formal::stats;

fn main() {
    // Equivalent to `rand::thread_rng().gen::<f64>()` but reproducible.
    let mut rng = XorShift64::new(0x1234_5678_9abc_def0);
    let samples: [f64; 8] = core::array::from_fn(|_| rng.next_f64());
    println!("uniform samples in [0,1): {samples:?}");

    let normal = Normal::standard();
    let z: [f64; 5] = core::array::from_fn(|_| normal.sample(&mut rng));
    println!("standard-normal draws: {z:?}");

    println!("sample mean: {}", stats::mean(&samples).unwrap());
}
