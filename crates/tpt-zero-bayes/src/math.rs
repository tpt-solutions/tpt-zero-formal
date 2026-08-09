//! Local floating-point helpers for this `core`-only crate.
//!
//! `f64::sqrt`, `f64::ln`, and `f64::exp` are not available in every
//! `core`-only target, so the helpers here delegate to
//! [`tpt_zero_float`] (a subnormal-safe, `no_std` implementation) while the
//! gamma function and floor remain implemented locally.

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
/// use tpt_zero_bayes::math::sqrt;
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
/// use tpt_zero_bayes::math::exp;
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
/// use tpt_zero_bayes::math::ln;
///
/// assert!(ln(1.0).abs() < 1e-12);
/// assert!((ln(core::f64::consts::E) - 1.0).abs() < 1e-12);
/// assert!(ln(-1.0).is_nan());
/// ```
#[must_use]
pub fn ln(x: f64) -> f64 {
    tpt_zero_float::ln(x)
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
