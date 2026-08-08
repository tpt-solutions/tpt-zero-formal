#![doc = include_str!("../README.md")]

#![no_std]
#![warn(missing_docs)]
#![forbid(unsafe_code)]

mod tensor;
mod tensor2;

pub use tensor::Tensor;
pub use tensor2::Tensor2;

#[cfg(test)]
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
mod proptests {
    use proptest::prelude::*;

    use crate::{Tensor, Tensor2};

    prop_compose! {
        fn vec3()(a in -100i64..100, b in -100i64..100, c in -100i64..100) -> [i64; 3] {
            [a, b, c]
        }
    }

    proptest! {
        #[test]
        fn dot_is_commutative(x in vec3(), y in vec3()) {
            let a = Tensor::from(x);
            let b = Tensor::from(y);
            prop_assert_eq!(a.dot(&b), b.dot(&a));
        }

        #[test]
        fn add_then_sub_is_identity(x in vec3(), y in vec3()) {
            let a = Tensor::from(x);
            let b = Tensor::from(y);
            prop_assert_eq!(Tensor::sub(&Tensor::add(&a, &b), &b), a);
        }

        #[test]
        fn transpose_involution(m in prop::array::uniform3(0i64..100)
                .prop_map(|row| [row; 3])) {
            // Build a 3x3 matrix from three identical rows (simplest non-trivial
            // source of varied values), then transpose twice.
            let mat = Tensor2::from(m);
            prop_assert_eq!(mat.transpose().transpose(), mat);
        }

        #[test]
        fn matrix_mul_dimensions(a in prop::array::uniform2(vec3()),
                b in prop::array::uniform3(prop::array::uniform2(0i64..100))) {
            // a is 2x3, b is 3x2 -> product is 2x2.
            let a = Tensor2::from(a);
            let b = Tensor2::from(b);
            let c = Tensor2::mul(&a, &b);
            prop_assert_eq!(c.nrows(), 2);
            prop_assert_eq!(c.ncols(), 2);
            // (A*B) computed against an independent reference column sum check:
            // each entry is sum_k a[r][k]*b[k][c], which must match get().
            for r in 0..2 {
                for k in 0..2 {
                    let mut expected = 0i64;
                    for mid in 0..3 {
                        expected += a.get(r, mid).copied().unwrap() * b.get(mid, k).copied().unwrap();
                    }
                    prop_assert_eq!(c.get(r, k), Some(&expected));
                }
            }
        }

        #[test]
        fn transpose_swaps_indices(m in prop::array::uniform2(vec3())) {
            let mat = Tensor2::from(m);
            let t = mat.transpose();
            for r in 0..2 {
                for c in 0..3 {
                    prop_assert_eq!(t.get(c, r), mat.get(r, c));
                }
            }
        }
    }
}
