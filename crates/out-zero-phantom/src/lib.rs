#![doc = include_str!("../README.md")]

#![no_std]
#![warn(missing_docs)]
#![forbid(unsafe_code)]

/// A zero-sized newtype wrapper that carries a phantom type parameter `T`
/// without ever storing a value of `T`.
///
/// The wrapper has size `0` regardless of `T`, so it can be used to tag
/// data at the type level (e.g. `Handle<Read>`, `Handle<Write>`) or to make
/// a struct generic over a type it does not actually own, purely to drive
/// the type checker and the borrow checker.
///
/// # Examples
///
/// ```
/// use out_zero_phantom::Phantom;
///
/// let a: Phantom<u32> = Phantom::new();
/// let b: Phantom<u32> = Phantom::default();
/// assert_eq!(a, b);
/// assert_eq!(core::mem::size_of::<Phantom<u32>>(), 0);
/// ```
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Phantom<T: ?Sized> {
    #[allow(dead_code, unused_parens)]
    _marker: core::marker::PhantomData<T>,
}

impl<T: ?Sized> Phantom<T> {
    /// Creates a new zero-sized phantom wrapper carrying `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// use out_zero_phantom::Phantom;
    ///
    /// let _: Phantom<&str> = Phantom::new();
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self {
            _marker: core::marker::PhantomData,
        }
    }
}

/// A zero-sized marker that is **covariant** in `T`.
///
/// Variance matters when `T` appears in positions that are *produced* (e.g.
/// a shared `&T`). A covariant marker allows `F<Sub>` to be used where
/// `F<Super>` is expected. `PhantomData<T>` itself is covariant, so this is
/// the natural default for read-only ownership.
///
/// # Examples
///
/// ```
/// use out_zero_phantom::Covariant;
///
/// fn wants_super(_: Covariant<&'static str>) {}
/// let m: Covariant<&'static str> = Covariant::new();
/// wants_super(m); // covariant: no coercion needed at the type level here,
///                 // but the marker documents the intended variance.
/// assert_eq!(core::mem::size_of::<Covariant<u32>>(), 0);
/// ```
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Covariant<T: ?Sized> {
    #[allow(dead_code, unused_parens)]
    _marker: core::marker::PhantomData<T>,
}

impl<T: ?Sized> Covariant<T> {
    /// Creates a new covariant marker carrying `T`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            _marker: core::marker::PhantomData,
        }
    }
}

/// A zero-sized marker that is **contravariant** in `T`.
///
/// A contravariant marker appears in *consumed* positions (e.g.
/// `fn(T) -> ()`). It allows `F<Super>` to be used where `F<Sub>` is
/// expected. Expressed here with `PhantomData<fn(T)>`.
///
/// # Examples
///
/// ```
/// use out_zero_phantom::Contravariant;
///
/// let _: Contravariant<u32> = Contravariant::new();
/// assert_eq!(core::mem::size_of::<Contravariant<u32>>(), 0);
/// ```
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Contravariant<T: ?Sized> {
    #[allow(dead_code, unused_parens)]
    _marker: core::marker::PhantomData<fn(T)>,
}

impl<T: ?Sized> Contravariant<T> {
    /// Creates a new contravariant marker carrying `T`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            _marker: core::marker::PhantomData,
        }
    }
}

/// A zero-sized marker that is **invariant** in `T`.
///
/// An invariant marker can neither be widened nor narrowed: `F<Sub>` is
/// *not* a subtype of `F<Super>`, and vice versa. This is the correct
/// choice for types that both read and write `T` (mutability ties the
/// lifetimes together), expressed here with `PhantomData<*const T>` plus a
/// function pointer to remove auto-trait consequences while keeping the
/// invariance. The `fn(T) -> T` form is the canonical invariant encoding.
///
/// # Examples
///
/// ```
/// use out_zero_phantom::Invariant;
///
/// let _: Invariant<u32> = Invariant::new();
/// assert_eq!(core::mem::size_of::<Invariant<u32>>(), 0);
/// ```
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Invariant<T: ?Sized> {
    #[allow(dead_code, unused_parens)]
    _marker: core::marker::PhantomData<fn(T) -> T>,
}

impl<T: ?Sized> Invariant<T> {
    /// Creates a new invariant marker carrying `T`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            _marker: core::marker::PhantomData,
        }
    }
}

/// Returns a fresh zero-sized [`Covariant`] marker carrying `T`.
///
/// # Examples
///
/// ```
/// use out_zero_phantom::{covariant, Covariant};
///
/// let m: Covariant<&str> = covariant();
/// assert_eq!(core::mem::size_of_val(&m), 0);
/// ```
#[must_use]
pub const fn covariant<T: ?Sized>() -> Covariant<T> {
    Covariant::new()
}

/// Returns a fresh zero-sized [`Contravariant`] marker carrying `T`.
///
/// # Examples
///
/// ```
/// use out_zero_phantom::{contravariant, Contravariant};
///
/// let m: Contravariant<&str> = contravariant();
/// assert_eq!(core::mem::size_of_val(&m), 0);
/// ```
#[must_use]
pub const fn contravariant<T: ?Sized>() -> Contravariant<T> {
    Contravariant::new()
}

