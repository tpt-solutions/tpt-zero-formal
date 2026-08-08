//! Linear algebra over fixed-size tensors for `no_std`, with zero external
//! production dependencies. Part of the
//! [tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal)
//! ecosystem.
//!
//! This crate operates on the compile-time-sized tensors from
//! [`tpt_zero_tensor`]: [`Tensor<T, N>`](tpt_zero_tensor::Tensor) (a length-`N`
//! vector) and [`Tensor2<T, R, C>`](tpt_zero_tensor::Tensor2) (an `R`-by-`C`
//! matrix), both backed by fixed-size arrays so no heap allocation is needed.
//!
//! It provides:
//!
//! - vector products: [`dot`] and [`cross`],
//! - vector norms: [`norm_l1`], [`norm_l2`], [`norm_max`],
//! - [`normalize`] (which returns `None` on the zero vector),
//! - matrix helpers on `Tensor2`: [`trace`], [`frobenius_norm`],
//!   [`mat_vec_mul`].
//!
//! `#[no_std]` and const-generic, the crate uses a Newton-iteration square root
//! ([`sqrt`]) so it never relies on float intrinsics that are unavailable in
//! some `core`-only targets.
//!
//! ```
//! use tpt_zero_linalg::{cross, dot, normalize, norm_l2, norm_max};
//! use tpt_zero_tensor::Tensor;
//!
//! let a = Tensor::from([1.0, 2.0, 3.0]);
//! let b = Tensor::from([4.0, 5.0, 6.0]);
//!
//! assert_eq!(dot(&a, &b), 32.0);
//! assert_eq!(cross(&a, &b).as_ref(), &[-3.0, 6.0, -3.0]);
//! assert_eq!(norm_max(&a), 3.0);
//!
//! let n = normalize(&a).unwrap();
//! assert!((norm_l2(&n) - 1.0).abs() < 1e-12);
//! ```

#![no_std]
#![warn(missing_docs)]
#![forbid(unsafe_code)]

// The float primitives used by a normal `core` build (notably `f64::sqrt` and
// the `f64`/`usize` casts in the norm and cross-product implementations) are
// unavailable in this crate's target configuration, so we provide our own
// `sqrt` and the missing float conversion helpers. Those helpers necessarily
// convert between `usize`/`i64` and `f64` on data-sized counts that are never
// large enough for the conversions to lose meaningful information, so the
// associated cast lints are allowed.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]

use tpt_zero_tensor::{Tensor, Tensor2};

/// Computes the dot (inner) product of two equal-length vectors.
///
/// The inputs must have the same length `N`; the function is generic over `N`.
///
/// # Examples
///
/// ```
/// use tpt_zero_linalg::dot;
/// use tpt_zero_tensor::Tensor;
///
/// let a = Tensor::from([1.0, 2.0, 3.0]);
/// let b = Tensor::from([4.0, 5.0, 6.0]);
/// assert_eq!(dot(&a, &b), 32.0);
/// assert_eq!(dot(&a, &a), 14.0);
/// ```
#[must_use]
pub fn dot<const N: usize>(a: &Tensor<f64, N>, b: &Tensor<f64, N>) -> f64 {
    let mut sum = 0.0;
    let mut i = 0;
    while i < N {
        sum += a[i] * b[i];
        i += 1;
    }
    sum
}

/// Computes the cross product `a × b` of two 3D vectors.
///
/// The result is a vector orthogonal to both `a` and `b`; its magnitude equals
/// the area of the parallelogram spanned by `a` and `b`.
///
/// # Examples
///
/// ```
/// use tpt_zero_linalg::cross;
/// use tpt_zero_tensor::Tensor;
///
/// let a = Tensor::from([1.0, 0.0, 0.0]);
/// let b = Tensor::from([0.0, 1.0, 0.0]);
/// assert_eq!(cross(&a, &b).as_ref(), &[0.0, 0.0, 1.0]);
/// ```
#[must_use]
pub fn cross(a: &Tensor<f64, 3>, b: &Tensor<f64, 3>) -> Tensor<f64, 3> {
    Tensor::new([
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ])
}

