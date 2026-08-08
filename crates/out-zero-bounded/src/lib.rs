#![doc = include_str!("../README.md")]

#![no_std]
#![warn(missing_docs)]
#![forbid(unsafe_code)]
// i64 arithmetic on the inner value is checked and clamped, but the pedantic
// cast/arithmetic lints flag the explicit `as i64` saturation paths and the
// `i128` widening used to detect overflow -- they are noise for a crate that
// is fundamentally about safe, checked integer arithmetic.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]

/// Error returned when a value cannot be represented by a [`BoundedInt`],
/// because it falls outside the `[MIN, MAX]` range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundsError;

impl core::fmt::Display for BoundsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("value is outside the BoundedInt range")
    }
}

impl core::error::Error for BoundsError {}

/// A zero-cost wrapper around an `i64` guaranteed to lie in `[MIN, MAX]`.
///
/// The range `[MIN, MAX]` is checked at compile time via
/// [`assert_i64_le`] (from `out-zero-assert-const`), so an inverted range is a compile
/// error rather than a runtime panic. The wrapped value is checked against
/// the range on construction and the arithmetic helpers saturate/clamp to
/// the bounds, so a `BoundedInt` never holds an out-of-range value.
///
/// # Examples
///
/// ```
/// use out_zero_bounded::{BoundedInt, BoundsError};
///
/// type Small = BoundedInt<-5, 5>;
///
/// let a = Small::new(3).unwrap();
/// let b = Small::new(-10);
/// assert_eq!(a.value(), 3);
/// assert!(matches!(b, Err(BoundsError)));
///
/// let summed = a.checked_add(1).unwrap();
/// assert_eq!(summed.value(), 4);
/// let clamped = a.saturating_add(100);
/// assert_eq!(clamped.value(), 5);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct BoundedInt<const MIN: i64, const MAX: i64> {
    /// The stored value. Invariant: `MIN <= value <= MAX`.
    value: i64,
}

impl<const MIN: i64, const MAX: i64> BoundedInt<MIN, MAX> {
    // Force evaluation of the MIN<=MAX invariant at every constructor. An
    // associated const is only monomorphized when it is referenced, so without
    // this explicit reference an inverted range (`BoundedInt<100, 0>`) would
    // compile and silently violate the invariant. Referencing `ASSERT_RANGE`
    // from `new` and `clamp` turns it into a compile error wherever the type
    // is actually used.
    const ASSERT_RANGE: () = assert!(MIN <= MAX, "BoundedInt: MIN must be <= MAX");

    /// The inclusive lower bound of this type's range.
    pub const MIN: i64 = MIN;

    /// The inclusive upper bound of this type's range.
    pub const MAX: i64 = MAX;