/// Returns a fresh zero-sized [`Invariant`] marker carrying `T`.
///
/// # Examples
///
/// ```
/// use out_zero_phantom::{invariant, Invariant};
///
/// let m: Invariant<&str> = invariant();
/// assert_eq!(core::mem::size_of_val(&m), 0);
/// ```
#[must_use]
pub const fn invariant<T: ?Sized>() -> Invariant<T> {
    Invariant::new()
}

/// Declares a zero-sized marker type that implements `Copy`, `Clone`,
/// `Default`, `PartialEq`, `Eq`, `Hash`, and `Debug`, and carries a
/// `PhantomData` so it can also be parameterized over a tag type.
///
/// The macro supports three shapes:
///
/// - `marker!(Name);` â€” a unit-like zero-sized marker with no phantom tag.
/// - `marker!(pub Name);` â€” same, but `pub`.
/// - `marker!(pub Name<T>);` â€” a generic marker carrying a phantom `T`.
///
/// Because the produced type is zero-sized, it adds no runtime cost and is
/// useful for encoding state machines, capability tokens, and type-level
/// tags at the type level.
///
/// # Examples
///
/// ```
/// use out_zero_phantom::marker;
///
/// marker!(pub Read);
/// marker!(pub Write);
///
/// fn read_only(_: Read) {}
///
/// let token = Read::new();
/// assert_eq!(core::mem::size_of_val(&token), 0);
/// read_only(token);
/// ```
///
/// Generic markers carry a phantom parameter and remain zero-sized:
///
/// ```
/// use out_zero_phantom::marker;
///
/// marker!(pub Handle<T>);
///
/// let h: Handle<u32> = Handle::new();
/// assert_eq!(core::mem::size_of_val(&h), 0);
/// ```
#[macro_export]
macro_rules! marker {
    (pub $name:ident $(< $($gen:ident),* $(,)? >)? $(,)?) => {
        $crate::marker!(@struct pub $name $(< $($gen),* >)?);
    };
    ($name:ident $(< $($gen:ident),* $(,)? >)? $(,)?) => {
        $crate::marker!(@struct $name $(< $($gen),* >)?);
    };
    (@struct pub $name:ident $(< $($gen:ident),* >)?) => {
        #[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
        #[allow(dead_code, unused_parens)]
        #[doc = "Zero-sized marker type generated by the `marker!` macro."]
        pub struct $name $(< $($gen),* >)? {
            #[allow(dead_code, unused_parens)]
            _marker: ::core::marker::PhantomData<($($($gen,)*)? ())>,
        }

        impl $(< $($gen),* >)? $name $(< $($gen),* >)? {
            /// Creates a new zero-sized marker value.
            #[must_use]
            pub const fn new() -> Self {
                Self { _marker: ::core::marker::PhantomData }
            }
        }
    };
    (@struct $name:ident $(< $($gen:ident),* >)?) => {
        #[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
        #[allow(dead_code, unused_parens)]
        #[doc = "Zero-sized marker type generated by the `marker!` macro."]
        struct $name $(< $($gen),* >)? {
            #[allow(dead_code, unused_parens)]
            _marker: ::core::marker::PhantomData<($($($gen,)*)? ())>,
        }

        impl $(< $($gen),* >)? $name $(< $($gen),* >)? {
            /// Creates a new zero-sized marker value.
            #[must_use]
            #[allow(dead_code)]
            pub const fn new() -> Self {
                Self { _marker: ::core::marker::PhantomData }
            }
        }
    };
}

marker!(TestMarkerA);
marker!(pub TestMarkerB);
marker!(pub TestMarkerG<T>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phantom_is_zero_sized_and_eq() {
        assert_eq!(core::mem::size_of::<Phantom<u32>>(), 0);
        assert_eq!(Phantom::<u32>::new(), Phantom::default());
    }

    #[test]
    fn variance_markers_are_zero_sized() {
        assert_eq!(core::mem::size_of::<Covariant<u32>>(), 0);
        assert_eq!(core::mem::size_of::<Contravariant<u32>>(), 0);
        assert_eq!(core::mem::size_of::<Invariant<u32>>(), 0);
    }

    #[test]
    fn constructor_functions_are_zero_sized() {
        assert_eq!(core::mem::size_of_val(&covariant::<u32>()), 0);
        assert_eq!(core::mem::size_of_val(&contravariant::<u32>()), 0);
        assert_eq!(core::mem::size_of_val(&invariant::<u32>()), 0);
    }

    #[test]
    fn marker_macro_unit_is_zero_sized() {
        assert_eq!(core::mem::size_of::<TestMarkerA>(), 0);
        assert_eq!(core::mem::size_of::<TestMarkerB>(), 0);
        assert_eq!(TestMarkerA::new(), TestMarkerA::default());
    }

    #[test]
    fn marker_macro_generic_is_zero_sized() {
        let g: TestMarkerG<u64> = TestMarkerG::new();
        assert_eq!(core::mem::size_of_val(&g), 0);
    }

    #[test]
    fn phantom_covariance_compiles() {
        // Covariant<&str> is a subtype relationship that the compiler
        // accepts because the marker is covariant in T.
        fn takes(_: Covariant<&str>) {}
        let m: Covariant<&str> = Covariant::new();
        takes(m);
    }
}




