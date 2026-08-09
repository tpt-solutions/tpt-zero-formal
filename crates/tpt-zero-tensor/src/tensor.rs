//! A fixed-length vector of `N` elements stored as a `[T; N]` array.
//!
//! [`Tensor<T, N>`] is a compile-time-sized 1D tensor. It supports elementwise
//! `add`/`sub` for `T: Add/Sub`, the [`Add`]/`Sub`] trait operators, `dot`
//! product, `map`, `zip`, functional construction with [`from_fn`], and
//! iteration. No allocation is performed; `N` is always known at compile time.

use core::ops::{Add, Mul, Sub};

/// A fixed-length vector of `N` elements.
///
/// The storage is a plain `[T; N]` array, so the type is `Copy`/`Clone` when
/// `T` is and works in `no_std` with no heap allocation.
///
/// # Examples
///
/// ```
/// use tpt_zero_tensor::Tensor;
///
/// let a = Tensor::from([1, 2, 3]);
/// let b = Tensor::from([4, 5, 6]);
/// let s = a.add(&b);
/// assert_eq!(s.as_ref(), &[5, 7, 9]);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Tensor<T, const N: usize> {
    data: [T; N],
}

impl<T, const N: usize> Tensor<T, N> {
    /// Creates a tensor directly from a fixed-size array.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_tensor::Tensor;
    ///
    /// let t = Tensor::new([1, 2, 3]);
    /// assert_eq!(t.get(0), Some(&1));
    /// ```
    #[must_use]
    pub const fn new(data: [T; N]) -> Self {
        Self { data }
    }

    /// Builds a tensor by evaluating `f(i)` for each index `i` in `0..N`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_tensor::Tensor;
    ///
    /// let t: Tensor<i32, 5> = Tensor::from_fn(|i| i as i32 * 2);
    /// assert_eq!(t.as_ref(), &[0, 2, 4, 6, 8]);
    /// ```
    #[must_use]
    pub fn from_fn<F>(f: F) -> Self
    where
        F: FnMut(usize) -> T,
    {
        Self {
            data: core::array::from_fn(f),
        }
    }

    /// Returns the number of elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_tensor::Tensor;
    ///
    /// let t = Tensor::<i32, 7>::default();
    /// assert_eq!(t.len(), 7);
    /// ```
    #[must_use]
    pub const fn len(&self) -> usize {
        N
    }

