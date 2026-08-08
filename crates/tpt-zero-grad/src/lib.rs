//! Forward-mode automatic differentiation via dual numbers for `no_std`, with
//! zero external production dependencies. Part of the
//! [tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal)
//! ecosystem.
//!
//! A [`Dual<T>`] carries a primal value together with its derivative with
//! respect to a single independent variable (the "seed"). Arithmetic operators
//! are overloaded with the standard dual-number rules, so evaluating any
//! differentiable expression composed from them also produces its derivative
//! for free. [`grad`] seeds the derivative to `1.0` and returns the
//! derivative part at a given point.
//!
//! ```
//! use tpt_zero_grad::{grad, Dual};
//!
//! // d/dx x^2 = 2x
//! let d = grad(|x| x * x, 3.0);
//! assert!((d - 6.0).abs() < 1e-12);
//!
//! // d/dx sin(x) = cos(x)
//! let d = grad(Dual::sin, 1.0);
//! assert!((d - 1.0f64.cos()).abs() < 1e-12);
//! ```
//!
//! The crate is `#[no_std]` and const-generic friendly: `Dual<T>` works for any
//! `T: Copy + the usual arithmetic`, and gradient-based operations over the
//! fixed-size [`Tensor`](tpt_zero_tensor::Tensor)s from `tpt-zero-tensor` are
//! provided through [`grad_tensor`].

#![no_std]
#![warn(missing_docs)]
#![forbid(unsafe_code)]
// The float primitives used by a normal `core` build (notably `f64::sin`,
// `f64::cos`, `f64::exp`, and `f64::ln`) are unavailable in this crate's target
// configuration, so we provide our own implementations in `float_fallback`.
// Those helpers convert between `usize`/`i64` and `f64` on counts that can
// never be large enough to lose information, so the associated cast lints are
// allowed.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    // Single-letter names (`i`, `k`, `n`, `s`, `c`, ...) are idiomatic for the
    // math in `float_fallback` (series indices, accumulators), and the
    // transcendental constants are clearer without digit-group separators.
    clippy::many_single_char_names,
    clippy::unreadable_literal
)]

use core::ops::{Add, Div, Mul, Neg, Sub};

/// Marker trait for the numeric types usable as a dual number's scalar,
/// providing the additive and multiplicative identities.
///
/// Implemented for the floating-point types this crate targets. The
/// identities are the actual `0` and `1` literals, which are the only values
/// needed to seed and zero derivatives.
pub trait DualNum: Copy {
    /// The additive identity (`0`).
    const ZERO: Self;
    /// The multiplicative identity (`1`).
    const ONE: Self;
}

impl DualNum for f64 {
    const ZERO: f64 = 0.0;
    const ONE: f64 = 1.0;
}

impl DualNum for f32 {
    const ZERO: f32 = 0.0;
    const ONE: f32 = 1.0;
}

/// A dual number carrying a primal value `value` and its derivative `deriv`
/// with respect to a single independent variable.
///
/// Evaluating ordinary arithmetic on `Dual`s composes the standard
/// dual-number rules, so the `deriv` field tracks `d(value)/dx` automatically.
/// For example, given `x = Dual::seed(3.0)` (value `3`, derivative `1`),
/// `x * x` yields `Dual { value: 9, deriv: 6 }`, i.e. `d/dx x² = 2x` at `x = 3`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dual<T> {
    /// The primal (ordinary) value of the quantity.
    pub value: T,
    /// The derivative of the quantity with respect to the independent variable.
    pub deriv: T,
}

impl<T> Dual<T> {
    /// Constructs a dual number from its primal `value` and `deriv`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_grad::Dual;
    ///
    /// let d = Dual::new(3.0, 6.0);
    /// assert_eq!(d.value, 3.0);
    /// assert_eq!(d.deriv, 6.0);
    /// ```
    #[must_use]
    pub const fn new(value: T, deriv: T) -> Self {
        Self { value, deriv }
    }
}

