use std::fmt;
use std::ops::{Add, Sub, Mul};

use packed_simd::f64x2;

//#################################################################################################
//
//                                          c128 type
//
//#################################################################################################

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq)]
pub struct c128(f64x2);

impl c128 {
    pub const ZERO: c128 = c128::new(0.0, 0.0);
    pub const ONE: c128 = c128::new(1.0, 0.0);

    #[inline]
    pub const fn new(re: f64, im: f64) -> c128 {
        c128(f64x2::new(re, im))
    }

    #[inline(always)]
    pub fn re(self) -> f64 {
        unsafe { self.0.extract_unchecked(0) }
    }

    #[inline(always)]
    pub fn im(self) -> f64 {
        unsafe { self.0.extract_unchecked(1) }
    }

    #[inline]
    pub fn norm_sqr(self) -> f64 {
        (self.0 * self.0).sum()
    }

    #[inline]
    pub fn eq<const E: f64>(lhs: c128, rhs: c128) -> bool {
        (lhs.0 - rhs.0).abs().le(f64x2::splat(E)).all()
    }
}

//#################################################################################################
//
//                                       c128 operations
//
//#################################################################################################

impl Add<c128> for c128 {
    type Output = c128;

    #[inline]
    fn add(self, rhs: c128) -> c128 {
        c128(self.0 + rhs.0)
    }
}

impl Sub<c128> for c128 {
    type Output = c128;

    #[inline]
    fn sub(self, rhs: c128) -> c128 {
        c128(self.0 - rhs.0)
    }
}

impl Mul<c128> for c128 {
    type Output = c128;

    #[inline]
    fn mul(self, rhs: c128) -> c128 {
        c128(f64x2::add(
            f64x2::mul(
                self.0,
                f64x2::splat(rhs.re()),
            ),
            f64x2::mul(
                f64x2::new(-self.im(), self.re()),
                f64x2::splat(rhs.im()),
            ),
        ))
    }
}

//#################################################################################################
//
//                                         c128 fmt
//
//#################################################################################################

impl fmt::Debug for c128 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}{:+?}i", self.re(), self.im())
    }
}

//#################################################################################################
//
//                                         tests
//
//#################################################################################################

#[cfg(test)]
mod tests {
    use super::*;

    use test::{Bencher, black_box};

    #[test]
    fn correctness_test() {
        let z1 = c128::new(12.0, 53.5);
        let z2 = c128::new(-67.0, 18.5);

        let add = z1 + z2;
        assert!(c128::eq::<1e-14>(add, c128::new(-55.0, 72.0)));

        let sub = z1 - z2;
        assert!(c128::eq::<1e-14>(sub, c128::new(79.0, 35.0)));

        let mul = z1 * z2;
        assert!(c128::eq::<1e-14>(mul, c128::new(-1793.75, -3362.5)));
    }


    #[bench]
    fn add_bench(b: &mut Bencher) {
        let z1 = c128::new(12.0, 53.5);
        let z2 = c128::new(-67.0, 18.5);

        b.iter(|| {
            for _ in 0..100 {
                black_box(black_box(z1) + black_box(z2));
            }
        })
    }

    #[bench]
    fn mul_bench(b: &mut Bencher) {
        let z1 = c128::new(12.0, 53.5);
        let z2 = c128::new(-67.0, 18.5);

        b.iter(|| {
            for _ in 0..100 {
                black_box(black_box(z1) * black_box(z2));
            }
        })
    }
}