/// Returns the Euclidean (L2) norm of `v`: `sqrt(Σ v[i]²)`.
///
/// The result is always non-negative. Returns `0.0` for the zero vector.
///
/// # Examples
///
/// ```
/// use tpt_zero_linalg::norm_l2;
/// use tpt_zero_tensor::Tensor;
///
/// let v = Tensor::from([3.0, 4.0]);
/// assert!((norm_l2(&v) - 5.0).abs() < 1e-12);
/// ```
#[must_use]
pub fn norm_l2<const N: usize>(v: &Tensor<f64, N>) -> f64 {
    sqrt(dot(v, v))
}

/// Returns the Manhattan (L1) norm of `v`: `Σ |v[i]|`.
///
/// The result is always non-negative.
///
/// # Examples
///
/// ```
/// use tpt_zero_linalg::norm_l1;
/// use tpt_zero_tensor::Tensor;
///
/// let v = Tensor::from([-1.0, 2.0, -3.0]);
/// assert_eq!(norm_l1(&v), 6.0);
/// ```
#[must_use]
pub fn norm_l1<const N: usize>(v: &Tensor<f64, N>) -> f64 {
    let mut sum = 0.0;
    let mut i = 0;
    while i < N {
        sum += v[i].abs();
        i += 1;
    }
    sum
}

/// Returns the maximum-norm (L∞) of `v`: `max_i |v[i]|`.
///
/// The result is always non-negative, and equals `0.0` only for the zero
/// vector.
///
/// # Examples
///
/// ```
/// use tpt_zero_linalg::norm_max;
/// use tpt_zero_tensor::Tensor;
///
/// let v = Tensor::from([-1.0, 5.0, -3.0]);
/// assert_eq!(norm_max(&v), 5.0);
/// ```
#[must_use]
pub fn norm_max<const N: usize>(v: &Tensor<f64, N>) -> f64 {
    let mut m = 0.0;
    let mut i = 0;
    while i < N {
        let a = v[i].abs();
        if a > m {
            m = a;
        }
        i += 1;
    }
    m
}

/// Returns `v` scaled to unit length (`v / ‖v‖`), or `None` if `v` is the zero
/// vector (which has no well-defined direction).
///
/// The returned vector satisfies `norm_l2(&result) == 1.0` whenever it is
/// `Some`.
///
/// # Examples
///
/// ```
/// use tpt_zero_linalg::{normalize, norm_l2};
/// use tpt_zero_tensor::Tensor;
///
/// let v = Tensor::from([3.0, 4.0]);
/// let n = normalize(&v).unwrap();
/// assert!((norm_l2(&n) - 1.0).abs() < 1e-12);
///
/// let zero = Tensor::from([0.0, 0.0]);
/// assert_eq!(normalize(&zero), None);
/// ```
#[must_use]
pub fn normalize<const N: usize>(v: &Tensor<f64, N>) -> Option<Tensor<f64, N>> {
    let len = norm_l2(v);
    if len == 0.0 {
        return None;
    }
    Some(Tensor::new(core::array::from_fn(|i| v[i] / len)))
}

/// Returns the trace of a square matrix: the sum of its diagonal entries.
///
/// Panics if the matrix is not square at runtime (i.e. `R != C`); this crate
/// requires a square matrix by contract, and a non-square call is a programming
/// error. The compile-time dimensions are checked by the caller.
///
/// # Examples
///
/// ```
/// use tpt_zero_linalg::trace;
/// use tpt_zero_tensor::Tensor2;
///
/// let m = Tensor2::from([[1.0, 2.0], [3.0, 4.0]]);
/// assert_eq!(trace(&m), 5.0);
/// ```
///
/// # Panics
///
/// Panics if `R != C`.
#[must_use]
pub fn trace<const R: usize, const C: usize>(m: &Tensor2<f64, R, C>) -> f64 {
    assert!(R == C, "trace is only defined for square matrices (R == C)");
    let mut sum = 0.0;
    let mut i = 0;
    while i < R {
        sum += m[(i, i)];
        i += 1;
    }
    sum
}