impl<T: DualNum> Dual<T> {
    /// Creates a constant dual number: a value with zero derivative.
    ///
    /// Constants do not depend on the independent variable, so their derivative
    /// is `0`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_grad::Dual;
    ///
    /// let c = Dual::constant(5.0);
    /// assert_eq!(c.value, 5.0);
    /// assert_eq!(c.deriv, 0.0);
    /// ```
    #[must_use]
    pub const fn constant(value: T) -> Self {
        Self {
            value,
            deriv: T::ZERO,
        }
    }

    /// Creates a seed dual number: the independent variable itself, with
    /// derivative `1`.
    ///
    /// This is the starting point for forward-mode differentiation; every other
    /// quantity's derivative is computed relative to this seed.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_grad::Dual;
    ///
    /// let x = Dual::seed(2.0);
    /// assert_eq!(x.value, 2.0);
    /// assert_eq!(x.deriv, 1.0);
    /// ```
    #[must_use]
    pub const fn seed(value: T) -> Self {
        Self {
            value,
            deriv: T::ONE,
        }
    }
}

impl<T: Add<Output = T> + Copy> Add for Dual<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value + rhs.value,
            deriv: self.deriv + rhs.deriv,
        }
    }
}

impl<T: Sub<Output = T> + Copy> Sub for Dual<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value - rhs.value,
            deriv: self.deriv - rhs.deriv,
        }
    }
}

impl<T: Add<Output = T> + Mul<Output = T> + Sub<Output = T> + Copy> Mul for Dual<T> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        // d(uv) = u'v + uv'
        Self {
            value: self.value * rhs.value,
            deriv: self.deriv * rhs.value + self.value * rhs.deriv,
        }
    }
}

impl<T: Mul<Output = T> + Sub<Output = T> + Div<Output = T> + Copy> Div for Dual<T> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        // d(u/v) = (u'v - uv') / v^2
        let denom = rhs.value * rhs.value;
        Self {
            value: self.value / rhs.value,
            deriv: (self.deriv * rhs.value - self.value * rhs.deriv) / denom,
        }
    }
}

impl<T: Neg<Output = T>> Neg for Dual<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            value: -self.value,
            deriv: -self.deriv,
        }
    }
}

impl<T: Copy> Dual<T> {
    /// Lifts a pure function on primal values into one that threads a dual
    /// derivative, given the derivative of the function.
    ///
    /// This is the primitive used to build transcendental operations: `df` must
    /// return `g'(x)` for the function `g` being applied. The chain rule
    /// `d(g(u)) = g'(u) * u'` is applied automatically.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_grad::Dual;
    ///
    /// // Square: g(x) = x^2, g'(x) = 2x
    /// let x = Dual::seed(3.0);
    /// let y = x.lift(|v| v * v, |v| 2.0 * v);
    /// assert_eq!(y.value, 9.0);
    /// assert_eq!(y.deriv, 6.0);
    /// ```
    #[must_use]
    pub fn lift<U: DualNum + Mul<T, Output = U>>(
        self,
        f: impl Fn(T) -> U,
        df: impl Fn(T) -> U,
    ) -> Dual<U> {
        Dual {
            value: f(self.value),
            // Chain rule: d(g(u)) = g'(u) * u'.
            deriv: df(self.value) * self.deriv,
        }
    }
}

impl Dual<f64> {
    /// Returns the natural exponential `e^value`, with derivative
    /// `e^value * deriv`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_grad::{grad, Dual};
    ///
    /// // d/dx e^x = e^x
    /// let d = grad(Dual::exp, 2.0);
    /// assert!((d - 2.0f64.exp()).abs() < 1e-9);
    /// ```
    #[must_use]
    pub fn exp(self) -> Self {
        let e = float_fallback::exp(self.value);
        Self {
            value: e,
            deriv: e * self.deriv,
        }
    }

