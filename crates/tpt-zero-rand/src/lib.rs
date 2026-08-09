#![doc = include_str!("../README.md")]
#![no_std]
#![warn(missing_docs)]
#![forbid(unsafe_code)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

/// A source of pseudo-randomness.
///
/// Implementors produce a stream of bits grouped into `u32` (and optionally
/// `u64`) words. The remaining methods have default implementations built on
/// top of [`Rng::next_u32`], so a generator only needs to implement that one
/// method to gain all the helpers.
pub trait Rng {
    /// Returns the next `u32` in the sequence.
    ///
    /// Implementations must advance the generator's internal state so that
    /// successive calls yield different values (until the period wraps).
    fn next_u32(&mut self) -> u32;

    /// Returns the next `u64` in the sequence.
    ///
    /// The default implementation assembles the value from two `u32` outputs,
    /// so it matches the generator's natural word size and ordering.
    fn next_u64(&mut self) -> u64 {
        let hi = u64::from(self.next_u32());
        let lo = u64::from(self.next_u32());
        (hi << 32) | lo
    }

    /// Returns the next `f64` uniformly distributed in the half-open interval
    /// `[0, 1)`.
    ///
    /// This uses 53 bits of entropy (the precision of an `f64`'s mantissa) and
    /// scales by `2^-53`, so the result is never exactly `1.0`.
    fn next_f64(&mut self) -> f64 {
        const SCALE: f64 = 1.0 / ((1u64 << 53) as f64);
        let bits = self.next_u64() >> 11;
        (bits as f64) * SCALE
    }

    /// Fills `dest` with random bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_rand::{Rng, SeedableRng, XorShift64};
    ///
    /// let mut rng = XorShift64::seed_from_u64(7);
    /// let mut buf = [0u8; 8];
    /// rng.fill_bytes(&mut buf);
    /// assert_ne!(buf, [0u8; 8]);
    /// ```
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        let mut chunks = dest.chunks_exact_mut(4);
        for chunk in &mut chunks {
            let word = self.next_u32().to_le_bytes();
            chunk.copy_from_slice(&word);
        }
        let rem = chunks.into_remainder();
        if !rem.is_empty() {
            let word = self.next_u32().to_le_bytes();
            rem.copy_from_slice(&word[..rem.len()]);
        }
    }
}

/// Construct a generator from a seed.
///
/// The [`SeedableRng::from_seed`] method takes a single `u64`, which is enough
/// to deterministically derive the full internal state of every generator in
/// this crate.
pub trait SeedableRng: Sized {
    /// Creates a generator from the given `seed`.
    ///
    /// The same seed always produces the same stream of values.
    fn from_seed(seed: u64) -> Self;

    /// Convenience wrapper around [`SeedableRng::from_seed`]; provided so that
    /// callers have a clear naming for the single-`u64` seeding path.
    #[must_use]
    fn seed_from_u64(seed: u64) -> Self {
        Self::from_seed(seed)
    }
}

/// A 64-bit xorshift generator (`xorshift64*`-style shift pattern).
///
/// The internal state is a single `u64`. A zero state would stall the
/// generator (every output would be `0`), so a zero seed is remapped to a
/// non-zero state. The generator has a period of `2^64 - 1`.
///
/// # Examples
///
/// ```
/// use tpt_zero_rand::{Rng, SeedableRng, XorShift64};
///
/// let mut rng = XorShift64::seed_from_u64(12345);
/// let a = rng.next_u32();
/// let b = rng.next_u32();
/// assert_ne!(a, b);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    /// The non-zero state used when the requested seed maps to `0`.
    const NONZERO_FALLBACK: u64 = 0x9E37_79B9_7F4A_7C15;

    /// Creates a generator directly from an internal `u64` state.
    ///
    /// Unlike [`SeedableRng::from_seed`], this accepts a raw state value and
    /// remaps `0` to `XorShift64::NONZERO_FALLBACK` so the generator never
    /// stalls.
    #[must_use]
    pub fn new(state: u64) -> Self {
        let state = if state == 0 {
            Self::NONZERO_FALLBACK
        } else {
            state
        };
        Self { state }
    }
}

