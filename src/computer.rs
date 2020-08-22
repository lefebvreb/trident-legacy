use ocl::{Buffer, Kernel, ProQue};

use std::collections::HashMap;

use crate::complex::c64;
use crate::gates::Gate;
use crate::program::Program;

pub type Address = u8;

//#################################################################################################
//
//                                       Computer Builder
//
//#################################################################################################

pub struct ComputerBuilder {
    size: Address,
    gates: HashMap<&'static str, Gate>,
}

impl ComputerBuilder {
    pub fn add_gate(mut self, gate_name: &'static str, gate: Gate) -> ComputerBuilder {
        if !gate.is_unitary() {
            panic!(
                "Gate \"{}\" is not unitary", 
                gate_name,
            );
        }
        if self.gates.insert(gate_name, gate).is_some() {
            panic!(
                "Gate name duplicata: \"{}\"", 
                gate_name
            );
        };
        self
    }

    pub fn add_standard_gates(mut self) -> ComputerBuilder {
        unimplemented!()
    }

    pub fn build(self) -> Computer {
        let size = self.size;

        let pro_que = ProQue::builder()
            .src(include_str!("opencl/kernels.cl"))
            .dims(1 << size)
            .build().expect("Cannot build compute shader");

        let amplitudes = pro_que.create_buffer::<c64>()
            .expect("Cannot create amplitudes buffer");

        let gates = self.gates;

        let initialize = pro_que.kernel_builder("initialize")
            .arg(&amplitudes)
            .arg(0u64)
            .build()
            .expect("Cannot build `initialize` kernel");

        let apply_gate = pro_que.kernel_builder("apply_gate")
            .arg(&amplitudes)
            .arg(0u8)
            .arg(c64::ZERO)
            .arg(c64::ZERO)
            .arg(c64::ZERO)
            .arg(c64::ZERO)
            .build()
            .expect("Cannot build `apply_gate` kernel");

        let apply_controlled_gate = pro_que.kernel_builder("apply_controlled_gate")
            .arg(&amplitudes)
            .arg(0u8)
            .arg(c64::ZERO)
            .arg(c64::ZERO)
            .arg(c64::ZERO)
            .arg(c64::ZERO)
            .arg(0u8)
            .build()
            .expect("Cannot build `apply_controlled_gate` kernel");

        Computer {
            size,
            _pro_que: pro_que,
            _amplitudes: amplitudes,
            gates,
            initialize,
            apply_gate,
            apply_controlled_gate,
        }
    }
}

//#################################################################################################
//
//                                        Computer
//
//#################################################################################################

pub struct Computer {
    size: Address,
    _pro_que: ProQue,
    _amplitudes: Buffer<c64>,
    gates: HashMap<&'static str, Gate>,
    initialize: Kernel,
    apply_gate: Kernel,
    apply_controlled_gate: Kernel,
}

impl Computer {
    pub fn new(size: Address) -> ComputerBuilder {
        ComputerBuilder {
            size,
            gates: HashMap::new(),
        }
    }

    pub fn compile_and_run(&self, program: Program) -> Box<[u64]> {
        // Checks aka 'compilation'
        if program.initial_state >= (1u64 << self.size) {
            panic!(
                "Initial state `{}` can't be represented with a `{}` qbits register", 
                program.initial_state, 
                self.size,
            );
        }

        for instruction in program.instructions.iter() {
            if instruction.target >= self.size {
                panic!(
                    "Target's address `{}` is out of the `{}` qbits register", 
                    instruction.target,
                    self.size,
                );
            } 
            if !self.gates.contains_key(instruction.gate_name) {
                panic!(
                    "No gate associated to the name \"{}\"", 
                    instruction.gate_name,
                );
            }
            if let Some(control) = instruction.control {
                if control >= self.size {
                    panic!(
                        "Control's address `{}` is out of the `{}` qbits register", 
                        instruction.target,
                        self.size,
                    );
                }
            }
        }

        // Initialization of register
        self.initialize.set_arg(1, program.initial_state).unwrap();
        unsafe { 
            self.initialize.enq()
                .expect("Cannot call `initialize` kernel")
        }

        // Apply gates
        for instruction in program.instructions.iter() {
            let target = instruction.target;
            let gate = self.gates.get(instruction.gate_name).unwrap();
            let kernel: &Kernel;

            if let Some(control) = instruction.control {
                kernel = &self.apply_controlled_gate;
                kernel.set_arg(6, control).unwrap();
            } else {
                kernel = &self.apply_gate;
            }

            kernel.set_arg(1, target).unwrap();
            kernel.set_arg(2, gate.coefficients.0).unwrap();
            kernel.set_arg(3, gate.coefficients.1).unwrap();
            kernel.set_arg(4, gate.coefficients.2).unwrap();
            kernel.set_arg(5, gate.coefficients.3).unwrap();

            unsafe { 
                kernel.enq()
                    .expect("Cannot call `apply_gate` or `apply_gate_controlled` kernel") 
            }
        }

        {
            let mut vec = vec![c64::ZERO; self._amplitudes.len()];
            self._amplitudes.read(&mut vec).enq().unwrap();
            println!("{:?}", vec);
        }

        vec![].into()
    }
}