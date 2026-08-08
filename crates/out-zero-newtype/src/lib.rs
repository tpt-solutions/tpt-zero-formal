#![doc = include_str!("../README.md")]

#![no_std]
#![warn(missing_docs)]
#![forbid(unsafe_code)]

#[cfg(feature = "alloc")]
extern crate alloc;

/// A newtype wrapper that knows its inner, wrapped type.
///
/// Every type produced by [`define_newtype!`] implements this trait, giving
/// uniform, name-free access to the wrapped value in generic code.
///
/// # Examples
///
/// ```
/// use out_zero_newtype::{define_newtype, Newtype};
///
/// define_newtype!(UserId, u64);
///
/// fn unwrap<T: Newtype>(w: T) -> T::Inner {
///     w.into_inner()
/// }
///
/// assert_eq!(unwrap(UserId(7)), 7u64);
/// ```
pub trait Newtype {
    /// The wrapped, underlying type.
    type Inner;

    /// Consumes the wrapper and returns the inner value.
    ///
    /// # Examples
    ///
    /// ```
    /// use out_zero_newtype::{define_newtype, Newtype};
    ///
    /// define_newtype!(Count, u32);
    /// let c = Count(5);
    /// assert_eq!(c.into_inner(), 5u32);
    /// ```
    fn into_inner(self) -> Self::Inner;

    /// Wraps an inner value into the newtype.
    ///
    /// # Examples
    ///
    /// ```
    /// use out_zero_newtype::{define_newtype, Newtype};
    ///
    /// define_newtype!(Count, u32);
    /// let c = Count::from_inner(5);
    /// assert_eq!(c.0, 5u32);
    /// ```
    fn from_inner(inner: Self::Inner) -> Self;
}

