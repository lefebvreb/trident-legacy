use crate::complex::{approx_eq, c64};

pub struct Gate {
    pub(crate) coefficients: (c64, c64, c64, c64),
}

impl Gate {
    pub fn new<E1, E2, E3, E4>(u00: E1, u01: E2, u10: E3, u11: E4) -> Gate
    where 
        E1: Into<c64> + Copy,
        E2: Into<c64> + Copy,
        E3: Into<c64> + Copy,
        E4: Into<c64> + Copy,
    {
        let gate = Gate::new_unchecked(u00, u01, u10, u11);

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

    #[inline]
    pub fn new_unchecked<E1, E2, E3, E4>(u00: E1, u01: E2, u10: E3, u11: E4) -> Gate
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

    // U = [a b]
    //     [c d]
    // UU* == 1
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