/// Returns the Frobenius norm of a matrix: `sqrt(Σ_i Σ_j m[i][j]²)`.
///
/// This is the Euclidean norm of the matrix treated as a vector of its entries.
///
/// # Examples
///
/// ```
/// use tpt_zero_linalg::frobenius_norm;
/// use tpt_zero_tensor::Tensor2;
///
/// let m = Tensor2::from([[3.0, 4.0]]);
/// assert!((frobenius_norm(&m) - 5.0).abs() < 1e-12);
/// ```
#[must_use]
pub fn frobenius_norm<const R: usize, const C: usize>(m: &Tensor2<f64, R, C>) -> f64 {
    let mut sum = 0.0;
    let mut r = 0;
    while r < R {
        let mut c = 0;
        while c < C {
            let x = m[(r, c)];
            sum += x * x;
            c += 1;
        }
        r += 1;
    }
    sqrt(sum)
}

/// Multiplies an `R`-by-`C` matrix `m` by a length-`C` column vector `v`,
/// producing a length-`R` vector.
///
/// Each output entry `i` is the dot product of row `i` of `m` with `v`.
///
/// # Examples
///
/// ```
/// use tpt_zero_linalg::mat_vec_mul;
/// use tpt_zero_tensor::{Tensor, Tensor2};
///
/// let m = Tensor2::from([[1.0, 2.0], [3.0, 4.0]]);
/// let v = Tensor::from([5.0, 6.0]);
/// let y = mat_vec_mul(&m, &v);
/// assert_eq!(y.as_ref(), &[17.0, 39.0]);
/// ```
#[must_use]
pub fn mat_vec_mul<const R: usize, const C: usize>(
    m: &Tensor2<f64, R, C>,
    v: &Tensor<f64, C>,
) -> Tensor<f64, R> {
    Tensor::new(core::array::from_fn(|r| {
        let mut acc = 0.0;
        let mut c = 0;
        while c < C {
            acc += m[(r, c)] * v[c];
            c += 1;
        }
        acc
    }))
}

/// Computes the square root of `x >= 0.0`.
///
/// Returns `f64::NAN` for negative inputs and for `f64::NAN`. This is provided
/// because `f64::sqrt` is not available in every `core`-only target. The
/// implementation lives in [`out_zero_float`]; it is subnormal-safe and uses a
/// relative convergence tolerance, so it remains accurate for both tiny
/// (`1e-30`) and huge (`1e300`) magnitudes.
///
/// # Examples
///
/// ```
/// use tpt_zero_linalg::sqrt;
///
/// assert!((sqrt(2.0) - core::f64::consts::SQRT_2).abs() < 1e-12);
/// assert_eq!(sqrt(0.0), 0.0);
/// assert!(sqrt(-1.0).is_nan());
/// ```
#[must_use]
pub fn sqrt(x: f64) -> f64 {
    out_zero_float::sqrt(x)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_zero_tensor::Tensor;

    #[test]
    fn dot_basic() {
        let a = Tensor::from([1.0, 2.0, 3.0]);
        let b = Tensor::from([4.0, 5.0, 6.0]);
        assert!((dot(&a, &b) - 32.0).abs() < 1e-12);
        // self dot equals squared L2 norm squared.
        assert!((dot(&a, &a) - norm_l2(&a) * norm_l2(&a)).abs() < 1e-12);
    }

    #[test]
    fn cross_basic() {
        let x = Tensor::from([1.0, 0.0, 0.0]);
        let y = Tensor::from([0.0, 1.0, 0.0]);
        assert_eq!(cross(&x, &y).as_ref(), &[0.0, 0.0, 1.0]);
        // a × a = 0.
        assert_eq!(cross(&x, &x).as_ref(), &[0.0, 0.0, 0.0]);
    }

    #[test]
    fn cross_orthogonal_to_inputs() {
        let a = Tensor::from([1.0, 2.0, 3.0]);
        let b = Tensor::from([4.0, 5.0, 6.0]);
        let c = cross(&a, &b);
        assert!((dot(&c, &a)).abs() < 1e-12);
        assert!((dot(&c, &b)).abs() < 1e-12);
    }

    #[test]
    fn norms_basic() {
        let v = Tensor::from([-3.0, 4.0]);
        assert!((norm_l1(&v) - 7.0).abs() < 1e-12);
        assert!((norm_max(&v) - 4.0).abs() < 1e-12);
        assert!((norm_l2(&v) - 5.0).abs() < 1e-12);
    }

    #[test]
    fn norm_non_negative() {
        for &x in &[-2.0, -1.0, 0.0, 1.0, 2.0] {
            let v = Tensor::from([x]);
            assert!(norm_l1(&v) >= 0.0);
            assert!(norm_l2(&v) >= 0.0);
            assert!(norm_max(&v) >= 0.0);
        }
    }

    #[test]
    fn normalize_zero_is_none() {
        let zero = Tensor::from([0.0, 0.0, 0.0]);
        assert_eq!(normalize(&zero), None);
    }

    #[test]
    fn normalize_scales_to_unit() {
        let v = Tensor::from([3.0, 4.0, 0.0]);
        let n = normalize(&v).unwrap();
        assert!((norm_l2(&n) - 1.0).abs() < 1e-12);
        // scaling preserves direction exactly.
        assert!((n[0] - 0.6).abs() < 1e-12);
        assert!((n[1] - 0.8).abs() < 1e-12);
    }

    #[test]
    fn trace_basic() {
        let m = Tensor2::from([[1.0, 2.0], [3.0, 4.0]]);
        assert!((trace(&m) - 5.0).abs() < 1e-12);
        let m3 = Tensor2::from([[1.0, 0.0, 0.0], [0.0, 2.0, 0.0], [0.0, 0.0, 3.0]]);
        assert!((trace(&m3) - 6.0).abs() < 1e-12);
    }

    #[test]
    fn frobenius_norm_basic() {
        let m = Tensor2::from([[3.0, 4.0]]);
        assert!((frobenius_norm(&m) - 5.0).abs() < 1e-12);
    }

    #[test]
    fn mat_vec_mul_basic() {
        let m = Tensor2::from([[1.0, 2.0], [3.0, 4.0]]);
        let v = Tensor::from([5.0, 6.0]);
        assert_eq!(mat_vec_mul(&m, &v).as_ref(), &[17.0, 39.0]);
    }

    #[test]
    fn sqrt_matches_reference() {
        for &x in &[0.0, 1.0, 2.0, 4.0, 9.0, 25.0, 100.0, 1234.5] {
            assert!((sqrt(x) - x.sqrt()).abs() < 1e-9, "sqrt({x})");
        }
        assert!(sqrt(-1.0).is_nan());
        assert!(sqrt(f64::NAN).is_nan());
    }

    #[test]
    #[should_panic(expected = "trace is only defined for square matrices")]
    fn trace_panics_on_rectangular() {
        let m = Tensor2::from([[1.0, 2.0, 3.0]]);
        let _ = trace(&m);
    }
}

