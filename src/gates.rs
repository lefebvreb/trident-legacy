use crate::complex::{c64, EPSILON};

pub struct Gate {
    pub(crate) coefficients: (c64, c64, c64, c64),
}

impl Gate {
    pub const fn new(coefficients: [[c64; 2]; 2]) -> Gate {
        Gate {
            coefficients: (
                coefficients[0][0], 
                coefficients[0][1], 
                coefficients[1][0], 
                coefficients[1][1]
            ),
        }
    }

    // U = [a b]
    //     [c d]
    // UU* = I
    pub fn is_unitary(&self) -> bool {
        let (a, b, c, d) = self.coefficients;

        (a.norm_sqr() + c.norm_sqr() - 1f32).abs() < EPSILON &&
        (a*b.conjugate() + c*d.conjugate()).approx_eq(c64::ZERO) &&
        (b.norm_sqr() + d.norm_sqr() - 1f32).abs() < EPSILON
    }
}