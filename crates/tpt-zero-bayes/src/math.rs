//! Local, dependency-free floating-point helpers.
//!
//! This crate targets a `core`-only environment where the `std` float methods
//! `f64::sqrt`, `f64::ln`, `f64::exp`, and the gamma function are not
//! available. The functions here reimplement them from scratch with enough
//! accuracy for the conjugate-posterior densities and moments in this crate.

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
/// use tpt_zero_bayes::math::floor;
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
    if t > x {
        t - 1.0
    } else {
        t
    }
}

/// Computes the square root of `x` via Newton's method.
///
/// Returns `f64::NAN` for negative inputs and for `f64::NAN`, matching the
/// behaviour of `f64::sqrt` (which is unavailable in this `core`-only crate).
///
/// # Examples
///
/// ```
/// use tpt_zero_bayes::math::sqrt;
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
/// `x = k * ln2 + r` with `|r| <= ln2 / 2`, `2^k` is formed by direct
/// exponent bit manipulation, and `e^r` is summed from its Taylor series.
///
/// # Examples
///
/// ```
/// use tpt_zero_bayes::math::exp;
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
/// Reimplements `f64::ln` (unavailable here). The input is decomposed into a
/// mantissa and binary exponent and `ln(m)` is evaluated with the rapidly
/// converging `atanh` series `2 * (s + s^3/3 + s^5/5 + ...)` where
/// `s = (m - 1) / (m + 1)`.
///
/// Returns `f64::NAN` for negative inputs, `f64::NEG_INFINITY` for `0.0`,
/// and `f64::NAN` for `f64::NAN`.
///
/// # Examples
///
/// ```
/// use tpt_zero_bayes::math::ln;
///
/// assert!(ln(1.0).abs() < 1e-12);
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

pub mod gamma {
    //! Gamma-function helpers for the conjugate-prior densities.
    //!
    //! Natural log of the Gamma function via the Lanczos approximation, plus a
    //! direct `gamma(x)` for callers who need the unlogged value.

    use super::{exp, ln};

    /// Coefficients `g = 7`, `n = 9` Lanczos approximation.
    const G: f64 = 7.0;
    const C: [f64; 9] = [
        0.999_999_999_999_809_9,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_295,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311e-7,
    ];

    /// Returns `ln(Gamma(x))` for `x > 0`.
    ///
    /// For non-positive integer `x` (poles) this returns `f64::INFINITY`.
    /// Non-finite input returns `f64::NAN`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_bayes::math::gamma::ln_gamma;
    ///
    /// // Gamma(n) = (n-1)!, so ln(Gamma(5)) = ln(24).
    /// assert!((ln_gamma(5.0) - 24.0f64.ln()).abs() < 1e-9);
    /// ```
    #[must_use]
    pub fn ln_gamma(x: f64) -> f64 {
        if !x.is_finite() {
            return f64::NAN;
        }
        if x < 0.5 {
            // Reflection formula: ln(Gamma(z)) = ln(pi) - ln(Gamma(1-z)) - ln(sin(pi z)).
            let sin_pi_x = sin_pi(x);
            if sin_pi_x == 0.0 {
                return f64::INFINITY;
            }
            let reflect = ln_gamma(1.0 - x);
            return ln(core::f64::consts::PI) - ln(sin_pi_x.abs()) - reflect;
        }

        let z = x - 1.0;
        let mut acc = C[0];
        for (i, &c) in C.iter().enumerate().take(9).skip(1) {
            acc += c / (z + (i as f64));
        }
        let t = z + G + 0.5;
        // sqrt(2*pi) normalization.
        let two_pi = 2.0 * core::f64::consts::PI;
        0.5 * ln(two_pi) + (z + 0.5) * ln(t) - t + ln(acc)
    }

    /// Returns `Gamma(x)` for `x > 0`.
    ///
    /// Implementation of `exp(ln_gamma(x))`; see [`ln_gamma`] for edge
    /// behaviour (poles return `f64::INFINITY`, non-finite returns `NAN`).
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_bayes::math::gamma::gamma;
    ///
    /// // Gamma(5) = 4! = 24.
    /// assert!((gamma(5.0) - 24.0).abs() < 1e-7);
    /// ```
    #[must_use]
    pub fn gamma(x: f64) -> f64 {
        exp(ln_gamma(x))
    }

    /// Sine of `pi * x`, accurate enough for the reflection formula.
    ///
    /// Implemented via range reduction and the Taylor series around 0, since
    /// `f64::sin` is unavailable in this `core`-only crate.
    fn sin_pi(x: f64) -> f64 {
        // Reduce to angle a = pi * frac(x) in [0, 1), then a*sin depends on quadrant.
        let frac = x - floor_local(x);
        let a = frac * core::f64::consts::PI;
        // Fold a into [0, pi] then reflect to [0, pi/2] for the series.
        let a = if a > core::f64::consts::PI {
            2.0 * core::f64::consts::PI - a
        } else {
            a
        };
        let a2 = a * a;
        let mut term = a;
        let mut sum = a;
        let mut sign = -1.0;
        let mut m = 3.0;
        for _ in 0..14 {
            term *= a2 / (m * (m - 1.0));
            sum += sign * term;
            sign = -sign;
            m += 2.0;
        }
        sum
    }

    /// Local floor used by the reflection formula (avoids importing the parent
    /// `floor`, which is also fine but keeps this module self-contained).
    fn floor_local(x: f64) -> f64 {
        if !x.is_finite() {
            return x;
        }
        let t = x as i64 as f64;
        if t > x {
            t - 1.0
        } else {
            t
        }
    }

    #[cfg(test)]
    mod tests {
    #![allow(clippy::float_cmp, clippy::cast_lossless)]
        use super::*;

        #[test]
        fn ln_gamma_factorials() {
            // Gamma(n) = (n-1)!, so ln(Gamma(n)) = ln((n-1)!).
            let mut fact = 1.0f64;
            for n in 1..=10 {
                assert!(
                    (ln_gamma(n as f64) - fact.ln()).abs() < 1e-7 * (1.0 + fact),
                    "ln_gamma({n})"
                );
                fact *= n as f64;
            }
        }

        #[test]
        fn gamma_factorials() {
            assert!((gamma(5.0) - 24.0).abs() < 1e-6);
            assert!((gamma(1.0) - 1.0).abs() < 1e-9);
            assert!((gamma(0.5) - core::f64::consts::PI.sqrt()).abs() < 1e-7);
        }

        #[test]
        fn ln_gamma_half() {
            // Gamma(1/2) = sqrt(pi).
            assert!((ln_gamma(0.5) - 0.5 * core::f64::consts::PI.ln()).abs() < 1e-9);
        }
    }
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
            let want = std::f64::consts::SQRT_2 * 0.0 + v.sqrt();
            assert!((got - want).abs() < 1e-9 * (1.0 + want), "sqrt({v}) = {got}, want {want}");
        }
        assert!(sqrt(-1.0).is_nan());
        assert!(sqrt(f64::NAN).is_nan());
    }

    #[test]
    fn exp_matches_reference() {
        for &v in &[-5.0, -1.0, 0.0, 0.5, 1.0, 2.5, 10.0] {
            let got = exp(v);
            let want = core::f64::consts::E.powf(v);
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
}
