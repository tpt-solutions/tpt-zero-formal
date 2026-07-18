//! Local, dependency-free floating-point helpers.
//!
//! This crate targets a `core`-only environment where the `std` float methods
//! `f64::sqrt`, `f64::ln`, `f64::exp`, and `f64::floor` are not available.
//! The functions here reimplement them from scratch (Newton's method for the
//! square root, range-reduced Taylor series for the exponential, and an
//! `atanh`-style series for the natural logarithm) with enough accuracy for
//! the distribution PDFs, CDFs, and samplers in this crate.

/// `ln(2)`, used for range reduction in [`exp`] and [`ln`].
const LN2: f64 = core::f64::consts::LN_2;

/// Rounds `x` to the nearest integer, ties away from zero.
fn round(x: f64) -> f64 {
    let t = x as i64;
    let frac = x - (t as f64);
    if frac >= 0.5 {
        (t + 1) as f64
    } else if frac <= -0.5 {
        (t - 1) as f64
    } else {
        t as f64
    }
}

/// Returns the largest integer not greater than `x` (mathematical floor).
///
/// Mirrors `f64::floor`, which is unavailable in this `core`-only crate.
///
/// # Examples
///
/// ```
/// use tpt_zero_dist::math::floor;
///
/// assert_eq!(floor(2.7), 2.0);
/// assert_eq!(floor(-2.1), -3.0);
/// assert_eq!(floor(4.0), 4.0);
/// ```
#[must_use]
pub fn floor(x: f64) -> f64 {
    if !x.is_finite() {
        return x;
    }
    let t = x as i64 as f64;
    if t > x { t - 1.0 } else { t }
}

/// Computes the square root of `x` via Newton's method.
///
/// Returns `f64::NAN` for negative inputs and for `f64::NAN`, matching the
/// behaviour of `f64::sqrt` (which is unavailable in this `core`-only crate).
///
/// # Examples
///
/// ```
/// use tpt_zero_dist::math::sqrt;
///
/// assert!((sqrt(4.0) - 2.0).abs() < 1e-12);
/// assert!((sqrt(2.0) - 1.414_213_562_373_095).abs() < 1e-12);
/// assert!(sqrt(-1.0).is_nan());
/// ```
#[must_use]
pub fn sqrt(x: f64) -> f64 {
    if x.is_nan() || x < 0.0 {
        return f64::NAN;
    }
    if x == 0.0 || x == f64::INFINITY {
        return x;
    }
    let mut guess = x;
    let mut prev = 0.0;
    for _ in 0..64 {
        if (guess - prev).abs() < 1e-15 * guess {
            break;
        }
        prev = guess;
        guess = 0.5 * (guess + x / guess);
    }
    guess
}

/// Computes `e^x` via range reduction and a Taylor series.
///
/// Reimplements `f64::exp` (unavailable here). The exponent is reduced as
/// `x = k * ln2 + r` with `|r| <= ln2 / 2`, `2^k` is formed by direct exponent
/// bit manipulation, and `e^r` is summed from its Taylor series.
///
/// # Examples
///
/// ```
/// use tpt_zero_dist::math::exp;
///
/// assert!((exp(0.0) - 1.0).abs() < 1e-12);
/// assert!((exp(1.0) - core::f64::consts::E).abs() < 1e-12);
/// ```
#[must_use]
pub fn exp(x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    if x == 0.0 {
        return 1.0;
    }
    if x < -745.0 {
        return 0.0;
    }
    if x > 709.0 {
        return f64::INFINITY;
    }
    let k = round(x / LN2);
    let r = x - k * LN2;
    let exp_k = f64::from_bits(((1023.0 + k) as i64 as u64) << 52);
    let mut term = 1.0;
    let mut sum = 1.0;
    let mut fact = 1.0;
    for i in 1..=14 {
        fact *= f64::from(i);
        term *= r;
        sum += term / fact;
    }
    exp_k * sum
}