    /// Attempts to construct a [`BoundedInt`] from `value`, returning
    /// [`BoundsError`] if it falls outside `[MIN, MAX]`.
    ///
    /// # Examples
    ///
    /// ```
    /// use out_zero_bounded::{BoundedInt, BoundsError};
    ///
    /// type U8 = BoundedInt<0, 255>;
    /// assert_eq!(U8::new(200).unwrap().value(), 200);
    /// assert!(matches!(U8::new(256), Err(BoundsError)));
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`BoundsError`] when `value < MIN` or `value > MAX`.
    pub fn new(value: i64) -> Result<Self, BoundsError> {
        let () = Self::ASSERT_RANGE;
        if value < MIN || value > MAX {
            return Err(BoundsError);
        }
        Ok(Self { value })
    }

    /// Constructs a [`BoundedInt`] from `value` without checking the range.
    ///
    /// This is a safe, `unsafe_code`-free escape hatch that **clamps** into
    /// `[MIN, MAX]` in every build profile, so the resulting value always
    /// satisfies the invariant. Prefer [`BoundedInt::new`] when an
    /// out-of-range value should be rejected rather than clamped.
    ///
    /// # Examples
    ///
    /// ```
    /// use out_zero_bounded::BoundedInt;
    ///
    /// type U8 = BoundedInt<0, 255>;
    /// let v = U8::new_clamped(42);
    /// assert_eq!(v.value(), 42);
    /// assert_eq!(U8::new_clamped(300).value(), 255);
    /// assert_eq!(U8::new_clamped(-5).value(), 0);
    /// ```
    #[must_use]
    pub const fn new_clamped(value: i64) -> Self {
        Self::clamp(value)
    }

    /// Returns the wrapped value.
    ///
    /// # Examples
    ///
    /// ```
    /// use out_zero_bounded::BoundedInt;
    ///
    /// assert_eq!(BoundedInt::<0, 10>::new(7).unwrap().value(), 7);
    /// ```
    #[must_use]
    pub const fn value(self) -> i64 {
        self.value
    }

    /// Returns the inclusive lower bound `MIN`.
    #[must_use]
    pub const fn min(self) -> i64 {
        MIN
    }

    /// Returns the inclusive upper bound `MAX`.
    #[must_use]
    pub const fn max(self) -> i64 {
        MAX
    }

    /// Clamps `value` into `[MIN, MAX]` and returns the resulting
    /// [`BoundedInt`].
    ///
    /// Unlike [`BoundedInt::new`], this never fails: values below `MIN`
    /// become `MIN` and values above `MAX` become `MAX`.
    ///
    /// # Examples
    ///
    /// ```
    /// use out_zero_bounded::BoundedInt;
    ///
    /// type U8 = BoundedInt<0, 255>;
    /// assert_eq!(U8::clamp(-5).value(), 0);
    /// assert_eq!(U8::clamp(300).value(), 255);
    /// assert_eq!(U8::clamp(100).value(), 100);
    /// ```
    #[must_use]
    pub const fn clamp(value: i64) -> Self {
        let clamped = if value < MIN { MIN } else if value > MAX { MAX } else { value };
        Self { value: clamped }
    }

    /// Returns `Some(self + rhs)` clamped to `[MIN, MAX]`, or `None` if the
    /// true sum would fall outside the range.
    ///
    /// This checks the full `i64` range before clamping, so it never
    /// overflows.
    ///
    /// # Examples
    ///
    /// ```
    /// use out_zero_bounded::BoundedInt;
    ///
    /// type U8 = BoundedInt<0, 255>;
    /// assert_eq!(U8::new(10).unwrap().checked_add(5).unwrap().value(), 15);
    /// assert!(U8::new(250).unwrap().checked_add(10).is_none());
    /// ```
    #[must_use]
    pub fn checked_add(self, rhs: i64) -> Option<Self> {
        let sum = i128::from(self.value).checked_add(i128::from(rhs))?;
        if sum < i128::from(MIN) || sum > i128::from(MAX) {
            return None;
        }
        Some(Self { value: sum as i64 })
    }

    /// Returns `self + rhs` clamped to `[MIN, MAX]`, saturating at the
    /// bounds instead of overflowing or erroring.
    ///
    /// # Examples
    ///
    /// ```
    /// use out_zero_bounded::BoundedInt;
    ///
    /// type U8 = BoundedInt<0, 255>;
    /// assert_eq!(U8::new(10).unwrap().saturating_add(5).value(), 15);
    /// assert_eq!(U8::new(250).unwrap().saturating_add(100).value(), 255);
    /// ```
    #[must_use]
    pub fn saturating_add(self, rhs: i64) -> Self {
        if let Some(v) = self.checked_add(rhs) {
            return v;
        }
        // Clamp in the full i128 range, then narrow to i64. Because the
        // result is guaranteed within [MIN, MAX] (itself an i64 range),
        // the final `as i64` cannot wrap or truncate.
        let clamped = i128::from(self.value)
            .saturating_add(i128::from(rhs))
            .clamp(i128::from(MIN), i128::from(MAX));
        Self { value: clamped as i64 }
    }

    /// Returns `Some(self - rhs)` clamped to `[MIN, MAX]`, or `None` if the
    /// true difference would fall outside the range.
    ///
    /// # Examples
    ///
    /// ```
    /// use out_zero_bounded::BoundedInt;
    ///
    /// type U8 = BoundedInt<0, 255>;
    /// assert_eq!(U8::new(10).unwrap().checked_sub(5).unwrap().value(), 5);
    /// assert!(U8::new(5).unwrap().checked_sub(10).is_none());
    /// ```
    #[must_use]
    pub fn checked_sub(self, rhs: i64) -> Option<Self> {
        let diff = i128::from(self.value).checked_sub(i128::from(rhs))?;
        if diff < i128::from(MIN) || diff > i128::from(MAX) {
            return None;
        }
        Some(Self { value: diff as i64 })
    }

    /// Returns `self - rhs` clamped to `[MIN, MAX]`, saturating at the
    /// bounds instead of overflowing or erroring.
    ///
    /// # Examples
    ///
    /// ```
    /// use out_zero_bounded::BoundedInt;
    ///
    /// type U8 = BoundedInt<0, 255>;
    /// assert_eq!(U8::new(10).unwrap().saturating_sub(5).value(), 5);
    /// assert_eq!(U8::new(5).unwrap().saturating_sub(100).value(), 0);
    /// ```
    #[must_use]
    pub fn saturating_sub(self, rhs: i64) -> Self {
        if let Some(v) = self.checked_sub(rhs) {
            return v;
        }
        let clamped = i128::from(self.value)
            .saturating_sub(i128::from(rhs))
            .clamp(i128::from(MIN), i128::from(MAX));
        Self { value: clamped as i64 }
    }
}