#[cfg(test)]
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
mod proptests {
    use proptest::prelude::*;

    use super::*;
    use tpt_zero_tensor::Tensor;

    prop_compose! {
        fn vec3()(
            a in -100i64..100,
            b in -100i64..100,
            c in -100i64..100,
        ) -> [f64; 3] {
            [a as f64, b as f64, c as f64]
        }
    }

    prop_compose! {
        fn vec4()(
            vals in proptest::array::uniform4(-100i64..100),
        ) -> [f64; 4] {
            vals.map(|x| x as f64)
        }
    }

    proptest! {
        #[test]
        fn cross_orthogonal_to_a_and_b(a in vec3(), b in vec3()) {
            let a = Tensor::from(a);
            let b = Tensor::from(b);
            let c = cross(&a, &b);
            prop_assert!((dot(&c, &a)).abs() < 1e-9, "not orthogonal to a: {}", dot(&c, &a));
            prop_assert!((dot(&c, &b)).abs() < 1e-9, "not orthogonal to b: {}", dot(&c, &b));
        }

        #[test]
        fn norm_non_negative(v in vec4()) {
            let v = Tensor::from(v);
            prop_assert!(norm_l1(&v) >= 0.0);
            prop_assert!(norm_l2(&v) >= 0.0);
            prop_assert!(norm_max(&v) >= 0.0);
        }

        #[test]
        fn normalize_scales_to_unit(v in vec3()) {
            let v = Tensor::from(v);
            match normalize(&v) {
                None => prop_assert!(norm_l2(&v) == 0.0),
                Some(n) => prop_assert!((norm_l2(&n) - 1.0).abs() < 1e-9, "len {}", norm_l2(&n)),
            }
        }
    }
}