    /// Returns `true` if the tensor has zero elements.
    ///
    /// This is only possible when `N == 0`.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        N == 0
    }

    /// Returns a reference to the element at `index`, or `None` if out of
    /// bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_tensor::Tensor;
    ///
    /// let t = Tensor::from([10, 20, 30]);
    /// assert_eq!(t.get(1), Some(&20));
    /// assert_eq!(t.get(3), None);
    /// ```
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }

    /// Returns a mutable reference to the element at `index`, or `None` if out
    /// of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_tensor::Tensor;
    ///
    /// let mut t = Tensor::from([1, 2, 3]);
    /// *t.get_mut(1).unwrap() = 9;
    /// assert_eq!(t.get(1), Some(&9));
    /// ```
    #[must_use]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.data.get_mut(index)
    }

    /// Returns the underlying array as a slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_tensor::Tensor;
    ///
    /// let t = Tensor::from([1, 2, 3]);
    /// assert_eq!(t.as_ref(), &[1, 2, 3]);
    /// ```
    #[must_use]
    pub const fn as_ref(&self) -> &[T] {
        &self.data
    }

    /// Applies `f` elementwise, producing a new tensor with elements of type
    /// `U`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_tensor::Tensor;
    ///
    /// let t = Tensor::from([1, 2, 3]);
    /// let doubled: Tensor<i32, 3> = t.map(|x| x * 2);
    /// assert_eq!(doubled.as_ref(), &[2, 4, 6]);
    /// ```
    #[must_use]
    pub fn map<U, F>(self, mut f: F) -> Tensor<U, N>
    where
        F: FnMut(T) -> U,
        T: Clone,
    {
        Tensor::new(core::array::from_fn(|i| f(self.data[i].clone())))
    }

    /// Combines two tensors elementwise with `f`.
    ///
    /// The index `i` of each pair is also supplied to `f`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_tensor::Tensor;
    ///
    /// let a = Tensor::from([1, 2, 3]);
    /// let b = Tensor::from([4, 5, 6]);
    /// let zipped = a.zip(&b, |i, x, y| x + y + i as i32);
    /// assert_eq!(zipped.as_ref(), &[5, 8, 11]);
    /// ```
    #[must_use]
    pub fn zip<U, F>(self, other: &Tensor<U, N>, mut f: F) -> Tensor<T, N>
    where
        F: FnMut(usize, T, &U) -> T,
        T: Clone,
    {
        Tensor::new(core::array::from_fn(|i| {
            f(i, self.data[i].clone(), &other.data[i])
        }))
    }

    /// Returns an iterator over the elements by reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_tensor::Tensor;
    ///
    /// let t = Tensor::from([1, 2, 3]);
    /// let sum: i32 = t.iter().sum();
    /// assert_eq!(sum, 6);
    /// ```
    pub fn iter(&self) -> core::slice::Iter<'_, T> {
        self.data.iter()
    }

    /// Returns a mutable iterator over the elements.
    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, T> {
        self.data.iter_mut()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a Tensor<T, N> {
    type Item = &'a T;
    type IntoIter = core::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a mut Tensor<T, N> {
    type Item = &'a mut T;
    type IntoIter = core::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T, const N: usize> Tensor<T, N>
where
    T: Add<Output = T> + Copy,
{
    /// Computes the elementwise sum of `self` and `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_tensor::Tensor;
    ///
    /// let a = Tensor::from([1, 2, 3]);
    /// let b = Tensor::from([4, 5, 6]);
    /// assert_eq!(a.add(&b).as_ref(), &[5, 7, 9]);
    /// ```
    #[must_use]
    pub fn add(&self, other: &Tensor<T, N>) -> Tensor<T, N> {
        Tensor::new(core::array::from_fn(|i| self.data[i] + other.data[i]))
    }
}

impl<T, const N: usize> Tensor<T, N>
where
    T: Sub<Output = T> + Copy,
{
    /// Computes the elementwise difference `self - other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_tensor::Tensor;
    ///
    /// let a = Tensor::from([4, 5, 6]);
    /// let b = Tensor::from([1, 2, 3]);
    /// assert_eq!(a.sub(&b).as_ref(), &[3, 3, 3]);
    /// ```
    #[must_use]
    pub fn sub(&self, other: &Tensor<T, N>) -> Tensor<T, N> {
        Tensor::new(core::array::from_fn(|i| self.data[i] - other.data[i]))
    }
}

impl<T, const N: usize> Tensor<T, N>
where
    T: Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Copy,
{
    /// Returns the dot (inner) product of `self` and `other`.
    ///
    /// The dot product is commutative when `T` is, since it is the sum of
    /// `self[i] * other[i]`.
    ///
    /// # Panics
    ///
    /// Panics if `N == 0`, since an empty tensor has no well-defined dot
    /// product.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_tensor::Tensor;
    ///
    /// let a = Tensor::from([1, 2, 3]);
    /// let b = Tensor::from([4, 5, 6]);
    /// assert_eq!(a.dot(&b), 32);
    /// ```
    #[must_use]
    pub fn dot(&self, other: &Tensor<T, N>) -> T
    where
        T: Mul<Output = T>,
    {
        let mut acc: Option<T> = None;
        let mut i = 0;
        while i < N {
            let product = self.data[i] * other.data[i];
            acc = Some(match acc {
                Some(a) => a + product,
                None => product,
            });
            i += 1;
        }
        // `N >= 1` guarantees `acc` was assigned in the first iteration;
        // an empty tensor (N == 0) has no well-defined dot product.
        match acc {
            Some(value) => value,
            None => panic!("dot product of an empty tensor (N == 0) is undefined"),
        }
    }
}

impl<T, const N: usize> Default for Tensor<T, N>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            data: core::array::from_fn(|_| T::default()),
        }
    }
}

