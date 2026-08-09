#![doc = include_str!("../README.md")]

#![no_std]
#![warn(missing_docs)]
#![forbid(unsafe_code)]
// This crate is low-level floating-point bit-twiddling: the casts below are
// inherent to IEEE-754 manipulation and are each individually safe (the source
// ranges are documented at each site).
#![allow(
    clippy::cast_lossless,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::unreadable_literal,
    clippy::float_cmp,
    clippy::float_equality_without_abs,
    clippy::approx_constant
)]

const LN_2: f64 = core::f64::consts::LN_2;
const LOG2_E: f64 = core::f64::consts::LOG2_E;
/// Largest finite `exp` input before overflow to `+inf`.
const EXP_MAX: f64 = 709.0;
/// Smallest `exp` input before underflow to `0.0` (below the smallest
/// subnormal, `2^-1074`).
const EXP_MIN: f64 = -745.0;
/// Convergence thresholds for the series expansions.
const TAYLOR_EPS: f64 = 1e-16;

/// Computes the square root of `x >= 0.0` via a bit-manipulation initial
/// guess followed by Newton's method with a *relative* tolerance.
///
/// Returns `f64::NAN` for negative or `NaN` input, `0.0` for `0.0`, and
/// `+inf` for `+inf`. Works correctly for both tiny (`1e-30`) and huge
/// (`1e300`) magnitudes, including the subnormal range, unlike an
/// absolute-tolerance iteration that stalls for `x < 1e-12`.
///
/// # Examples
///
/// ```
/// use tpt_zero_float::sqrt;
///
/// assert_eq!(sqrt(0.0), 0.0);
/// assert_eq!(sqrt(4.0), 2.0);
/// assert!((sqrt(1e-30) - 1e-15).abs() < 1e-25);
/// assert!((sqrt(1e300) - 1e150).abs() < 1e135);
/// ```
#[must_use]
pub fn sqrt(x: f64) -> f64 {
    if x.is_nan() || x < 0.0 {
        return f64::NAN;
    }
    if x == 0.0 {
        return 0.0;
    }
    if x.is_infinite() {
        return f64::INFINITY;
    }

    // Initial guess from the exponent bits (within a factor of ~2 of the
    // true root). Newton converges quadratically from here.
    let bits = x.to_bits();
    let exp = ((bits >> 52) & 0x7FF) as i64;
    let mant = bits & 0x000F_FFFF_FFFF_FFFF;
    let new_exp = (((exp - 1023) >> 1) + 1023) as u64;
    let new_mant = mant >> 1;
    let mut guess = f64::from_bits((new_exp << 52) | new_mant);

    for _ in 0..64 {
        let next = 0.5 * (guess + x / guess);
        if (next - guess).abs() <= guess.abs() * TAYLOR_EPS {
            guess = next;
            break;
        }
        guess = next;
    }
    guess
}

/// Computes `2^n` for an integer `n`, correctly producing subnormals for
/// `n <= -1023` and clamping to `0.0`/`+inf` outside the representable
/// range.
fn pow2(n: i32) -> f64 {
    if n >= 1024 {
        return f64::INFINITY;
    }
    if n >= -1022 {
        // Normal range: exponent field is `1023 + n`.
        f64::from_bits(((1023i64 + n as i64) as u64) << 52)
    } else if n >= -1074 {
        // Subnormal range: the value is encoded entirely in the mantissa.
        f64::from_bits(1u64 << (n as i64 + 1074))
    } else {
        0.0
    }
}

/// Taylor series for `e^r` where `|r| <= ln(2)/2` (so it converges fast).
fn exp_taylor(r: f64) -> f64 {
    let mut term = 1.0;
    let mut sum = 1.0;
    let mut i = 1;
    loop {
        term *= r / (i as f64);
        sum += term;
        if term.abs() < sum.abs() * TAYLOR_EPS {
            break;
        }
        i += 1;
        if i > 200 {
            break;
        }
    }
    sum
}

