#![deny(missing_docs)]
#![allow(non_snake_case)]

use crate::curve::secq256k1::Fr;
use ark_std::{vec, vec::Vec, One, Zero};
use clear_on_drop::clear::Clear;

use crate::inner_product_proof::inner_product;

/// Represents a degree-3 vector polynomial
/// \\(\mathbf{a} + \mathbf{b} \cdot x + \mathbf{c} \cdot x^2 + \mathbf{d} \cdot x^3 \\).
#[cfg(feature = "yoloproofs")]
pub struct VecPoly3(pub Vec<Fr>, pub Vec<Fr>, pub Vec<Fr>, pub Vec<Fr>);

/// Represents a degree-6 scalar polynomial, without the zeroth degree
/// \\(a \cdot x + b \cdot x^2 + c \cdot x^3 + d \cdot x^4 + e \cdot x^5 + f \cdot x^6\\)
#[cfg(feature = "yoloproofs")]
pub struct Poly6 {
    pub t1: Fr,
    pub t2: Fr,
    pub t3: Fr,
    pub t4: Fr,
    pub t5: Fr,
    pub t6: Fr,
}

/// Provides an iterator over the powers of a `Fr`.
///
/// This struct is created by the `exp_iter` function.
pub struct FrExp {
    x: Fr,
    next_exp_x: Fr,
}

impl Iterator for FrExp {
    type Item = Fr;

    fn next(&mut self) -> Option<Fr> {
        let exp_x = self.next_exp_x;
        self.next_exp_x *= self.x;
        Some(exp_x)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::MAX, None)
    }
}

/// Return an iterator of the powers of `x`.
pub fn exp_iter(x: Fr) -> FrExp {
    let next_exp_x = Fr::one();
    FrExp { x, next_exp_x }
}

#[cfg(feature = "yoloproofs")]
impl VecPoly3 {
    pub fn zero(n: usize) -> Self {
        VecPoly3(
            vec![Fr::zero(); n],
            vec![Fr::zero(); n],
            vec![Fr::zero(); n],
            vec![Fr::zero(); n],
        )
    }

    /// Compute an inner product of `lhs`, `rhs` which have the property that:
    /// - `lhs.0` is zero;
    /// - `rhs.2` is zero;
    /// This is the case in the constraint system proof.
    pub fn special_inner_product(lhs: &Self, rhs: &Self) -> Poly6 {
        // TODO: make checks that l_poly.0 and r_poly.2 are zero.

        let t1 = inner_product(&lhs.1, &rhs.0);
        let t2 = inner_product(&lhs.1, &rhs.1) + inner_product(&lhs.2, &rhs.0);
        let t3 = inner_product(&lhs.2, &rhs.1) + inner_product(&lhs.3, &rhs.0);
        let t4 = inner_product(&lhs.1, &rhs.3) + inner_product(&lhs.3, &rhs.1);
        let t5 = inner_product(&lhs.2, &rhs.3);
        let t6 = inner_product(&lhs.3, &rhs.3);

        Poly6 {
            t1,
            t2,
            t3,
            t4,
            t5,
            t6,
        }
    }

    pub fn eval(&self, x: Fr) -> Vec<Fr> {
        let n = self.0.len();
        let mut out = vec![Fr::zero(); n];
        for i in 0..n {
            out[i] = self.0[i] + x * (self.1[i] + x * (self.2[i] + x * self.3[i]));
        }
        out
    }
}

#[cfg(feature = "yoloproofs")]
impl Poly6 {
    pub fn eval(&self, x: Fr) -> Fr {
        x * (self.t1 + x * (self.t2 + x * (self.t3 + x * (self.t4 + x * (self.t5 + x * self.t6)))))
    }
}

#[cfg(feature = "yoloproofs")]
impl Drop for VecPoly3 {
    fn drop(&mut self) {
        for e in self.0.iter_mut() {
            e.clear();
        }
        for e in self.1.iter_mut() {
            e.clear();
        }
        for e in self.2.iter_mut() {
            e.clear();
        }
        for e in self.3.iter_mut() {
            e.clear();
        }
    }
}

#[cfg(feature = "yoloproofs")]
impl Drop for Poly6 {
    fn drop(&mut self) {
        self.t1.clear();
        self.t2.clear();
        self.t3.clear();
        self.t4.clear();
        self.t5.clear();
        self.t6.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exp_2_is_powers_of_2() {
        let exp_2: Vec<_> = exp_iter(Fr::from(2u64)).take(4).collect();

        assert_eq!(exp_2[0], Fr::from(1u64));
        assert_eq!(exp_2[1], Fr::from(2u64));
        assert_eq!(exp_2[2], Fr::from(4u64));
        assert_eq!(exp_2[3], Fr::from(8u64));
    }

    #[test]
    fn test_inner_product() {
        let a = vec![
            Fr::from(1u64),
            Fr::from(2u64),
            Fr::from(3u64),
            Fr::from(4u64),
        ];
        let b = vec![
            Fr::from(2u64),
            Fr::from(3u64),
            Fr::from(4u64),
            Fr::from(5u64),
        ];
        assert_eq!(Fr::from(40u64), inner_product(&a, &b));
    }

    #[test]
    fn vec_of_scalars_clear_on_drop() {
        let mut v = vec![Fr::from(24u64), Fr::from(42u64)];

        for e in v.iter_mut() {
            e.clear();
        }

        fn flat_slice<T>(x: &[T]) -> &[u8] {
            use core::mem;
            use core::slice;

            unsafe { slice::from_raw_parts(x.as_ptr() as *const u8, mem::size_of_val(x)) }
        }

        assert_eq!(flat_slice(&v.as_slice()), &[0u8; 80][..]);
        assert_eq!(v[0], Fr::zero());
        assert_eq!(v[1], Fr::zero());
    }
}
