//! A fixed-size `R`-by-`C` matrix stored as a `[[T; C]; R]` array.
//!
//! [`Tensor2<T, R, C>`] is a compile-time-sized 2D tensor: `R` rows, each a
//! length-`C` array. It supports matrix multiplication ([`mul`]), [`transpose`],
//! row/column access ([`row`]/[`col`]), elementwise `add`/`sub`, `map`, `zip`,
//! functional construction with [`from_fn`], and iteration. No allocation is
//! performed; `R` and `C` are always known at compile time.

use core::ops::{Add, Mul, Sub};

use crate::Tensor;

/// A fixed-size `R`-row by `C`-column matrix.
///
/// Storage is `[[T; C]; R]`: each row is a contiguous length-`C` array. The
/// type is `Copy`/`Clone` when `T` is and works in `no_std` with no heap
/// allocation.
///
/// # Examples
///
/// ```
/// use tpt_zero_tensor::Tensor2;
///
/// let m = Tensor2::from([[1, 2], [3, 4]]);
/// assert_eq!(m.get(1, 0), Some(&3));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Tensor2<T, const R: usize, const C: usize> {
    rows: [[T; C]; R],
}

impl<T, const R: usize, const C: usize> Tensor2<T, R, C> {
    /// Creates a matrix directly from a row-major nested array.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_tensor::Tensor2;
    ///
    /// let m = Tensor2::new([[1, 2], [3, 4]]);
    /// assert_eq!(m.row(0).as_ref(), &[1, 2]);
    /// ```
    #[must_use]
    pub const fn new(rows: [[T; C]; R]) -> Self {
        Self { rows }
    }

    /// Builds a matrix by evaluating `f(r, c)` for each `(r, c)` coordinate.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_tensor::Tensor2;
    ///
    /// let m: Tensor2<i32, 3, 3> = Tensor2::from_fn(|r, c| (r * 10 + c) as i32);
    /// assert_eq!(m.get(1, 2), Some(&12));
    /// ```
    #[must_use]
    pub fn from_fn<F>(mut f: F) -> Self
    where
        F: FnMut(usize, usize) -> T,
    {
        Self {
            rows: core::array::from_fn(|r| core::array::from_fn(|c| f(r, c))),
        }
    }

    /// Returns the number of rows.
    #[must_use]
    pub const fn nrows(&self) -> usize {
        R
    }

    /// Returns the number of columns.
    #[must_use]
    pub const fn ncols(&self) -> usize {
        C
    }

    /// Returns a reference to the element at `(row, col)`, or `None` if either
    /// index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_tensor::Tensor2;
    ///
    /// let m = Tensor2::from([[1, 2], [3, 4]]);
    /// assert_eq!(m.get(0, 1), Some(&2));
    /// assert_eq!(m.get(2, 0), None);
    /// ```
    #[must_use]
    pub fn get(&self, row: usize, col: usize) -> Option<&T> {
        self.rows.get(row).and_then(|r| r.get(col))
    }

    /// Returns a mutable reference to the element at `(row, col)`, or `None` if
    /// out of bounds.
    #[must_use]
    pub fn get_mut(&mut self, row: usize, col: usize) -> Option<&mut T> {
        self.rows.get_mut(row).and_then(|r| r.get_mut(col))
    }

    /// Returns the `row`-th row as a [`Tensor`].
    ///
    /// # Panics
    ///
    /// Panics if `row >= R`. Use [`get_row`] for a fallible variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_tensor::Tensor2;
    ///
    /// let m = Tensor2::from([[1, 2, 3], [4, 5, 6]]);
    /// assert_eq!(m.row(1).as_ref(), &[4, 5, 6]);
    /// ```
    ///
    /// [`get_row`]: Tensor2::get_row
    #[must_use]
    pub fn row(&self, row: usize) -> Tensor<T, C>
    where
        T: Copy,
    {
        Tensor::new(self.rows[row])
    }

    /// Returns the `row`-th row as a [`Tensor`], or `None` if out of bounds.
    #[must_use]
    pub fn get_row(&self, row: usize) -> Option<Tensor<T, C>>
    where
        T: Copy,
    {
        self.rows.get(row).map(|r| Tensor::new(*r))
    }

    /// Returns the `col`-th column as a [`Tensor`] of length `R`.
    ///
    /// # Panics
    ///
    /// Panics if `col >= C`. Use [`get_col`] for a fallible variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_tensor::Tensor2;
    ///
    /// let m = Tensor2::from([[1, 2, 3], [4, 5, 6]]);
    /// assert_eq!(m.col(2).as_ref(), &[3, 6]);
    /// ```
    ///
    /// [`get_col`]: Tensor2::get_col
    #[must_use]
    pub fn col(&self, col: usize) -> Tensor<T, R>
    where
        T: Copy,
    {
        Tensor::from_fn(|r| self.rows[r][col])
    }

