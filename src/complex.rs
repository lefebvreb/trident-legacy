use std::fmt;

#[allow(non_camel_case_types)]
#[derive(Copy, PartialEq, Clone, Default)]
pub struct c64(f32, f32);

unsafe impl ocl::traits::OclPrm for c64 {}

impl c64 {
    #[inline]
    pub const fn new(re: f32, im: f32) -> c64 {
        c64(re, im)
    }
}

impl fmt::Debug for c64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}{:+?}i", self.0, self.1)
    }
}