/// Declares a newtype wrapper around an inner type with auto-derived traits.
///
/// The basic form, `define_newtype!(Name, Inner);`, produces:
///
/// - a `#[repr(transparent)]` tuple struct `Name(Inner)`,
/// - derived `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`, and `Default`,
/// - `From<Inner>` (and therefore `Into<Inner>`) conversions,
/// - an implementation of the [`Newtype`] trait.
///
/// By default the macro derives `Clone`, `Debug`, `PartialEq`, and
/// `Default`, and adds `From<Inner>`/`Into<Inner>` plus a `Newtype`
/// implementation. `Eq` and `Copy` are *not* derived by default because
/// `Inner` may be a floating-point, `String`, or other type that does not
/// implement them; pass `eq` and/or `copy` to derive them when `Inner`
/// provides them.
///
/// Append `deref` to also derive `Deref`/`DerefMut` so the wrapper
/// transparently exposes the inner type's methods:
///
/// ```rust
/// use out_zero_newtype::define_newtype;
///
/// define_newtype!(Tag, String, deref);
///
/// let mut t = Tag(String::from("a"));
/// t.push('b');
/// assert_eq!(t.0, "ab");
/// ```
///
/// The flags `eq`, `copy`, and `deref` may be combined in any order, e.g.
/// `define_newtype!(Name, Inner, eq, copy, deref);`.
///
/// The generated struct is `#[repr(transparent)]`, so it has exactly the
/// same layout as `Inner` and costs nothing at runtime.
///
/// # Panics
///
/// `Default` is derived for the generated type, so it relies on `Inner:
/// Default`. If `Inner` is not `Default`, building with the `default` trait
/// in scope will fail to compile.
#[macro_export]
macro_rules! define_newtype {
    ($name:ident, $inner:ty $(,)?) => {
        $crate::_define_newtype_base!($name, $inner, @no_eq, @no_copy);
    };
    ($name:ident, $inner:ty, eq $(,)?) => {
        $crate::_define_newtype_base!($name, $inner, @eq, @no_copy);
    };
    ($name:ident, $inner:ty, copy $(,)?) => {
        $crate::_define_newtype_base!($name, $inner, @no_eq, @copy);
    };
    ($name:ident, $inner:ty, eq, copy $(,)?) => {
        $crate::_define_newtype_base!($name, $inner, @eq, @copy);
    };
    ($name:ident, $inner:ty, copy, eq $(,)?) => {
        $crate::_define_newtype_base!($name, $inner, @eq, @copy);
    };
    ($name:ident, $inner:ty, deref $(,)?) => {
        $crate::_define_newtype_base!($name, $inner, @no_eq, @no_copy);
        $crate::_define_newtype_deref!($name, $inner);
    };
    ($name:ident, $inner:ty, eq, deref $(,)?) => {
        $crate::_define_newtype_base!($name, $inner, @eq, @no_copy);
        $crate::_define_newtype_deref!($name, $inner);
    };
    ($name:ident, $inner:ty, deref, eq $(,)?) => {
        $crate::_define_newtype_base!($name, $inner, @eq, @no_copy);
        $crate::_define_newtype_deref!($name, $inner);
    };
    ($name:ident, $inner:ty, copy, deref $(,)?) => {
        $crate::_define_newtype_base!($name, $inner, @no_eq, @copy);
        $crate::_define_newtype_deref!($name, $inner);
    };
    ($name:ident, $inner:ty, deref, copy $(,)?) => {
        $crate::_define_newtype_base!($name, $inner, @no_eq, @copy);
        $crate::_define_newtype_deref!($name, $inner);
    };
    ($name:ident, $inner:ty, eq, copy, deref $(,)?) => {
        $crate::_define_newtype_base!($name, $inner, @eq, @copy);
        $crate::_define_newtype_deref!($name, $inner);
    };
    ($name:ident, $inner:ty, copy, eq, deref $(,)?) => {
        $crate::_define_newtype_base!($name, $inner, @eq, @copy);
        $crate::_define_newtype_deref!($name, $inner);
    };
    ($name:ident, $inner:ty, eq, deref, copy $(,)?) => {
        $crate::_define_newtype_base!($name, $inner, @eq, @copy);
        $crate::_define_newtype_deref!($name, $inner);
    };
    ($name:ident, $inner:ty, copy, deref, eq $(,)?) => {
        $crate::_define_newtype_base!($name, $inner, @eq, @copy);
        $crate::_define_newtype_deref!($name, $inner);
    };
    ($name:ident, $inner:ty, deref, eq, copy $(,)?) => {
        $crate::_define_newtype_base!($name, $inner, @eq, @copy);
        $crate::_define_newtype_deref!($name, $inner);
    };
    ($name:ident, $inner:ty, deref, copy, eq $(,)?) => {
        $crate::_define_newtype_base!($name, $inner, @eq, @copy);
        $crate::_define_newtype_deref!($name, $inner);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _define_newtype_base {
    ($name:ident, $inner:ty, @eq, @copy) => {
        #[repr(transparent)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
        pub struct $name($inner);

        $crate::_define_newtype_impls!($name, $inner);
    };
    ($name:ident, $inner:ty, @eq, @no_copy) => {
        #[repr(transparent)]
        #[derive(Clone, Debug, PartialEq, Eq, Default)]
        pub struct $name($inner);

        $crate::_define_newtype_impls!($name, $inner);
    };
    ($name:ident, $inner:ty, @no_eq, @copy) => {
        #[repr(transparent)]
        #[derive(Clone, Copy, Debug, PartialEq, Default)]
        pub struct $name($inner);

        $crate::_define_newtype_impls!($name, $inner);
    };
    ($name:ident, $inner:ty, @no_eq, @no_copy) => {
        #[repr(transparent)]
        #[derive(Clone, Debug, PartialEq, Default)]
        pub struct $name($inner);

        $crate::_define_newtype_impls!($name, $inner);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _define_newtype_deref {
    ($name:ident, $inner:ty) => {
        impl ::core::ops::Deref for $name {
            type Target = $inner;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl ::core::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _define_newtype_impls {
    ($name:ident, $inner:ty) => {
        impl $name {
            /// Creates a new wrapper from an inner value.
            #[must_use]
            pub const fn new(inner: $inner) -> Self {
                Self(inner)
            }

            /// Returns a reference to the inner value.
            #[must_use]
            pub const fn as_inner(&self) -> &$inner {
                &self.0
            }
        }

        impl ::core::convert::From<$inner> for $name {
            fn from(inner: $inner) -> Self {
                Self(inner)
            }
        }

        impl ::core::convert::From<$name> for $inner {
            fn from(wrapper: $name) -> Self {
                wrapper.0
            }
        }

        impl $crate::Newtype for $name {
            type Inner = $inner;

            fn into_inner(self) -> Self::Inner {
                self.0
            }

            fn from_inner(inner: Self::Inner) -> Self {
                Self(inner)
            }
        }
    };
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::default_trait_access)]
mod tests {
    use super::*;

    define_newtype!(Meters, f64);
    define_newtype!(UserId, u64);
    define_newtype!(Count, u32, eq);

    #[test]
    fn eq_flag_derives_eq() {
        let a = Count(1);
        let b = Count(1);
        assert!(a == b);
        assert!(a != Count(2));
    }

    #[test]
    fn from_inner_into_inner_roundtrip() {
        let m = Meters::from_inner(2.5);
        assert_eq!(m.into_inner(), 2.5);
    }

    #[test]
    fn from_conversion_and_tuple_access() {
        let m: Meters = 1.0.into();
        assert_eq!(m.0, 1.0);
        let back: f64 = m.into();
        assert_eq!(back, 1.0);
    }

    #[test]
    fn new_and_as_inner() {
        let u = UserId::new(42);
        assert_eq!(u.as_inner(), &42u64);
        assert_eq!(u.into_inner(), 42u64);
    }

    #[test]
    fn derived_traits() {
        let a = Meters(1.0);
        let b = a.clone();
        assert_eq!(a, b);
        assert_eq!(a, Meters(1.0));

        let d: Meters = Default::default();
        assert_eq!(d.0, 0.0);
    }
}

#[cfg(all(test, feature = "alloc"))]
mod alloc_tests {
    use super::*;

    #[cfg(feature = "alloc")]
    use alloc::string::String;

    define_newtype!(Tag, String, deref);

    #[test]
    fn deref_exposes_inner_methods() {
        let mut t = Tag::from_inner(String::from("hi"));
        t.push_str(" there");
        assert_eq!(t.0, "hi there");
        assert_eq!(t.len(), 8);

        t.clear();
        assert_eq!(t.0, "");
    }
}