impl SeedableRng for XorShift64 {
    fn from_seed(seed: u64) -> Self {
        Self::new(seed)
    }
}

impl Rng for XorShift64 {
    fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        if self.state == 0 {
            self.state = Self::NONZERO_FALLBACK;
        }
        x
    }
}

/// A 32-bit PCG generator (PCG-XSH-RR, using the `rxs_m_xs` output function).
///
/// PCG combines a cheap linear congruential generator with a powerful
/// *output permutation*, giving much better statistical quality than a bare
/// LCG while staying tiny. Each generator carries a `state` and an `inc`
/// (the stream selector); `inc` must have its low bit set.
///
/// # Examples
///
/// ```
/// use tpt_zero_rand::{Rng, SeedableRng, Pcg32};
///
/// let mut rng = Pcg32::seed_from_u64(99);
/// let v = rng.next_u32();
/// let w = rng.next_u32();
/// assert_ne!(v, w);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pcg32 {
    state: u64,
    inc: u64,
}

impl Pcg32 {
    const MULTIPLIER: u64 = 6_364_136_223_846_793_005;
    const DEFAULT_INC: u64 = 0x5851_F42D_4C95_7F2D;

    /// Creates a PCG generator from an explicit `state` and `inc`.
    ///
    /// The low bit of `inc` is forced on, since PCG requires an odd increment.
    #[must_use]
    pub const fn new(state: u64, inc: u64) -> Self {
        Self {
            state,
            inc: inc | 1,
        }
    }
}

impl SeedableRng for Pcg32 {
    fn from_seed(seed: u64) -> Self {
        let mut rng = Pcg32::new(0, Self::DEFAULT_INC);
        rng.state = seed.wrapping_add(rng.inc);
        rng.state = rng
            .state
            .wrapping_mul(Self::MULTIPLIER)
            .wrapping_add(rng.inc);
        let _ = rng.next_u32();
        rng.state = seed.wrapping_add(rng.inc);
        rng.state = rng
            .state
            .wrapping_mul(Self::MULTIPLIER)
            .wrapping_add(rng.inc);
        let _ = rng.next_u32();
        rng
    }
}

impl Rng for Pcg32 {
    fn next_u32(&mut self) -> u32 {
        let old_state = self.state;
        self.state = old_state
            .wrapping_mul(Self::MULTIPLIER)
            .wrapping_add(self.inc);

        let xorshifted = (((old_state >> 18) ^ old_state) >> 27) as u32;
        let rot = (old_state >> 59) as u32;
        xorshifted.rotate_right(rot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::manual_range_contains
    )]
    fn in_unit(v: f64) -> bool {
        v >= 0.0 && v < 1.0
    }

    #[test]
    fn xorshift_zero_seed_rejected() {
        let mut rng = XorShift64::from_seed(0);
        assert_ne!(rng.state, 0);
        assert_ne!(rng.next_u64(), 0);
    }

    #[test]
    fn xorshift_deterministic_stream() {
        let mut rng = XorShift64::from_seed(0x1234_5678_9abc_def0);
        let outs = [
            rng.next_u64(),
            rng.next_u64(),
            rng.next_u64(),
            rng.next_u64(),
        ];
        assert_eq!(
            outs,
            [
                18_338_672_411_873_516_365,
                929_928_003_690_994_747,
                4_746_713_305_112_613_827,
                7_850_530_070_273_504_292,
            ]
        );
    }

    #[test]
    fn xorshift_u32_matches_high_word() {
        let mut rng = XorShift64::from_seed(1);
        let u64_val = rng.next_u64();
        let mut rng2 = XorShift64::from_seed(1);
        let u32_val = rng2.next_u32();
        assert_eq!(u32_val, (u64_val >> 32) as u32);
    }

