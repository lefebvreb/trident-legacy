use std::fmt::{Debug, Formatter, Result};
use std::ops::{Add, Sub, Mul, Div};

use packed_simd::{f64x2, f64x4};

//#################################################################################################
//
//                                          c64 type
//
//#################################################################################################

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq)]
pub struct c64(f64x2);

impl c64 {
    #[inline]
    pub fn new(re: f64, im: f64) -> c64 {
        c64(f64x2::new(re, im))
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
    pub fn eq<const E: f64>(self, rhs: c64) -> bool {
        (self.0 - rhs.0).abs().le(f64x2::splat(E)).all()
    }
}

//#################################################################################################
//
//                                       c64 operations
//
//#################################################################################################

impl Add<c64> for c64 {
    type Output = c64;

    #[inline]
    fn add(self, rhs: c64) -> c64 {
        c64(self.0 + rhs.0)
    }
}

impl Sub<c64> for c64 {
    type Output = c64;

    #[inline]
    fn sub(self, rhs: c64) -> c64 {
        c64(self.0 - rhs.0)
    }
}

impl Mul<c64> for c64 {
    type Output = c64;

    #[inline]
    fn mul(self, rhs: c64) -> c64 {
        let tmp = f64x4::mul(
            f64x4::new(self.re(), self.im(), self.re(), self.im()),
            f64x4::new(rhs.re(), -rhs.im(), rhs.im(), rhs.re()),
        );
        unsafe {
            c64(f64x2::add(
                f64x2::new(tmp.extract_unchecked(0), tmp.extract_unchecked(2)),
                f64x2::new(tmp.extract_unchecked(1), tmp.extract_unchecked(3)),
            ))
        }
    }
}

impl Div<c64> for c64 {
    type Output = c64;

    #[inline]
    fn div(self, rhs: c64) -> c64 {
        let tmp = f64x4::mul(
            f64x4::new(self.re(), self.im(), self.im(), -self.re()),
            f64x4::new(rhs.re(), rhs.im(), rhs.re(), rhs.im()),
        );
        unsafe {
            c64(f64x2::mul(
                f64x2::add(
                    f64x2::new(tmp.extract_unchecked(0), tmp.extract_unchecked(2)),
                    f64x2::new(tmp.extract_unchecked(1), tmp.extract_unchecked(3)),
                ),
                f64x2::splat(rhs.norm_sqr().recip()),
            ))
        }
    }
}

//#################################################################################################
//
//                                         c64 fmt
//
//#################################################################################################

impl Debug for c64 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
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
    fn add_sub_test() {
        let z1 = c64::new(12.0, 53.5);
        let z2 = c64::new(-67.0, 18.5);

        let add = z1 + z2;
        assert!(c64::eq::<1e-14>(add, c64::new(-55.0, 72.0)));

        let sub = z1 - z2;
        assert!(c64::eq::<1e-14>(sub, c64::new(79.0, 35.0)));
    }

    #[test]
    fn mul_div_test() {
        let z1 = c64::new(12.0, 53.5);
        let z2 = c64::new(-67.0, 18.5);

        let mul = z1 * z2;
        assert!(c64::eq::<1e-14>(mul, c64::new(-1793.75, -3362.5)));

        let div = z1 / z2;
        assert!(c64::eq::<1e-14>(div, c64::new(0.0384476067270375, -0.787891332470893)));
    }

    #[bench]
    fn add_bench(b: &mut Bencher) {
        let z1 = c64::new(12.0, 53.5);
        let z2 = c64::new(-67.0, 18.5);

        b.iter(|| {
            for _ in 0..100 {
                black_box(black_box(z1) + black_box(z2));
            }
        })
    }

    #[bench]
    fn mul_bench(b: &mut Bencher) {
        let z1 = c64::new(12.0, 53.5);
        let z2 = c64::new(-67.0, 18.5);

        b.iter(|| {
            for _ in 0..100 {
                black_box(black_box(z1) * black_box(z2));
            }
        })
    }

    #[bench]
    fn div_bench(b: &mut Bencher) {
        let z1 = c64::new(12.0, 53.5);
        let z2 = c64::new(-67.0, 18.5);

        b.iter(|| {
            for _ in 0..100 {
                black_box(black_box(z1) / black_box(z2));
            }
        })
    }
}