/// Computes the natural logarithm of `x > 0`.
///
/// Reimplements `f64::ln` (unavailable here). The input is decomposed into its
/// mantissa and binary exponent, the mantissa `m in [1, 2)` is scaled to
/// `[sqrt(1/2), sqrt(2))`, and `ln(m)` is evaluated with the rapidly
/// converging `atanh` series `2 * (s + s^3/3 + s^5/5 + ...)` where
/// `s = (m - 1) / (m + 1)`.
///
/// Returns `f64::NAN` for negative inputs, `f64::NEG_INFINITY` for `0.0`, and
/// `f64::NAN` for `f64::NAN`.
///
/// # Examples
///
/// ```
/// use tpt_zero_dist::math::ln;
///
/// assert!((ln(1.0)).abs() < 1e-12);
/// assert!((ln(core::f64::consts::E) - 1.0).abs() < 1e-12);
/// assert!(ln(-1.0).is_nan());
/// ```
#[must_use]
pub fn ln(x: f64) -> f64 {
    if x.is_nan() || x < 0.0 {
        return f64::NAN;
    }
    if x == 0.0 {
        return f64::NEG_INFINITY;
    }
    if x == f64::INFINITY {
        return f64::INFINITY;
    }
    // Decompose x = m * 2^e with m in [1, 2).
    let bits = x.to_bits();
    let mut exponent = ((bits >> 52) & 0x7ff) as i64;
    let mut mantissa;
    if exponent == 0 {
        // Subnormal: normalize by scaling up by 2^54.
        let scaled = x * f64::from_bits(0x4350_0000_0000_0000);
        let sbits = scaled.to_bits();
        exponent = ((sbits >> 52) & 0x7ff) as i64 - 54;
        mantissa = f64::from_bits((sbits & 0x000f_ffff_ffff_ffff) | 0x3ff0_0000_0000_0000);
    } else {
        mantissa = f64::from_bits((bits & 0x000f_ffff_ffff_ffff) | 0x3ff0_0000_0000_0000);
    }
    let mut e = exponent - 1023;
    // Center the mantissa around 1 for fast series convergence.
    if mantissa > core::f64::consts::SQRT_2 {
        mantissa *= 0.5;
        e += 1;
    }
    let s = (mantissa - 1.0) / (mantissa + 1.0);
    let s2 = s * s;
    let mut term = s;
    let mut sum = 0.0;
    let mut k = 1.0;
    for _ in 0..30 {
        sum += term / k;
        term *= s2;
        k += 2.0;
    }
    2.0 * sum + (e as f64) * LN2
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    use super::*;

    #[test]
    fn sqrt_matches_reference() {
        for &v in &[0.0, 1.0, 2.0, 3.0, 4.0, 9.0, 1e-6, 1e6, 123.456] {
            let got = sqrt(v);
            let want = libm_sqrt(v);
            assert!((got - want).abs() < 1e-9 * (1.0 + want), "sqrt({v}) = {got}, want {want}");
        }
        assert!(sqrt(-1.0).is_nan());
        assert!(sqrt(f64::NAN).is_nan());
    }

    #[test]
    fn exp_matches_reference() {
        for &v in &[-5.0, -1.0, 0.0, 0.5, 1.0, 2.5, 10.0] {
            let got = exp(v);
            let want = std::f64::consts::E.powf(v);
            assert!((got - want).abs() < 1e-9 * (1.0 + want), "exp({v}) = {got}, want {want}");
        }
    }

    #[test]
    fn ln_matches_reference() {
        for &v in &[1e-6, 0.5, 1.0, 2.0, core::f64::consts::E, 10.0, 1e6] {
            let got = ln(v);
            let want = v.ln();
            assert!((got - want).abs() < 1e-9 * (1.0 + want.abs()), "ln({v}) = {got}, want {want}");
        }
        assert!(ln(-1.0).is_nan());
        assert_eq!(ln(0.0), f64::NEG_INFINITY);
    }

    #[test]
    fn exp_ln_round_trip() {
        for &v in &[0.1, 0.5, 1.0, 3.0, 7.5] {
            assert!((ln(exp(v)) - v).abs() < 1e-9);
            assert!((exp(ln(v)) - v).abs() < 1e-9);
        }
    }

    #[test]
    fn floor_matches_reference() {
        for &v in &[2.7, -2.1, 4.0, 0.0, -0.5, 100.999] {
            assert_eq!(floor(v), v.floor());
        }
    }

    // A reference sqrt via the std method, used only in tests.
    fn libm_sqrt(x: f64) -> f64 {
        x.sqrt()
    }
}