/// Computes `e^x`.
///
/// Reduction splits `x = n*ln(2) + r` with integer `n` and `|r| <= ln(2)/2`;
/// the result is `2^n * e^r`. This handles the full range, including the
/// subnormal underflow region that naive exponent-bit reconstruction gets
/// wrong, and never returns a negative value for large negative `x`.
///
/// # Examples
///
/// ```
/// use tpt_zero_float::exp;
///
/// assert_eq!(exp(0.0), 1.0);
/// assert!((exp(1.0) - core::f64::consts::E).abs() < 1e-12);
/// assert_eq!(exp(-745.0), 0.0);
/// assert_eq!(exp(800.0), f64::INFINITY);
/// ```
#[must_use]
pub fn exp(x: f64) -> f64 {
    if x.is_nan() {
        return x;
    }
    if x == 0.0 {
        return 1.0;
    }
    if x > EXP_MAX {
        return f64::INFINITY;
    }
    if x < EXP_MIN {
        return 0.0;
    }
    let n = floor_f64(x * LOG2_E) as i32;
    let r = x - (n as f64) * LN_2;
    pow2(n) * exp_taylor(r)
}

/// `no_std`-safe floor: returns the largest integer `<= x`. `f64::floor` is a
/// `std`-only method, so we implement it via bit manipulation.
fn floor_f64(x: f64) -> f64 {
    let bits = x.to_bits();
    let exp = ((bits >> 52) & 0x7FF) as i64;
    // Already an integer (|x| >= 2^52) or NaN/inf.
    if exp >= 1023 + 52 {
        return x;
    }
    if exp < 1023 {
        // |x| < 1.
        if x == 0.0 {
            return 0.0;
        }
        return if x < 0.0 { -1.0 } else { 0.0 };
    }
    let frac_bits = (52 - (exp - 1023)) as u32;
    let mask = (1u64 << frac_bits) - 1;
    let integer_part = bits & !mask;
    let floored = f64::from_bits(integer_part);
    if x < 0.0 && (bits & mask) != 0 {
        floored - 1.0
    } else {
        floored
    }
}

/// Decomposes `x` into a normalized mantissa `m in [1, 2)` and an integer
/// exponent `e` such that `x = m * 2^e`. Correct for subnormals.
fn frexp(x: f64) -> (f64, i32) {
    if x == 0.0 {
        return (0.0, 0);
    }
    let bits = x.to_bits();
    let exp_field = (bits >> 52) & 0x7FF;
    if exp_field != 0 {
        // Normal: reconstruct mantissa in [1, 2) and exponent.
        let m = f64::from_bits((bits & 0x800F_FFFF_FFFF_FFFF) | 0x3FF0_0000_0000_0000);
        (m, (exp_field as i32) - 1023)
    } else {
        // Subnormal: scale into the normal range and recurse, then fix up the
        // exponent. Scaling by 2^1023 keeps every subnormal in (0, 2) without
        // overflowing.
        let scaled = x * f64::from_bits(0x7FE0_0000_0000_0000);
        let (m, e) = frexp(scaled);
        (m, e - 1023)
    }
}

/// Inverse-hyperbolic-tangent series for `z` with `|z| < 1`.
fn atanh_series(z: f64) -> f64 {
    let mut sum = z;
    let mut power = z * z * z;
    let z2 = z * z;
    let mut k = 3;
    loop {
        let term = power / (k as f64);
        sum += term;
        if term.abs() < sum.abs() * TAYLOR_EPS {
            break;
        }
        power *= z2;
        k += 2;
        if k > 400 {
            break;
        }
    }
    sum
}