    #[test]
    fn xorshift_fill_bytes_length() {
        let mut rng = XorShift64::from_seed(7);
        let mut buf = [0u8; 10];
        rng.fill_bytes(&mut buf);
        assert_ne!(buf, [0u8; 10]);
        let mut rng2 = XorShift64::from_seed(7);
        let expected = [rng2.next_u32().to_le_bytes(), rng2.next_u32().to_le_bytes()].concat();
        assert_eq!(&buf[..8], &expected[..]);
    }

    #[test]
    fn pcg_deterministic_stream() {
        let mut rng = Pcg32::seed_from_u64(42);
        let outs = [rng.next_u32(), rng.next_u32(), rng.next_u32()];
        assert_eq!(outs, [1_698_300_867, 3_725_897_186, 2_636_848_158]);
    }

    #[test]
    fn pcg_inc_is_odd() {
        let rng = Pcg32::new(0, 0);
        assert_eq!(rng.inc & 1, 1);
        let rng = Pcg32::new(0, 4);
        assert_eq!(rng.inc & 1, 1);
    }

    #[test]
    fn pcg_different_seeds_differ() {
        let mut a = Pcg32::seed_from_u64(1);
        let mut b = Pcg32::seed_from_u64(2);
        assert_ne!(a.next_u32(), b.next_u32());
    }

    #[test]
    fn next_f64_in_range() {
        let mut rng = XorShift64::from_seed(100);
        for _ in 0..1000 {
            assert!(in_unit(rng.next_f64()));
        }
    }

    #[test]
    fn next_f64_spreads_bits() {
        let mut rng = Pcg32::seed_from_u64(5);
        let f = rng.next_f64();
        assert!(in_unit(f));
        let scaled = f * ((1u64 << 53) as f64);
        assert!(scaled < (1u64 << 53) as f64);
    }

    #[test]
    fn seed_from_u64_matches_from_seed() {
        let a = XorShift64::seed_from_u64(123);
        let b = XorShift64::from_seed(123);
        assert_eq!(a, b);
        let c = Pcg32::seed_from_u64(123);
        let d = Pcg32::from_seed(123);
        assert_eq!(c, d);
    }

    mod proptest_sanity {
        #![allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::manual_range_contains
        )]

        use super::*;
        use proptest::prelude::*;

        fn count_distinct(samples: &[u32]) -> usize {
            let mut distinct = 0usize;
            for &s in samples {
                if !samples[..distinct].contains(&s) {
                    distinct += 1;
                }
            }
            distinct
        }

        proptest! {
            #[test]
            fn f64_not_constant(seed in any::<u64>()) {
                let mut rng = Pcg32::from_seed(seed);
                let mut buckets = [0u32; 50];
                let mut all_equal = true;
                for i in 0..50 {
                    let f = rng.next_f64();
                    prop_assert!(in_unit(f));
                    let bucket = (f * 1e6) as u32;
                    if i > 0 {
                        all_equal &= bucket == buckets[i - 1];
                    }
                    buckets[i] = bucket;
                }
                prop_assert!(!all_equal, "f64 stream was unexpectedly constant");
                prop_assert!(count_distinct(&buckets) > 1, "f64 stream produced only one bucket");
            }

            #[test]
            fn u32_not_constant(seed in any::<u64>()) {
                let mut rng = XorShift64::from_seed(seed.wrapping_add(1));
                let mut samples = [0u32; 50];
                for s in &mut samples {
                    *s = rng.next_u32();
                }
                prop_assert!(count_distinct(&samples) > 1, "u32 stream collapsed to one value");
            }

            #[test]
            fn f64_spread(seed in any::<u64>()) {
                let mut rng = Pcg32::from_seed(seed);
                let mut below_half = 0usize;
                for _ in 0..200 {
                    let v = rng.next_f64();
                    prop_assert!(in_unit(v));
                    if v < 0.5 {
                        below_half += 1;
                    }
                }
                let frac = below_half as f64 / 200.0;
                prop_assert!(
                    (0.30..0.70).contains(&frac),
                    "f64 not spread across [0,1): fraction below 0.5 was {frac}"
                );
            }
        }
    }
}
