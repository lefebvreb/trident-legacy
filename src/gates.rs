use crate::complex::c64;
use crate::EPSILON;

pub struct Gate {
    pub(crate) coefficients: (c64, c64, c64, c64),
}

impl Gate {
    pub const fn new(row0: [c64; 2], row1: [c64; 2]) -> Gate {
        Gate {
            coefficients: (
                row0[0], row0[1], 
                row1[0], row1[1],
            ),
        }
    }

    // U = [a b]
    //     [c d]
    // UU* == 1
    pub(crate) fn is_unitary(&self) -> bool {
        let (a, b, c, d) = self.coefficients;

        (a.norm_sqr() + c.norm_sqr() - 1f32).abs() < EPSILON &&
        (a*b.conjugate() + c*d.conjugate()).approx_eq(c64::ZERO) &&
        (b.norm_sqr() + d.norm_sqr() - 1f32).abs() < EPSILON
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_unitary() {
        let h = { 
            let sqrt2inv = c64::real(2f32.sqrt().recip());
            Gate::new(
                [sqrt2inv, sqrt2inv], 
                [sqrt2inv, -sqrt2inv],
            )
        };
        assert!(h.is_unitary());

        let x = Gate::new(
            [c64::ZERO, c64::ONE],
            [c64::ONE, c64::ZERO],
        );
        assert!(x.is_unitary());

        let y = Gate::new(
            [c64::ZERO, -c64::I],
            [c64::I, c64::ZERO],
        );
        assert!(y.is_unitary());
    }
}