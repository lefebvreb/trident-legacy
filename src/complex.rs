use std::fmt;

use crate::approx_eq;

//#################################################################################################
//
//                                       Complex type
//
//#################################################################################################

/// Represents a complex number with two single precision (32 bits) floating point.
/// numbers.
#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Copy, PartialEq, Clone, Default)]
pub struct c64(f32, f32);

impl c64 {
    /// The complex reprensenting zero.
    pub const ZERO: c64 = c64(0f32, 0f32);
    /// The complex reprensenting one.
    pub const ONE: c64 = c64(1f32, 0f32);
    /// The complex reprensenting i, the imaginary unit.
    pub const I: c64 = c64(0f32, 1f32);

    /// Constructs a new complex from it's real part and imaginary part.
    #[inline]
    pub fn new(re: f32, im: f32) -> c64 {
        c64(re, im)
    }

    /// Constructs a new complex from it's radius and argument.
    #[inline]
    pub fn new_euler(r: f32, arg: f32) -> c64 {
        c64(r*arg.cos(), r*arg.sin())
    }

    /// Returns the complex conjugate of `self`.
    #[inline]
    pub fn conjugate(self) -> c64 {
        c64(self.0, -self.1)
    }

    /// Returns the multiplicative inverse of `self`.
    #[inline]
    pub fn recip(self) -> c64 {
        let d = self.norm_sqr().recip();
        c64(self.0*d, -self.1*d)
    }

    /// Returns the square of the norm of `self`.
    #[inline]
    pub fn norm_sqr(self) -> f32 {
        self.0*self.0 + self.1*self.1
    }

    /// Returns the norm of `self`.
    #[inline]
    pub fn norm(self) -> f32 {
        self.norm_sqr().sqrt()
    }

    /// Compare two complex numbers and returns true they parts are pairwise equal with 
    /// the `EPSILON` precision, where `EPSILON` is about `1e-7`.
    #[inline]
    pub fn approx_eq(self, rhs: c64) -> bool {
        approx_eq(self.0, rhs.0) && approx_eq(self.1, rhs.1)
    }
}

unsafe impl ocl::traits::OclPrm for c64 {}

//#################################################################################################
//
//                                    Various implementations
//
//#################################################################################################

macro_rules! impl_from_primitive {
    {$($from: ty),*} => {
        $(
            impl From<$from> for c64 {
                fn from(x: $from) -> c64 {
                    c64(x as f32, 0.0)
                }
            }
        )*        
    };
}

impl_from_primitive! {
    i8, i16, i32, i64, i128, isize,
    u8, u16, u32, u64, u128, usize,
    f32, f64
}

impl fmt::Debug for c64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.5e}{:+.5e}i", self.0, self.1)
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

impl std::ops::Div<c64> for c64 {
    type Output = c64;

    fn div(self, rhs: c64) -> c64 {
        let d = rhs.norm_sqr().recip();

        c64(
            (self.0*rhs.0 + self.1*rhs.1) * d,
            (self.1*rhs.0 - self.0*rhs.1) * d,
        )
    }
}
