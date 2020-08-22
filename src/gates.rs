use crate::complex::c64;
use crate::error::QRustResult;

use ocl::{Buffer, Kernel, ProQue};

#[repr(u8)]
pub(crate) enum GateKernel {
    Unitary(Kernel),
    Controlled(Kernel),
}

pub struct Gate {
    controlled: bool,
    coefficients: (c64, c64, c64, c64),
}

impl Gate {
    pub const fn new(coefficients: [[c64; 2]; 2]) -> Gate {
        Gate {
            controlled: false,
            coefficients: (
                coefficients[0][0], 
                coefficients[0][1], 
                coefficients[1][0], 
                coefficients[1][1]
            ),
        }
    }

    pub const fn new_controlled(coefficients: [[c64; 2]; 2]) -> Gate {
        Gate {
            controlled: true,
            coefficients: (
                coefficients[0][0], 
                coefficients[0][1], 
                coefficients[1][0], 
                coefficients[1][1]
            ),
        }
    }

    pub(crate) fn into_kernel(&self, buffer: &Buffer<c64>, pro_que: &ProQue) -> QRustResult<GateKernel> {
        Ok(if self.controlled {
            GateKernel::Controlled(pro_que.kernel_builder("apply_controlled_gate")
                .arg(buffer)
                .arg(self.coefficients.0)
                .arg(self.coefficients.1)
                .arg(self.coefficients.2)
                .arg(self.coefficients.3)
                .arg(0)
                .arg(0)
                .build()?)
        } else {
            GateKernel::Unitary(pro_que.kernel_builder("apply_gate")
                .arg(buffer)
                .arg(self.coefficients.0)
                .arg(self.coefficients.1)
                .arg(self.coefficients.2)
                .arg(self.coefficients.3)
                .arg(0)
                .build()?)
        })
    }
}