    /// Returns the natural logarithm `ln(value)`, with derivative
    /// `deriv / value`.
    ///
    /// Produces `NaN` for non-positive inputs, matching `f64::ln`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_grad::Dual;
    ///
    /// // d/dx ln(x) = 1/x
    /// let x = Dual::seed(4.0);
    /// let y = x.ln();
    /// assert!((y.deriv - 0.25).abs() < 1e-12);
    /// ```
    #[must_use]
    pub fn ln(self) -> Self {
        Self {
            value: float_fallback::ln(self.value),
            deriv: self.deriv / self.value,
        }
    }

    /// Returns the sine of `value`, with derivative `cos(value) * deriv`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_grad::Dual;
    ///
    /// // d/dx sin(x) = cos(x)
    /// let x = Dual::seed(1.0);
    /// let y = x.sin();
    /// assert!((y.deriv - 1.0f64.cos()).abs() < 1e-9);
    /// ```
    #[must_use]
    pub fn sin(self) -> Self {
        Self {
            value: float_fallback::sin(self.value),
            deriv: float_fallback::cos(self.value) * self.deriv,
        }
    }

    /// Returns the cosine of `value`, with derivative `-sin(value) * deriv`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_grad::Dual;
    ///
    /// // d/dx cos(x) = -sin(x)
    /// let x = Dual::seed(1.0);
    /// let y = x.cos();
    /// assert!((y.deriv + 1.0f64.sin()).abs() < 1e-9);
    /// ```
    #[must_use]
    pub fn cos(self) -> Self {
        Self {
            value: float_fallback::cos(self.value),
            deriv: -float_fallback::sin(self.value) * self.deriv,
        }
    }

    /// Raises `value` to the integer power `n` using repeated squaring of the
    /// dual, correctly tracking the derivative via the dual power rule.
    ///
    /// `n` may be negative; for `n == 0` the result is the constant `1`. The
    /// derivative is `n * value^(n-1) * deriv`, evaluated in dual arithmetic so
    /// intermediate products chain correctly.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_grad::Dual;
    ///
    /// // d/dx x^3 = 3 x^2
    /// let x = Dual::seed(2.0);
    /// let y = x.powi(3);
    /// assert_eq!(y.value, 8.0);
    /// assert_eq!(y.deriv, 12.0);
    /// ```
    #[must_use]
    pub fn powi(self, n: i32) -> Self {
        let mut result = Dual::new(1.0, 0.0);
        let mut base = self;
        let negative = n < 0;
        let mut e = if negative { -n } else { n };
        while e > 0 {
            if e % 2 == 1 {
                result = result * base;
            }
            base = base * base;
            e /= 2;
        }
        if negative {
            Dual::new(1.0, 0.0) / result
        } else {
            result
        }
    }
}

/// Computes the derivative `df/dx` at `x` of the dual-valued function `f`.
///
/// The independent variable is seeded with derivative `1.0`, the function is
/// evaluated, and the resulting `deriv` field is returned. `f` is a closure
/// taking a [`Dual<f64>`] (the seed) and returning another `Dual<f64>`.
///
/// # Examples
///
/// ```
/// use tpt_zero_grad::{grad, Dual};
///
/// // d/dx (x^2 + 3x) = 2x + 3
/// let d = grad(|x| x * x + Dual::constant(3.0) * x, 2.0);
/// assert!((d - 7.0).abs() < 1e-12);
/// ```
///
/// ```
/// use tpt_zero_grad::{grad, Dual};
///
/// // Chain rule: d/dx sin(x^2) = 2x cos(x^2)
/// let d = grad(|x| (x * x).sin(), 2.0);
/// assert!((d - 4.0 * (4.0f64).cos()).abs() < 1e-9);
/// ```
#[must_use]
pub fn grad<F>(f: F, x: f64) -> f64
where
    F: Fn(Dual<f64>) -> Dual<f64>,
{
    let seed = Dual::seed(x);
    let y = f(seed);
    y.deriv
}