    /// Returns the `col`-th column as a [`Tensor`], or `None` if out of bounds.
    #[must_use]
    pub fn get_col(&self, col: usize) -> Option<Tensor<T, R>>
    where
        T: Copy,
    {
        if col >= C {
            return None;
        }
        Some(Tensor::from_fn(|r| self.rows[r][col]))
    }

    /// Returns an iterator over the rows, each as a [`Tensor`].
    pub fn rows_iter(&self) -> impl Iterator<Item = Tensor<T, C>> + '_
    where
        T: Copy,
    {
        self.rows.iter().map(|r| Tensor::new(*r))
    }

    /// Returns a mutable iterator over the rows.
    pub fn rows_iter_mut(&mut self) -> impl Iterator<Item = &mut [T; C]> {
        self.rows.iter_mut()
    }

    /// Applies `f` elementwise, producing a new matrix with elements of type
    /// `U`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_tensor::Tensor2;
    ///
    /// let m = Tensor2::from([[1, 2], [3, 4]]);
    /// let doubled = m.map(|x| x * 2);
    /// assert_eq!(doubled.get(1, 1), Some(&8));
    /// ```
    #[must_use]
    pub fn map<U, F>(self, mut f: F) -> Tensor2<U, R, C>
    where
        F: FnMut(T) -> U,
        T: Clone,
    {
        Tensor2::new(core::array::from_fn(|r| {
            core::array::from_fn(|c| f(self.rows[r][c].clone()))
        }))
    }

    /// Transposes the matrix, swapping rows and columns.
    ///
    /// The result type flips the dimensions to `Tensor2<T, C, R>`. Applying
    /// [`transpose`] twice is the identity (see the crate tests).
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_tensor::Tensor2;
    ///
    /// let m = Tensor2::from([[1, 2, 3], [4, 5, 6]]);
    /// let t = m.transpose();
    /// assert_eq!(t.get(2, 1), Some(&6));
    /// ```
    ///
    /// [`transpose`]: Tensor2::transpose
    #[must_use]
    pub fn transpose(&self) -> Tensor2<T, C, R>
    where
        T: Copy,
    {
        Tensor2::new(core::array::from_fn(|r| {
            core::array::from_fn(|c| self.rows[c][r])
        }))
    }
}

impl<T, const R: usize, const C: usize> Tensor2<T, R, C>
where
    T: Add<Output = T> + Copy,
{
    /// Computes the elementwise sum of `self` and `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_tensor::Tensor2;
    ///
    /// let a = Tensor2::from([[1, 2], [3, 4]]);
    /// let b = Tensor2::from([[5, 6], [7, 8]]);
    /// assert_eq!(a.add(&b).get(1, 1), Some(&12));
    /// ```
    #[must_use]
    pub fn add(&self, other: &Tensor2<T, R, C>) -> Tensor2<T, R, C> {
        Tensor2::new(core::array::from_fn(|r| {
            core::array::from_fn(|c| self.rows[r][c] + other.rows[r][c])
        }))
    }
}

impl<T, const R: usize, const C: usize> Tensor2<T, R, C>
where
    T: Sub<Output = T> + Copy,
{
    /// Computes the elementwise difference `self - other`.
    #[must_use]
    pub fn sub(&self, other: &Tensor2<T, R, C>) -> Tensor2<T, R, C> {
        Tensor2::new(core::array::from_fn(|r| {
            core::array::from_fn(|c| self.rows[r][c] - other.rows[r][c])
        }))
    }
}

impl<T, const R: usize, const C: usize> Tensor2<T, R, C>
where
    T: Add<Output = T> + Mul<Output = T> + Copy,
{
    /// Multiplies `self` (an `R x C` matrix) by `other` (a `C x K` matrix),
    /// producing an `R x K` matrix.
    ///
    /// Each output element `(r, k)` is the dot product of row `r` of `self`
    /// with column `k` of `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tpt_zero_tensor::Tensor2;
    ///
    /// let a = Tensor2::from([[1, 2], [3, 4]]);
    /// let b = Tensor2::from([[5, 6], [7, 8]]);
    /// let c = a.mul(&b);
    /// assert_eq!(c.get(0, 0), Some(&19));
    /// assert_eq!(c.get(1, 1), Some(&50));
    /// ```
    #[must_use]
    pub fn mul<const K: usize>(&self, other: &Tensor2<T, C, K>) -> Tensor2<T, R, K> {
        Tensor2::new(core::array::from_fn(|r| {
            core::array::from_fn(|k| {
                let mut acc = self.rows[r][0] * other.rows[0][k];
                let mut c = 1;
                while c < C {
                    acc = acc + (self.rows[r][c] * other.rows[c][k]);
                    c += 1;
                }
                acc
            })
        }))
    }
}

impl<T, const R: usize, const C: usize> Default for Tensor2<T, R, C>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            rows: core::array::from_fn(|_| core::array::from_fn(|_| T::default())),
        }
    }
}

