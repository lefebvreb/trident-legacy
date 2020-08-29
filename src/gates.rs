use crate::complex::c64;

use crate::approx_eq;

/// Represents a unitary quantum gate.
#[derive(Debug)]
pub struct Gate {
    pub(crate) coefficients: (c64, c64, c64, c64),
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
                gate.coefficients.0,
                gate.coefficients.1,
                gate.coefficients.2,
                gate.coefficients.3,
            );
        }

        gate
    }

    /// Unsafe version of the Gate::new function. Serves the same purpose and does the same thing, but
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
            coefficients: (
                u00.into(), u01.into(),
                u10.into(), u11.into(),
            ),
        }
    }

    #[inline]
    pub(crate) fn is_unitary(&self) -> bool {
        let (a, b, c, d) = self.coefficients;

        approx_eq(a.norm_sqr() + c.norm_sqr(), 1f32) &&
        (a*b.conjugate() + c*d.conjugate()).approx_eq(c64::ZERO) &&
        approx_eq(b.norm_sqr() + d.norm_sqr(), 1f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_unitary() {
        let h = { 
            let sqrt2inv = 2f32.sqrt().recip();
            Gate::new(
                sqrt2inv, sqrt2inv, 
                sqrt2inv, -sqrt2inv,
            )
        };
        assert!(h.is_unitary());

        let x = Gate::new(
            0.0, 1.0,
            1.0, 0.0,
        );
        assert!(x.is_unitary());

        let y = Gate::new(
            0.0, -c64::I,
            c64::I, 0.0,
        );
        assert!(y.is_unitary());
    }
}