impl<T, const N: usize> From<[T; N]> for Tensor<T, N> {
    fn from(data: [T; N]) -> Self {
        Self { data }
    }
}

impl<T, const N: usize> AsRef<[T]> for Tensor<T, N> {
    fn as_ref(&self) -> &[T] {
        &self.data
    }
}

impl<T, const N: usize> core::ops::Index<usize> for Tensor<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<T, const N: usize> core::ops::IndexMut<usize> for Tensor<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl<T, const N: usize> Add for Tensor<T, N>
where
    T: Add<Output = T> + Copy,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Tensor::add(&self, &rhs)
    }
}

impl<T, const N: usize> Sub for Tensor<T, N>
where
    T: Sub<Output = T> + Copy,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Tensor::sub(&self, &rhs)
    }
}

#[cfg(test)]
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
mod tests {
    use super::*;

    #[test]
    fn from_array_and_index() {
        let t = Tensor::from([1, 2, 3]);
        assert_eq!(t[0], 1);
        assert_eq!(t[2], 3);
    }

    #[test]
    fn get_out_of_bounds() {
        let t = Tensor::<i32, 3>::new([1, 2, 3]);
        assert_eq!(t.get(3), None);
    }

    #[test]
    fn from_fn_index_order() {
        let t: Tensor<i32, 4> = Tensor::from_fn(|i| i as i32 * 10);
        assert_eq!(t.as_ref(), &[0, 10, 20, 30]);
    }

    #[test]
    fn add_sub_roundtrip() {
        let a = Tensor::from([1, 2, 3]);
        let b = Tensor::from([4, 5, 6]);
        let s = Tensor::add(&a, &b);
        let d = Tensor::sub(&s, &b);
        assert_eq!(d, a);
    }

    #[test]
    fn dot_product() {
        let a = Tensor::from([1, 2, 3]);
        let b = Tensor::from([4, 5, 6]);
        assert_eq!(a.dot(&b), 32);
    }

    #[test]
    fn dot_is_commutative() {
        let a = Tensor::from([1, 2, 3]);
        let b = Tensor::from([4, 5, 6]);
        assert_eq!(a.dot(&b), b.dot(&a));
    }

    #[test]
    fn map_and_zip() {
        let a = Tensor::from([1, 2, 3]);
        let doubled = a.map(|x| x * 2);
        assert_eq!(doubled.as_ref(), &[2, 4, 6]);

        let b = Tensor::from([10, 20, 30]);
        let z = a.zip(&b, |i, x, y| x + y + i as i32);
        assert_eq!(z.as_ref(), &[11, 23, 35]);
    }

    #[test]
    fn iter_sum() {
        let t = Tensor::from([1, 2, 3, 4]);
        assert_eq!(t.iter().sum::<i32>(), 10);
    }

    #[test]
    fn default_is_zeros() {
        let t: Tensor<i32, 4> = Tensor::default();
        assert_eq!(t.as_ref(), &[0, 0, 0, 0]);
    }

    #[test]
    fn operator_overloads() {
        let a = Tensor::from([1, 2]);
        let b = Tensor::from([3, 4]);
        assert_eq!((a + b).as_ref(), &[4, 6]);
        assert_eq!((b - a).as_ref(), &[2, 2]);
    }

    #[test]
    fn empty_tensor() {
        let t: Tensor<i32, 0> = Tensor::default();
        assert!(t.is_empty());
        assert_eq!(t.len(), 0);
    }
}