impl<const MIN: i64, const MAX: i64> core::fmt::Display for BoundedInt<MIN, MAX> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(&self.value, f)
    }
}

impl<const MIN: i64, const MAX: i64> TryFrom<i64> for BoundedInt<MIN, MAX> {
    type Error = BoundsError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl<const MIN: i64, const MAX: i64> From<BoundedInt<MIN, MAX>> for i64 {
    fn from(value: BoundedInt<MIN, MAX>) -> Self {
        value.value
    }
}

/// Demonstrates that `SafeCast` is wired into the construction path for
/// checked, explicit conversion of arbitrary integer types into a
/// [`BoundedInt`].
#[cfg(test)]
mod safe_cast_usage {
    use super::*;
    use out_zero_safe_cast::SafeCast;

    #[test]
    fn safe_cast_then_bound() {
        let v: u32 = 42;
        let as_i64: i64 = v.safe_cast().unwrap();
        assert_eq!(BoundedInt::<0, 100>::new(as_i64).unwrap().value(), 42);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn in_range_value_constructs() {
        assert_eq!(BoundedInt::<0, 100>::new(0).unwrap().value(), 0);
        assert_eq!(BoundedInt::<0, 100>::new(100).unwrap().value(), 100);
        assert_eq!(BoundedInt::<-50, 50>::new(-50).unwrap().value(), -50);
        assert_eq!(BoundedInt::<-50, 50>::new(50).unwrap().value(), 50);
    }

    #[test]
    fn out_of_range_value_errors() {
        assert!(BoundedInt::<0, 100>::new(-1).is_err());
        assert!(BoundedInt::<0, 100>::new(101).is_err());
        assert!(BoundedInt::<-10, 10>::new(11).is_err());
        assert!(BoundedInt::<-10, 10>::new(-11).is_err());
    }

    #[test]
    fn associated_consts_match_generics() {
        assert_eq!(BoundedInt::<5, 9>::MIN, 5);
        assert_eq!(BoundedInt::<5, 9>::MAX, 9);
        assert_eq!(BoundedInt::<5, 9>::new(7).unwrap().min(), 5);
        assert_eq!(BoundedInt::<5, 9>::new(7).unwrap().max(), 9);
    }

    #[test]
    fn clamp_saturates_both_ends() {
        assert_eq!(BoundedInt::<0, 10>::clamp(-5).value(), 0);
        assert_eq!(BoundedInt::<0, 10>::clamp(15).value(), 10);
        assert_eq!(BoundedInt::<0, 10>::clamp(5).value(), 5);
    }

    #[test]
    fn checked_add_in_bounds() {
        assert_eq!(BoundedInt::<0, 100>::new(40).unwrap().checked_add(10).unwrap().value(), 50);
    }

    #[test]
    fn checked_add_out_of_bounds_is_none() {
        assert!(BoundedInt::<0, 100>::new(95).unwrap().checked_add(10).is_none());
        assert!(BoundedInt::<0, 100>::new(0).unwrap().checked_add(-5).is_none());
    }

    #[test]
    fn saturating_add_clamps() {
        assert_eq!(BoundedInt::<0, 100>::new(95).unwrap().saturating_add(10).value(), 100);
        assert_eq!(BoundedInt::<0, 100>::new(5).unwrap().saturating_add(-10).value(), 0);
        assert_eq!(BoundedInt::<0, 100>::new(50).unwrap().saturating_add(20).value(), 70);
    }

    #[test]
    fn checked_sub_in_bounds() {
        assert_eq!(BoundedInt::<0, 100>::new(40).unwrap().checked_sub(10).unwrap().value(), 30);
    }

    #[test]
    fn checked_sub_out_of_bounds_is_none() {
        assert!(BoundedInt::<0, 100>::new(5).unwrap().checked_sub(10).is_none());
        assert!(BoundedInt::<0, 100>::new(95).unwrap().checked_sub(-10).is_none());
    }

    #[test]
    fn saturating_sub_clamps() {
        assert_eq!(BoundedInt::<0, 100>::new(5).unwrap().saturating_sub(10).value(), 0);
        assert_eq!(BoundedInt::<0, 100>::new(95).unwrap().saturating_sub(-10).value(), 100);
        assert_eq!(BoundedInt::<0, 100>::new(50).unwrap().saturating_sub(20).value(), 30);
    }

    #[test]
    fn try_from_roundtrip() {
        let v = BoundedInt::<0, 10>::try_from(7).unwrap();
        let back: i64 = v.into();
        assert_eq!(back, 7);
        assert!(BoundedInt::<0, 10>::try_from(99).is_err());
    }

    #[test]
    fn ordering_and_equality() {
        let a = BoundedInt::<0, 100>::new(10).unwrap();
        let b = BoundedInt::<0, 100>::new(20).unwrap();
        assert!(a < b);
        assert_eq!(a, BoundedInt::<0, 100>::new(10).unwrap());
        assert_ne!(a, b);
    }

    #[test]
    fn display_formats_inner() {
        use core::fmt::{self, Write};

        struct Buf<'a> {
            inner: &'a mut [u8],
            pos: usize,
        }
        impl Write for Buf<'_> {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                let bytes = s.as_bytes();
                if self.pos + bytes.len() > self.inner.len() {
                    return Err(fmt::Error);
                }
                self.inner[self.pos..self.pos + bytes.len()].copy_from_slice(bytes);
                self.pos += bytes.len();
                Ok(())
            }
        }