/// Computes the gradient of a scalar function of an `N`-dimensional vector
/// argument, returned as a [`Tensor`](tpt_zero_tensor::Tensor) of partial
/// derivatives.
///
/// Each input coordinate is seeded with derivative `1.0` in turn; the others
/// carry derivative `0`. This is the forward-mode Jacobian for a scalar output,
/// accumulated componentwise.
///
/// # Examples
///
/// ```
/// use tpt_zero_grad::grad_tensor;
/// use tpt_zero_tensor::Tensor;
///
/// // f(v) = v[0]^2 + v[0]*v[1]; partials: [2 v[0] + v[1], v[0]]
/// let v = Tensor::from([3.0, 5.0]);
/// let g = grad_tensor(|x: Tensor<tpt_zero_grad::Dual<f64>, 2>| {
///     x[0] * x[0] + x[0] * x[1]
/// }, &v);
/// assert!((g[0] - 11.0).abs() < 1e-12);
/// assert!((g[1] - 3.0).abs() < 1e-12);
/// ```
#[must_use]
pub fn grad_tensor<const N: usize>(
    f: impl Fn(tpt_zero_tensor::Tensor<Dual<f64>, N>) -> Dual<f64>,
    x: &tpt_zero_tensor::Tensor<f64, N>,
) -> tpt_zero_tensor::Tensor<f64, N> {
    let mut grads = [0.0f64; N];
    let mut i = 0;
    while i < N {
        let dual = tpt_zero_tensor::Tensor::new(core::array::from_fn(|j| {
            if j == i {
                Dual::seed(x[j])
            } else {
                Dual::constant(x[j])
            }
        }));
        grads[i] = f(dual).deriv;
        i += 1;
    }
    tpt_zero_tensor::Tensor::new(grads)
}

/// Float implementations that never rely on `f64` intrinsics unavailable in
/// some `core`-only targets. Used by [`Dual<f64>`]'s transcendental methods.
mod float_fallback {
    use core::f64::consts::{FRAC_PI_2, PI, TAU};

    /// Returns `e^x`.
    ///
    /// Delegates to the shared, subnormal-safe [`out_zero_float`] implementation,
    /// which handles the full range (including the subnormal underflow region) and
    /// never returns a negative value for large negative `x`.
    #[must_use]
    pub fn exp(x: f64) -> f64 {
        out_zero_float::exp(x)
    }

    /// Returns the natural logarithm `ln(x)`.
    ///
    /// Delegates to the shared, subnormal-safe [`out_zero_float`] implementation,
    /// which is accurate for subnormal magnitudes and returns `-inf` for `0`.
    /// Returns `NaN` for `x <= 0`.
    #[must_use]
    pub fn ln(x: f64) -> f64 {
        out_zero_float::ln(x)
    }

    /// Returns `sin(x)` via argument reduction into `[-π/2, π/2]` and a Taylor
    /// series. `cos` is derived from the same reduction using an offset of
    /// `π/2`.
    #[must_use]
    pub fn sin(x: f64) -> f64 {
        sin_cos_reduced(x).0
    }

    /// Returns `cos(x)`.
    #[must_use]
    pub fn cos(x: f64) -> f64 {
        sin_cos_reduced(x).1
    }

