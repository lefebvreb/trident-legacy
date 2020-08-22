use crate::complex::c64;
use crate::error::QRustResult;
use crate::gates::{Gate, GateKernel};

use ocl::{Buffer, ProQue};

use std::collections::HashMap;

pub struct ComputerBuilder {
    size: u16,
    gates: HashMap<char, Gate>,
}

pub struct Computer {
    pub(crate) size: u16,
    pub(crate) pro_que: ProQue,
    pub(crate) buffer: Buffer<c64>,
    pub(crate) gates: HashMap<char, GateKernel>,
}

impl Computer {
    pub fn new(size: u16) -> ComputerBuilder {
        ComputerBuilder {
            size,
            gates: HashMap::new(),
        }
    }

    #[inline]
    pub(crate) fn contains_gate(&self, gate_id: &char) -> bool {
        self.gates.contains_key(gate_id)
    }
}

impl ComputerBuilder {
    pub fn add_gate(&mut self, c: char, gate: Gate) -> &mut ComputerBuilder {
        self.gates.insert(c, gate);
        self
    }

    pub fn build(&self) -> QRustResult<Computer> {
        let size = self.size;

        let pro_que = ProQue::builder()
            .src(include_str!("opencl/kernels.cl"))
            .dims(1 << size)
            .build()?;

        let buffer = pro_que.create_buffer::<c64>()?;

        let gates = {
            let mut res = HashMap::with_capacity(self.gates.capacity());
            for (k, v) in self.gates.iter() {
                res.insert(*k, v.into_kernel(&buffer, &pro_que)?);
            }
            res
        };

        Ok(Computer {
            size,
            pro_que,
            buffer,
            gates,
        })
    }
}