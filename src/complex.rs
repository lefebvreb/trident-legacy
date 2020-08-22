use std::fmt;

use crate::EPSILON;

//#################################################################################################
//
//                                       Complex type
//
//#################################################################################################

#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Copy, PartialEq, Clone, Default)]
pub struct c64(f32, f32);

impl c64 {
    pub const ZERO: c64 = c64::new(0f32, 0f32);
    pub const ONE: c64 = c64::new(1f32, 0f32);
    pub const I: c64 = c64::new(0f32, 1f32);

    #[inline]
    pub const fn new(re: f32, im: f32) -> c64 {
        c64(re, im)
    }

    #[inline]
    pub const fn real(re: f32) -> c64 {
        c64(re, 0.0)
    }

    #[inline]
    pub fn conjugate(self) -> c64 {
        c64(self.0, -self.1)
    }

    #[inline]
    pub fn norm_sqr(self) -> f32 {
        self.0*self.0 + self.1*self.1
    }

    #[inline]
    pub fn approx_eq(self, rhs: c64) -> bool {
        (self.0 - rhs.0).abs() < EPSILON && (self.1 - rhs.1).abs() < EPSILON
    }
}

unsafe impl ocl::traits::OclPrm for c64 {}

//#################################################################################################
//
//                                    Debug format for c64
//
//#################################################################################################

impl fmt::Debug for c64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}{:+?}i", self.0, self.1)
    }
}

//#################################################################################################
//
//                                   + - * operators for c64
//
//#################################################################################################

impl std::ops::Add<c64> for c64 {
    type Output = c64;

    fn add(self, rhs: c64) -> c64 {
        c64(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl std::ops::Neg for c64 {
    type Output = c64;

    fn neg(self) -> c64 {
        c64(-self.0, -self.1)
    }
}

impl std::ops::Sub<c64> for c64 {
    type Output = c64;

    fn sub(self, rhs: c64) -> c64 {
        c64(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl std::ops::Mul<c64> for c64 {
    type Output = c64;

    fn mul(self, rhs: c64) -> c64 {
        c64(
            self.0*rhs.0 - self.1*rhs.1,
            self.0*rhs.1 + self.1*rhs.0,
        )
    }
}