        let v = BoundedInt::<0, 100>::new(42).unwrap();
        let mut storage = [0u8; 8];
        let mut buf = Buf { inner: &mut storage, pos: 0 };
        write!(buf, "{v}").unwrap();
        let len = buf.pos;
        let written = core::str::from_utf8(&storage[..len]).unwrap();
        assert_eq!(written, "42");
    }

    proptest! {
        #[test]
        fn prop_in_range_always_constructs(v in 0i64..=100) {
            let b = BoundedInt::<0, 100>::clamp(v);
            prop_assert!(b.value() >= 0 && b.value() <= 100);
            prop_assert_eq!(b.value(), v);
        }

        #[test]
        fn prop_construction_invariant(u in any::<i64>()) {
            let b = BoundedInt::<0, 100>::new(u);
            prop_assert_eq!(b.is_ok(), (0..=100).contains(&u));
        }

        #[test]
        fn prop_add_stays_in_bounds(v in 0i64..=100, rhs in any::<i64>()) {
            let b = BoundedInt::<0, 100>::new(v).unwrap();
            let added = b.saturating_add(rhs);
            prop_assert!(added.value() >= 0 && added.value() <= 100);
            let sum = i128::from(v).saturating_add(i128::from(rhs));
            if (0..=100).contains(&sum) {
                prop_assert_eq!(i128::from(added.value()), sum);
            }
        }

        #[test]
        fn prop_sub_stays_in_bounds(v in 0i64..=100, rhs in any::<i64>()) {
            let b = BoundedInt::<0, 100>::new(v).unwrap();
            let subbed = b.saturating_sub(rhs);
            prop_assert!(subbed.value() >= 0 && subbed.value() <= 100);
            let diff = i128::from(v).saturating_sub(i128::from(rhs));
            if (0..=100).contains(&diff) {
                prop_assert_eq!(i128::from(subbed.value()), diff);
            }
        }
    }
}
