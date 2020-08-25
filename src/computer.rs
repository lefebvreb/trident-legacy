use ocl::{Buffer, Kernel, ProQue};

use std::collections::HashMap;
use std::time::SystemTime;

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
        let dim = 1usize << size;

        let pro_que = ProQue::builder()
            .src(include_str!("opencl/kernels.cl"))
            .dims(dim)
            .build()
            .expect("Cannot build compute shader");

        let amplitudes = pro_que.create_buffer()
            .expect("Cannot create amplitudes buffer");

        let probabilities = pro_que.create_buffer()
            .expect("Cannot create probabilities buffer");

        let distribution = pro_que.create_buffer()
            .expect("Cannot create distribution buffer");

        let measurements = pro_que.buffer_builder()
            .len(1024usize)
            .build()
            .expect("Cannot create measurements buffer");

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
            .arg(&distribution)
            .global_work_size(dim >> 1)
            .build()
            .expect("Cannot build kernel `calculate_probabilities`");

        let reduce_distribution = pro_que.kernel_builder("reduce_distribution")
            .arg(&distribution)
            .build()
            .expect("Cannot build kernel `reduce_distribution`");

        let do_measurements = pro_que.kernel_builder("do_measurements")
            .arg(&probabilities)
            .arg(&distribution)
            .arg(&measurements)
            .arg(dim as u64)
            .arg(0u32)
            .global_work_size(Computer::MEASUREMENTS_BLOCK)
            .build()
            .expect("Cannot build kernel `do_measurements`");
            
        Computer {
            size,
            gates,
            amplitudes,
            probabilities,
            distribution,
            measurements,
            apply_gate,
            apply_controlled_gate,
            calculate_probabilities,
            reduce_distribution,
            do_measurements,
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
    gates: HashMap<&'static str, Gate>,
    amplitudes: Buffer<c64>,
    probabilities: Buffer<f32>,
    distribution: Buffer<f32>,
    measurements: Buffer<u64>,
    apply_gate: Kernel,
    apply_controlled_gate: Kernel,
    calculate_probabilities: Kernel,
    reduce_distribution: Kernel,
    do_measurements: Kernel,
}

impl Computer {
    const MEASUREMENTS_BLOCK: usize = 1;

    pub fn new(size: Address) -> ComputerBuilder {
        if size == 0 {
            panic!("Computer's register's size is 0, it should be at least 1")
        }

        ComputerBuilder {
            size,
            gates: HashMap::new(),
        }
    }

    pub fn compile_and_run(&mut self, program: Program) -> Box<[u64]> {
        // Checks aka 'compilation'
        if program.initial_state >= (1usize << self.size) {
            panic!(
                "Initial state `{:b}` can't be represented with a {} sized register, it needs at least {} qbits", 
                program.initial_state, 
                self.size,
                64 - program.initial_state.leading_zeros(),
            );
        }

        for instruction in program.instructions.iter() {
            if instruction.target >= self.size {
                panic!(
                    "Target's address #{} is out of the {} sized register", 
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
                        "Control's address #{} is out of the {} sized register", 
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
                .expect("Cannot access element in amplitudes buffer")
            = c64::ONE;
        }

        // Apply gates
        for instruction in program.instructions.iter() {
            let target = instruction.target;
            let gate = self.gates.get(instruction.gate_name).unwrap();
            let kernel;

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

        // Calculate the probabilities vector
        unsafe { 
            self.calculate_probabilities.enq()
                .expect("Cannot call kernel `calculate_probabilities`");
        }

        // Reduce the probabilities to the distribution vector
        {
            let mut worksize: usize = 1usize << (self.size - 1);
            let mut offset = worksize;
            while worksize > 2 {
                worksize >>= 1;
    
                self.reduce_distribution
                    .set_default_global_work_size(worksize.into())
                    .set_default_global_work_offset(offset.into());
                
                unsafe { 
                    self.reduce_distribution.enq()
                        .expect("Cannot call kernel `reduce_distribution`");
                }
    
                offset += worksize;
            }
        }

        // Display probabilites
        {
            let mut vec = vec![0.0; self.probabilities.len()];
            self.probabilities.read(&mut vec).enq().unwrap();
            println!("P = {:?}", vec);
        }
    
        // Display distribution
        {
            let mut vec = vec![0.0; self.distribution.len()];
            self.distribution.read(&mut vec).enq().unwrap();
            println!("D = {:?}", vec);
        }

        {
            let mut seed = (SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Duration since UNIX_EPOCH failed")
                .as_secs()
                & 0xFFFFFFFF) as u32;

            let mut results = Vec::with_capacity(program.samples);
            let mut buffer = vec![0u64; Computer::MEASUREMENTS_BLOCK];
            let mut remaining = program.samples;
            let mut measures;

            while remaining != 0 {
                measures = std::cmp::min(remaining, Computer::MEASUREMENTS_BLOCK);
                remaining -= measures;

                seed ^= seed.wrapping_shl(13);
                seed ^= seed.wrapping_shr(17);
                seed ^= seed.wrapping_shl(5);

                self.do_measurements.set_arg(4, seed).unwrap();

                unsafe {
                    self.do_measurements.enq()
                        .expect("Cannot call kernel `do_measurements`");
                }

                self.measurements.read(&mut buffer)
                    .enq()
                    .expect("Cannot read from buffer `measurements`");

                for _ in 0..measures {
                    results.push(buffer.pop().unwrap());
                }
            }

            println!();
            results.into()
        }
    }
}