impl<T, const R: usize, const C: usize> From<[[T; C]; R]> for Tensor2<T, R, C> {
    fn from(rows: [[T; C]; R]) -> Self {
        Self { rows }
    }
}

impl<T, const R: usize, const C: usize> core::ops::Index<(usize, usize)> for Tensor2<T, R, C> {
    type Output = T;

    fn index(&self, (row, col): (usize, usize)) -> &Self::Output {
        &self.rows[row][col]
    }
}

impl<T, const R: usize, const C: usize> core::ops::IndexMut<(usize, usize)> for Tensor2<T, R, C> {
    fn index_mut(&mut self, (row, col): (usize, usize)) -> &mut Self::Output {
        &mut self.rows[row][col]
    }
}

impl<T, const R: usize, const C: usize> Add for Tensor2<T, R, C>
where
    T: Add<Output = T> + Copy,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Tensor2::add(&self, &rhs)
    }
}

impl<T, const R: usize, const C: usize> Sub for Tensor2<T, R, C>
where
    T: Sub<Output = T> + Copy,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Tensor2::sub(&self, &rhs)
    }
}

#[cfg(test)]
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
mod tests {
    use super::*;

    #[test]
    fn index_and_get() {
        let m = Tensor2::from([[1, 2, 3], [4, 5, 6]]);
        assert_eq!(m[(1, 2)], 6);
        assert_eq!(m.get(0, 0), Some(&1));
        assert_eq!(m.get(2, 0), None);
    }

    #[test]
    fn from_fn_coordinates() {
        let m: Tensor2<i32, 3, 3> = Tensor2::from_fn(|r, c| (r * 10 + c) as i32);
        assert_eq!(m.get(1, 2), Some(&12));
    }

    #[test]
    fn row_and_col() {
        let m = Tensor2::from([[1, 2, 3], [4, 5, 6]]);
        assert_eq!(m.row(0).as_ref(), &[1, 2, 3]);
        assert_eq!(m.col(2).as_ref(), &[3, 6]);
        assert_eq!(m.get_row(2), None);
        assert_eq!(m.get_col(3), None);
    }

    #[test]
    fn transpose_is_involution() {
        let m = Tensor2::from([[1, 2, 3], [4, 5, 6]]);
        let t = m.transpose();
        assert_eq!(t.nrows(), 3);
        assert_eq!(t.ncols(), 2);
        assert_eq!(t.get(2, 1), Some(&6));
        assert_eq!(t.transpose(), m);
    }

    #[test]
    fn add_sub_roundtrip() {
        let a = Tensor2::from([[1, 2], [3, 4]]);
        let b = Tensor2::from([[5, 6], [7, 8]]);
        let s = Tensor2::add(&a, &b);
        assert_eq!(Tensor2::sub(&s, &b), a);
    }

    #[test]
    fn matrix_multiply() {
        let a = Tensor2::from([[1, 2], [3, 4]]);
        let b = Tensor2::from([[5, 6], [7, 8]]);
        let c = Tensor2::mul(&a, &b);
        assert_eq!(c.get(0, 0), Some(&19));
        assert_eq!(c.get(0, 1), Some(&22));
        assert_eq!(c.get(1, 0), Some(&43));
        assert_eq!(c.get(1, 1), Some(&50));
    }

    #[test]
    fn matrix_multiply_rectangular() {
        // 2x3 times 3x2 = 2x2.
        let a = Tensor2::from([[1, 2, 3], [4, 5, 6]]);
        let b = Tensor2::from([[7, 8], [9, 10], [11, 12]]);
        let c = Tensor2::mul(&a, &b);
        assert_eq!(c.nrows(), 2);
        assert_eq!(c.ncols(), 2);
        // row0: 1*7+2*9+3*11 = 58, 1*8+2*10+3*12 = 64
        assert_eq!(c.get(0, 0), Some(&58));
        assert_eq!(c.get(0, 1), Some(&64));
        // row1: 4*7+5*9+6*11 = 139, 4*8+5*10+6*12 = 154
        assert_eq!(c.get(1, 0), Some(&139));
        assert_eq!(c.get(1, 1), Some(&154));
    }

    #[test]
    fn map_and_operator_overloads() {
        let a = Tensor2::from([[1, 2], [3, 4]]);
        let doubled = a.map(|x| x * 2);
        assert_eq!(doubled.get(1, 1), Some(&8));

        let b = Tensor2::from([[1, 1], [1, 1]]);
        assert_eq!((a + b).get(0, 0), Some(&2));
        assert_eq!((a - b).get(1, 1), Some(&3));
    }

    #[test]
    fn default_is_zeros() {
        let m: Tensor2<i32, 2, 3> = Tensor2::default();
        assert_eq!(m.get(1, 2), Some(&0));
    }
}