    /// Reduces `x` into `[-π, π]` and returns `(sin x, cos x)` from Taylor
    /// series, handling quadrant sign/cosine via the periodic symmetry of
    /// sine/cosine.
    #[must_use]
    fn sin_cos_reduced(x: f64) -> (f64, f64) {
        let twopi = TAU;
        let pi = PI;
        let pi2 = FRAC_PI_2;

        // Normalize into [-pi, pi].
        let mut a = if x.is_nan() {
            return (f64::NAN, f64::NAN);
        } else {
            x - twopi * round_f64(x / twopi)
        };

        // sin/cos are odd/even respectively, so fold to [0, pi].
        let neg = a < 0.0;
        a = a.abs();

        // Reduce further to [0, pi/2] using sin(pi - a) = sin a,
        // cos(pi - a) = -cos a.
        let flip_cos = a > pi2;
        let a = if flip_cos { pi - a } else { a };

        // sin(a) = a - a^3/3! + a^5/5! - ... (degree 27).
        let mut s_term = a;
        let mut s = a;
        let mut n = 3i64;
        while n <= 27 {
            s_term *= -a * a / ((n - 1) as f64 * n as f64);
            s += s_term;
            n += 2;
        }

        // cos(a) = 1 - a^2/2! + a^4/4! - ... (degree 26).
        let a2 = a * a;
        let mut c_term = 1.0;
        let mut c = 1.0;
        let mut m = 2i64;
        while m <= 26 {
            c_term *= -a2 / ((m - 1) as f64 * m as f64);
            c += c_term;
            m += 2;
        }

        let (s, c) = if flip_cos { (s, -c) } else { (s, c) };
        let (s, c) = if neg { (-s, c) } else { (s, c) };
        (s, c)
    }

