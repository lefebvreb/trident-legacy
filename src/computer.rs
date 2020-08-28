use ocl::{Buffer, Kernel, ProQue};

use std::collections::HashMap;
use std::time::{Instant, SystemTime};

use crate::complex::c64;
use crate::gates::Gate;
use crate::measure::Measurements;
use crate::program::Program;

pub type Address = u8;

//#################################################################################################
//
//                                       Computer Builder
//
//#################################################################################################

#[derive(Debug)]
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

        let main_buffer = pro_que.create_buffer()
            .expect("Cannot create main buffer");

        let measurements_buffer = pro_que.buffer_builder()
            .len(Computer::MEASUREMENTS_BLOCK)
            .build()
            .expect("Cannot create measurements buffer");

        let gates = self.gates;

        let apply_gate = pro_que.kernel_builder("apply_gate")
            .arg(&main_buffer)
            .arg(0u8)
            .arg(c64::ZERO)
            .arg(c64::ZERO)
            .arg(c64::ZERO)
            .arg(c64::ZERO)
            .build()
            .expect("Cannot build kernel `apply_gate`");

        let apply_controlled_gate = pro_que.kernel_builder("apply_controlled_gate")
            .arg(&main_buffer)
            .arg(0u8)
            .arg(c64::ZERO)
            .arg(c64::ZERO)
            .arg(c64::ZERO)
            .arg(c64::ZERO)
            .arg(0u8)
            .build()
            .expect("Cannot build kernel `apply_controlled_gate`");

        let calculate_probabilities = pro_que.kernel_builder("calculate_probabilities")
            .arg(&main_buffer)
            .build()
            .expect("Cannot build kernel `calculate_probabilities`");

        let reduce_distribution = pro_que.kernel_builder("reduce_distribution")
            .arg(&main_buffer)
            .arg(0u8)
            .global_work_size(dim >> 1)
            .build()
            .expect("Cannot build kernel `reduce_distribution`");

        let do_measurements = pro_que.kernel_builder("do_measurements")
            .arg(&main_buffer)
            .arg(&measurements_buffer)
            .arg(size)
            .arg(0u32)
            .global_work_size(Computer::MEASUREMENTS_BLOCK)
            .build()
            .expect("Cannot build kernel `do_measurements`");
            
        Computer {
            size,
            gates,
            main_buffer,
            measurements_buffer,
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

#[derive(Debug)]
pub struct Computer {
    size: Address,
    gates: HashMap<&'static str, Gate>,
    main_buffer: Buffer<c64>,
    measurements_buffer: Buffer<u64>,
    apply_gate: Kernel,
    apply_controlled_gate: Kernel,
    calculate_probabilities: Kernel,
    reduce_distribution: Kernel,
    do_measurements: Kernel,
}

impl Computer {
    const MEASUREMENTS_BLOCK: usize = 1024;

    pub fn new(size: Address) -> ComputerBuilder {
        if size == 0 {
            panic!("Computer's register's size is 0, it should be at least 1")
        }

        ComputerBuilder {
            size,
            gates: HashMap::new(),
        }
    }

    pub fn compile_and_run(&mut self, program: Program, seed: Option<u32>) -> Measurements {
        let start = Instant::now();
        let dim = 1 << self.size;

        // Checks aka 'compilation'
        if program.initial_state >= (dim) {
            panic!(
                "Initial state `|{:b}>` can't be represented with a {}-sized register, it needs at least {} qbits", 
                program.initial_state, 
                self.size,
                64 - program.initial_state.leading_zeros(),
            );
        }

        for instruction in program.instructions.iter() {
            if instruction.target >= self.size {
                panic!(
                    "Target's address (#{}) is out of the {}-sized register", 
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
                        "Control's address (#{}) is out of the {}-sized register", 
                        instruction.target,
                        self.size,
                    );
                }
            }
        }

        // Initialization of amplitudes buffer
        unsafe {
            *self.main_buffer.map()
                .write_invalidate()
                .enq()
                .expect("Cannot write to the main buffer")
                .get_mut(program.initial_state)
                .expect("Cannot access element in the main buffer")
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

        // Reduce probabilities
        {
            let mut worksize = dim >> 1;

            for pass in 1..self.size {
                self.reduce_distribution.set_default_global_work_size(worksize.into());
                self.reduce_distribution.set_arg(1, pass).unwrap();

                unsafe {
                    self.reduce_distribution.enq()
                        .expect("Cannot call kernel `reduce_distribution`");
                }

                worksize >>= 1;
            }
        }

        // Display probabilites
        /*{
            let mut vec = vec![c64::ZERO; self.main_buffer.len()];
            self.main_buffer.read(&mut vec).enq().unwrap();
            println!("P = {:?}", vec);
        }*/

        {
            let mut seed = match seed {
                Some(s) => s,
                None => !(SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .expect("Duration since UNIX_EPOCH failed")
                    .as_secs()
                    & 0xFFFFFFFF
                ) as u32,
            };

            let mut buffer = vec![0; Computer::MEASUREMENTS_BLOCK];
            let mut results = HashMap::with_capacity(program.samples);
            let mut remaining = program.samples;

            while remaining != 0 {
                let measures = std::cmp::min(remaining, Computer::MEASUREMENTS_BLOCK);
                remaining -= measures;

                seed ^= seed.wrapping_shl(13);
                seed ^= seed.wrapping_shr(17);
                seed ^= seed.wrapping_shl(5);

                self.do_measurements.set_arg(3, seed).unwrap();

                unsafe {
                    self.do_measurements.enq()
                        .expect("Cannot call kernel `do_measurements`");
                }

                self.measurements_buffer.read(&mut buffer)
                    .enq()
                    .expect("Cannot read from buffer `measurements`");

                for i in 0..measures {
                    let state = buffer[i];

                    if let Some(freq) = results.get_mut(&state) {
                        *freq += 1;
                    } else {
                        results.insert(state, 1);
                    }
                }
            }

            Measurements::new(
                Instant::now().duration_since(start),
                self.size,
                program.samples,
                results,
            )
        }
    }
}