/// Computes the natural logarithm `ln(x)`.
///
/// `x` is split into `m * 2^e` with `m in [1, 2)`; then `ln(x) = e*ln(2) +
/// ln(m)`, where `ln(m)` is evaluated with the `atanh` series on
/// `z = (m - 1)/(m + 1)`. Returns `NaN` for negative input, `-inf` for `0`,
/// and `+inf` for `+inf`.
///
/// # Examples
///
/// ```
/// use tpt_zero_float::ln;
///
/// assert_eq!(ln(1.0), 0.0);
/// assert!((ln(core::f64::consts::E) - 1.0).abs() < 1e-12);
/// assert!((ln(10.0) - 2.302585093).abs() < 1e-9);
/// assert_eq!(ln(0.0), f64::NEG_INFINITY);
/// ```
#[must_use]
#[allow(clippy::float_cmp)]
pub fn ln(x: f64) -> f64 {
    if x < 0.0 {
        return f64::NAN;
    }
    if x == 0.0 {
        return f64::NEG_INFINITY;
    }
    if x.is_infinite() {
        return f64::INFINITY;
    }
    if x == 1.0 {
        return 0.0;
    }
    let (m, e) = frexp(x);
    let z = (m - 1.0) / (m + 1.0);
    let ln_m = 2.0 * atanh_series(z);
    (e as f64) * LN_2 + ln_m
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;

    // Reference implementations from std, used only in tests.
    use std::f64::consts::{E, LN_2 as STD_LN_2, PI, SQRT_2};

    fn rel(a: f64, b: f64) -> f64 {
        (a - b).abs() / b.abs().max(1.0)
    }

    #[test]
    fn sqrt_basics() {
        assert_eq!(sqrt(0.0), 0.0);
        assert_eq!(sqrt(4.0), 2.0);
        assert_eq!(sqrt(1.0), 1.0);
        assert!((sqrt(2.0) - SQRT_2).abs() < 1e-12);
    }

    #[test]
    fn sqrt_extremes() {
        // Subnormal and huge magnitudes were wrong in the hand-rolled copies.
        assert!((sqrt(1e-30) - 1e-15).abs() < 1e-25);
        assert!((sqrt(1e-300) - 1e-150).abs() < 1e-145);
        assert!((sqrt(1e100) - 1e50).abs() < 1e40);
        assert!((sqrt(1e300) - 1e150).abs() < 1e135);
        assert!((sqrt(25.0) - 5.0).abs() < 1e-12);
    }

    #[test]
    fn exp_basics() {
        assert_eq!(exp(0.0), 1.0);
        assert!((exp(1.0) - E).abs() < 1e-12);
        assert!((exp(-1.0) - 1.0 / E).abs() < 1e-12);
    }

    #[test]
    fn exp_subnormal_and_overflow() {
        // These returned 0 / negative / wrong values in the old copies.
        assert!((exp(-709.0)).is_finite());
        assert!(exp(-720.0) > 0.0);
        assert_eq!(exp(-800.0), 0.0);
        assert_eq!(exp(800.0), f64::INFINITY);
        assert!(exp(709.0).is_finite());
    }

    #[test]
    fn ln_basics() {
        assert_eq!(ln(1.0), 0.0);
        assert!((ln(E) - 1.0).abs() < 1e-12);
        assert!((ln(10.0) - 2.302_585_092_994_046).abs() < 1e-12);
        assert!((ln(0.5) + LN_2).abs() < 1e-12);
        assert_eq!(ln(0.0), f64::NEG_INFINITY);
        assert!(ln(-1.0).is_nan());
    }

    #[test]
    fn accuracy_vs_std_over_sweep() {
        for i in 0..2000u32 {
            let x = (i as f64) * 0.01 - 10.0;
            assert!(rel(sqrt(x * x), x.abs()) < 1e-12, "sqrt at {x}");
            if x > EXP_MIN && x < EXP_MAX {
                assert!(rel(exp(x), x.exp()) < 1e-12, "exp at {x}");
            }
            if x > 0.0 {
                assert!(rel(ln(x), x.ln()) < 1e-12, "ln at {x}");
            }
        }
        // Tiny and huge magnitudes.
        for &x in &[1e-30, 1e-300, 1e100, 1e300] {
            assert!(rel(sqrt(x), x.sqrt()) < 1e-12, "sqrt huge {x}");
            assert!(rel(ln(x), x.ln()) < 1e-12, "ln huge {x}");
        }
        // Large exp inputs (finite region).
        for &x in &[700.0, 709.0, -700.0, -744.0] {
            assert!(rel(exp(x), x.exp()) < 1e-11, "exp large {x}");
        }
        // A few irrational/pi checks.
        assert!(rel(ln(PI), PI.ln()) < 1e-12);
        assert!(rel(exp(PI), PI.exp()) < 1e-12);
        let _ = STD_LN_2;
    }
}