    /// Rounds `x` to the nearest integer, ties away from zero. Provided because
    /// `f64::round` is not guaranteed available in every `core`-only target.
    #[must_use]
    fn round_f64(x: f64) -> f64 {
        let trunc = (x as i64) as f64;
        let frac = x - trunc;
        let floor = if frac < 0.0 { trunc - 1.0 } else { trunc };
        let frac_from_floor = x - floor;
        if frac_from_floor < 0.5 {
            floor
        } else if frac_from_floor > 0.5 {
            floor + 1.0
        } else if (floor as i64) % 2 == 0 {
            // Exact half: bias away from zero.
            if floor < 0.0 { floor - 1.0 } else { floor + 1.0 }
        } else {
            floor
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grad_of_x_squared_is_2x() {
        let d = grad(|x| x * x, 5.0);
        assert!((d - 10.0).abs() < 1e-12);
    }

    #[test]
    fn grad_of_polynomial() {
        // f(x) = 3x^3 - 2x + 1  =>  f'(x) = 9x^2 - 2
        let d = grad(
            |x| {
                let x3 = x * x * x;
                x3 + x3 + x3 - x - x + Dual::constant(1.0)
            },
            2.0,
        );
        assert!((d - (9.0 * 4.0 - 2.0)).abs() < 1e-9);
    }

    #[test]
    fn grad_of_sin_is_cos() {
        for &x in &[0.0, 0.5, 1.0, 2.0, -1.5] {
            let d = grad(Dual::sin, x);
            assert!((d - float_fallback::cos(x)).abs() < 1e-9, "sin' at {x}");
        }
    }

    #[test]
    fn grad_of_cos_is_neg_sin() {
        for &x in &[0.0, 0.5, 1.0, 2.0, -1.5] {
            let d = grad(Dual::cos, x);
            assert!((d + float_fallback::sin(x)).abs() < 1e-9, "cos' at {x}");
        }
    }

    #[test]
    fn grad_of_exp_is_exp() {
        for &x in &[0.0, 1.0, 2.0, -1.0] {
            let d = grad(Dual::exp, x);
            assert!((d - float_fallback::exp(x)).abs() < 1e-9, "exp' at {x}");
        }
    }

    #[test]
    fn grad_of_ln_is_reciprocal() {
        for &x in &[0.5, 1.0, 2.0, 4.0] {
            let d = grad(Dual::ln, x);
            assert!((d - 1.0 / x).abs() < 1e-9, "ln' at {x}");
        }
    }

    #[test]
    fn chain_rule() {
        // f(x) = sin(x^2)  =>  f'(x) = 2x cos(x^2)
        for &x in &[0.0, 1.0, 2.0, -1.5] {
            let d = grad(|x| (x * x).sin(), x);
            let expected = 2.0 * x * float_fallback::cos(x * x);
            assert!((d - expected).abs() < 1e-9, "chain at {x}");
        }
    }

    #[test]
    fn powi_rule() {
        // f(x) = x^3  =>  f'(x) = 3 x^2
        let d = grad(|x| x.powi(3), 2.0);
        assert!((d - 12.0).abs() < 1e-12);
    }

    #[test]
    fn powi_negative_exponent() {
        // f(x) = x^-2  =>  f'(x) = -2 x^-3
        let d = grad(|x| x.powi(-2), 2.0);
        assert!((d - (-2.0 / 8.0)).abs() < 1e-12);
    }

    #[test]
    fn division_rule() {
        // f(x) = (x + 1) / (x - 1)  =>  f'(x) = -2/(x-1)^2
        let d = grad(
            |x| {
                let num = x + Dual::constant(1.0);
                let den = x - Dual::constant(1.0);
                num / den
            },
            3.0,
        );
        assert!((d - (-2.0 / 4.0)).abs() < 1e-12);
    }

    #[test]
    fn grad_tensor_partials() {
        let v = tpt_zero_tensor::Tensor::from([3.0, 5.0]);
        let g = grad_tensor(
            |x: tpt_zero_tensor::Tensor<Dual<f64>, 2>| x[0] * x[0] + x[0] * x[1],
            &v,
        );
        assert!((g[0] - 11.0).abs() < 1e-12);
        assert!((g[1] - 3.0).abs() < 1e-12);
    }

    #[test]
    fn float_fallback_matches_reference() {
        for &x in &[0.0, 0.25, 1.0, 2.0, 5.0, -1.0, -3.0] {
            // exp: only compare where reference is finite and safe.
            let e = float_fallback::exp(x);
            assert!((e - x.exp()).abs() < 1e-9 || e.is_nan(), "exp {x}");
            if x > 0.0 {
                let l = float_fallback::ln(x);
                assert!((l - x.ln()).abs() < 1e-9, "ln {x}");
            }
        }
        for &x in &[0.0, 0.5, 1.0, 2.0, 3.0, -1.0, -2.5] {
            assert!((float_fallback::sin(x) - x.sin()).abs() < 1e-9, "sin {x}");
            assert!((float_fallback::cos(x) - x.cos()).abs() < 1e-9, "cos {x}");
        }
    }
}

#[cfg(test)]
mod proptests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[test]
        fn grad_x_squared_equals_2x(x in -100.0..100.0) {
            let d = grad(|x| x * x, x);
            prop_assert!((d - 2.0 * x).abs() < 1e-9, "got {} expected {}", d, 2.0 * x);
        }

        #[test]
        fn grad_sin_equals_cos(x in -10.0..10.0) {
            let d = grad(Dual::sin, x);
            prop_assert!((d - float_fallback::cos(x)).abs() < 1e-6, "got {} expected {}", d, float_fallback::cos(x));
        }

        #[test]
        fn chain_rule_x_squared_inside_sin(x in -10.0..10.0) {
            let d = grad(|x| (x * x).sin(), x);
            let expected = 2.0 * x * float_fallback::cos(x * x);
            prop_assert!((d - expected).abs() < 1e-6, "got {} expected {}", d, expected);
        }

        #[test]
        fn grad_linear_is_slope(x in -100.0..100.0, m in -10.0..10.0, b in -10.0..10.0) {
            let d = grad(|x| x * Dual::constant(m) + Dual::constant(b), x);
            prop_assert!((d - m).abs() < 1e-12, "got {} expected {}", d, m);
        }

        #[test]
        fn dual_mul_rule(x in -100.0..100.0, y in -100.0..100.0) {
            let a = Dual::new(x, 1.0);
            let b = Dual::new(y, 0.0);
            let p = a * b;
            // d/dx (x*y) = y
            let diff: f64 = p.deriv - y;
            prop_assert!(diff.abs() < 1e-12_f64, "got {} expected {}", p.deriv, y);
        }
    }
}
