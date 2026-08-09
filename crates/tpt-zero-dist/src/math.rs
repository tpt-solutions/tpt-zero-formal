//! Local floating-point helpers for this `core`-only crate.
//!
//! `f64::sqrt`, `f64::ln`, and `f64::exp` are not available in every
//! `core`-only target, so the helpers here delegate to
//! [`tpt_zero_float`] (a subnormal-safe, `no_std` implementation) while the
//! floor remains implemented locally.

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

/// Computes the square root of `x >= 0.0`.
///
/// Returns `f64::NAN` for negative inputs and for `f64::NAN`, matching the
/// behaviour of `f64::sqrt`. Delegates to the shared, subnormal-safe
/// [`tpt_zero_float`] implementation so it remains accurate for tiny and huge
/// magnitudes.
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
    tpt_zero_float::sqrt(x)
}

/// Computes `e^x`.
///
/// Reimplements `f64::exp` (unavailable in this `core`-only crate). Delegates
/// to the shared, subnormal-safe [`tpt_zero_float`] implementation, which
/// handles the full range (including the subnormal underflow region) and never
/// returns a negative value for large negative `x`.
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
    tpt_zero_float::exp(x)
}

/// Computes the natural logarithm of `x > 0`.
///
/// Reimplements `f64::ln` (unavailable in this `core`-only crate). Delegates to
/// the shared [`tpt_zero_float`] implementation, which decomposes `x` into a
/// mantissa and binary exponent and evaluates `ln(m)` with an `atanh` series;
/// it is accurate for subnormal magnitudes and returns `-inf` for `0`.
///
/// Returns `f64::NAN` for negative inputs, `-inf` for `0.0`, and `+inf` for
/// `+inf`.
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
    tpt_zero_float::ln(x)
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
