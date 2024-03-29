use crate::approx_eq;
use crate::complex::c64;

/// Represents a unitary quantum gate.
#[derive(Copy, Clone)]
pub struct Gate {
    pub(crate) u00: c64,
    pub(crate) u01: c64,
    pub(crate) u10: c64,
    pub(crate) u11: c64,
}

impl Gate {
    /// Creates a new gate with the four supplied components, such that the cannonical matrix of the gate
    /// is defined by
    /// 
    /// ```math
    ///     ┌            ┐
    /// U = │  u00  u01  │
    ///     │  u10  u00  │
    ///     └            ┘
    /// ```
    /// 
    /// This matrix needs to be unitary, meaning that it must satisfy the relation `UU = 1`, where `1` is the
    /// identity matrix and `U†` is the conjugate transpose of `U`.
    /// 
    /// # Panics
    /// 
    /// This function panics if the cannonical matrix is not unitary.
    pub fn new<E1, E2, E3, E4>(u00: E1, u01: E2, u10: E3, u11: E4) -> Gate
    where 
        E1: Into<c64> + Copy,
        E2: Into<c64> + Copy,
        E3: Into<c64> + Copy,
        E4: Into<c64> + Copy,
    {
        let gate = unsafe { Gate::new_unchecked(u00, u01, u10, u11) };

        if !gate.is_unitary() {
            panic!(
                "The gate defined by the matrix\n\t[{:?}\t{:?}]\n\t[{:?}\t{:?}]\nis not unitary",
                gate.u00,
                gate.u01,
                gate.u10,
                gate.u11,
            );
        }

        gate
    }

    /// Unsafe version of the `Gate::new` function. Serves the same purpose and does the same thing, but
    /// will not panic if the given matrix is not unitary.
    #[inline]
    pub unsafe fn new_unchecked<E1, E2, E3, E4>(u00: E1, u01: E2, u10: E3, u11: E4) -> Gate
    where 
        E1: Into<c64> + Copy,
        E2: Into<c64> + Copy,
        E3: Into<c64> + Copy,
        E4: Into<c64> + Copy,
    {
        Gate {
            u00: u00.into(), 
            u01: u01.into(),
            u10: u10.into(), 
            u11: u11.into(),
        }
    }

    pub fn phase_shift(phi: f32) -> Gate {
        unsafe {
            Gate::new_unchecked(1, 0, 0, c64::new_euler(1.0, phi))
        }
    }   

    pub fn invert(&self) -> Gate {
        let (a, b, c, d) = (self.u00, self.u01, self.u10, self.u11);

        let det_inv = (a*d - b*c).recip();
        Gate{
            u00: d * det_inv,
            u01: -b * det_inv,
            u10: -c * det_inv,
            u11: a * det_inv,
        }
    }

    #[inline]
    pub(crate) fn is_unitary(&self) -> bool {
        let (a, b, c, d) = (self.u00, self.u01, self.u10, self.u11);

        approx_eq(a.norm_sqr() + c.norm_sqr(), 1f32) &&
        (a*b.conjugate() + c*d.conjugate()).approx_eq(c64::ZERO) &&
        approx_eq(b.norm_sqr() + d.norm_sqr(), 1f32)
    }
}