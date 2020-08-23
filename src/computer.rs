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

    /*pub fn add_standard_gates(mut self) -> ComputerBuilder {
        unimplemented!()
    }*/

    pub fn build(self) -> Computer {
        let size = self.size;

        let pro_que = ProQue::builder()
            .src(include_str!("opencl/kernels.cl"))
            .dims(1 << size)
            .build()
            .expect("Cannot build compute shader");

        let amplitudes = pro_que.create_buffer::<c64>()
            .expect("Cannot create amplitudes buffer");

        let probabilities = pro_que.create_buffer::<f32>()
            .expect("Cannot create probabilities buffer");

        let distribution = pro_que.create_buffer::<f32>()
            .expect("Cannot create distribution buffer");

        let gates = self.gates;

        let apply_gate = pro_que.kernel_builder("apply_gate")
            .arg(&amplitudes)
            .arg(0u8)
            .arg(c64::ZERO)
            .arg(c64::ZERO)
            .arg(c64::ZERO)
            .arg(c64::ZERO)
            .build()
            .expect("Cannot build kernel `apply_gate`");

        let apply_controlled_gate = pro_que.kernel_builder("apply_controlled_gate")
            .arg(&amplitudes)
            .arg(0u8)
            .arg(c64::ZERO)
            .arg(c64::ZERO)
            .arg(c64::ZERO)
            .arg(c64::ZERO)
            .arg(0u8)
            .build()
            .expect("Cannot build kernel `apply_controlled_gate`");

        let calculate_probabilities = pro_que.kernel_builder("calculate_probabilities")
            .arg(&amplitudes)
            .arg(&probabilities)
            .build()
            .expect("Cannot build kernel `calculate_probabilities`");

        let calculate_distribution = pro_que.kernel_builder("calculate_distribution")
            .arg(&probabilities)
            .arg(&distribution)
            .arg(size)
            .arg(0u8)
            .build()
            .expect("Cannot build kernel `calculate_distribution`");
            
        Computer {
            size,
            amplitudes,
            probabilities,
            distribution,
            gates,
            apply_gate,
            apply_controlled_gate,
            calculate_probabilities,
            calculate_distribution
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
    amplitudes: Buffer<c64>,
    probabilities: Buffer<f32>,
    distribution: Buffer<f32>,
    gates: HashMap<&'static str, Gate>,
    //initialize: Kernel,
    apply_gate: Kernel,
    apply_controlled_gate: Kernel,
    calculate_probabilities: Kernel,
    calculate_distribution: Kernel,
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
        if program.initial_state >= (1usize << self.size) {
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

        // Initialization of amplitudes buffer
        unsafe {
            *self.amplitudes.map()
                .write_invalidate()
                .enq()
                .expect("Cannot access the amplitudes buffer")
                .get_mut(program.initial_state)
                .expect(format!(
                    "Cannot access element #{} in amplitudes buffer",
                    program.initial_state,
                ).as_str())
            = c64::ONE;
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
                    .expect("Cannot call kernel `apply_gate` or `apply_gate_controlled`");
            }
        }

        //Measuring
        unsafe { 
            self.calculate_probabilities.enq()
                .expect("Cannot call kernel `calculate_probabilities`");
        }

        // Display amplitudes
        {
            let mut vec = vec![c64::ZERO; self.amplitudes.len()];
            self.amplitudes.read(&mut vec).enq().unwrap();
            println!("{:?}", vec);
        }

        // Display probabilites
        {
            let mut vec = vec![0.0; self.probabilities.len()];
            self.probabilities.read(&mut vec).enq().unwrap();
            println!("{:?}", vec);
        }

        vec![].into()
    }
}