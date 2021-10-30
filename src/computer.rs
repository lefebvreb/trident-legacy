use ocl::{Buffer, Kernel, ProQue};

use std::collections::HashMap;
use std::fmt;
use std::mem::swap;
use std::time::Instant;

use crate::MEASUREMENTS_BLOCK;
use crate::complex::c64;
use crate::gates::Gate;
use crate::measure::Measurements;
use crate::program::{Program, ProgramBuilder};
use crate::random::MWC64X;

/// Represents a qbit's address in the quantum computer.
pub type Address = u8;

//#################################################################################################
//
//                                       Computer Builder
//
//#################################################################################################

/// A builder for the `Computer` struct.
pub struct ComputerBuilder {
    size: Address,
    gates: HashMap<&'static str, Gate>,
    gates_inverses: HashMap<&'static str, Gate>,
    built: bool,
}

impl ComputerBuilder {
    /// Register a new gate for the Computer being build.
    /// 
    /// # Panics
    /// 
    /// This function will panic if they is already a gate named `gate_name`.
    pub fn add_gate(&mut self, gate_name: &'static str, gate: Gate) -> &mut ComputerBuilder {
        assert!(
            !self.built,
            "Computer has already been built, cannot modify it any more",
        );

        if self.gates.insert(gate_name, gate).is_some() {
            panic!(
                "Gate name duplicata: \"{}\"", 
                gate_name
            );
        };
        self.gates_inverses.insert(gate_name, gate.invert());
        self
    }

    pub fn add_default_gates(&mut self) -> &mut ComputerBuilder {
        assert!(
            !self.built,
            "Computer has already been built, cannot modify it any more",
        );

        let sqrt2inv = 2f32.sqrt().recip();

        unsafe {
            self.add_gate("1", Gate::new_unchecked(1, 0, 1, 0))
                .add_gate("H", Gate::new_unchecked(sqrt2inv,sqrt2inv, sqrt2inv, -sqrt2inv))
                .add_gate("X", Gate::new_unchecked(0, 1, 1, 0))
                .add_gate("Y", Gate::new_unchecked(0, -c64::I, c64::I, 0))
                .add_gate("Z", Gate::new_unchecked(1, 0, 0, -1))
        }
    }

    /// Builds and returns a new `Computer` from the builder and consumes it.
    /// 
    /// # Panics
    /// 
    /// This function will panic if something goes wrong when initializing opencl,
    /// compiling the shader or allocating memory on the gpu.
    pub fn build(&mut self) -> Computer {
        assert!(
            !self.built,
            "Computer has already been built, cannot modify it any more",
        );
        
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
            .len(MEASUREMENTS_BLOCK)
            .build()
            .expect("Cannot create measurements buffer");

        let gates = {
            let mut result = HashMap::with_capacity(0);
            swap(&mut self.gates, &mut result);
            result
        };

        let gates_inverses = {
            let mut result = HashMap::with_capacity(0);
            swap(&mut self.gates_inverses, &mut result);
            result
        };

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
            .arg(0u64)
            .global_work_size(MEASUREMENTS_BLOCK)
            .build()
            .expect("Cannot build kernel `do_measurements`");
            
        Computer {
            size,
            gates,
            gates_inverses,
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

/// Represents a quantum computer, with it's memory and capabilities.
pub struct Computer {
    pub(crate) size: Address,
    pub(crate) gates: HashMap<&'static str, Gate>,
    pub(crate) gates_inverses: HashMap<&'static str, Gate>,
    main_buffer: Buffer<c64>,
    measurements_buffer: Buffer<u64>,
    apply_gate: Kernel,
    apply_controlled_gate: Kernel,
    calculate_probabilities: Kernel,
    reduce_distribution: Kernel,
    do_measurements: Kernel,
}

impl<'computer> Computer {
    /// Creates a new `ComputerBuilder` struct to begin the construction of a new `Computer`.
    /// 
    /// # Panics
    /// 
    /// This function will panic if `size` is 0 or greater than the number of bits of the operating
    /// system's address size.
    pub fn new(size: Address) -> ComputerBuilder {
        if size == 0 {
            panic!("Computer's register's size is 0, it should be at least 1");
        }
        let ptr_size = 8 * std::mem::size_of::<usize>();
        if size as usize > ptr_size {
            panic!(
                "Computer's register's size is {}, but the device's address size are only {} bits wide: it needs at least {} bit(s) more",
                size,
                ptr_size,
                size as usize - ptr_size,
            );
        }

        let gates = HashMap::new();
        let gates_inverses = HashMap::new();
        let built = false;

        ComputerBuilder {
            size,
            gates,
            gates_inverses,
            built,
        }
    }

    pub fn new_program(&self, initial_state: &str) -> ProgramBuilder {
        ProgramBuilder::new(self, initial_state)
    }

    /// Runs the `program` on the computer. Uses, if provided, `seed` as the seed of the
    /// pseudo-random number generator to allow recreation of results. If `seed` is `None`, the system's
    /// time will be used as a seed.
    /// 
    /// Returns a Measurements struct, containing all needed information and results about the computation.
    /// 
    /// # Panics
    /// 
    /// This function will panic if something goes wrong while performing computations, such as the
    /// buffer being unwritable/unreadable or the kernels crashing somehow.
    pub fn run<S>(&mut self, program: Program, seed: S) -> Measurements
    where
        S: Into<Option<u64>>,
    {
        let start = Instant::now();

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

            let gate = if instruction.reverse {
                self.gates[instruction.gate_name]
            } else {
                self.gates_inverses[instruction.gate_name]
            };

            let kernel = if let Some(control) = instruction.control {
                let kernel = &self.apply_controlled_gate;
                kernel.set_arg(6, control).unwrap();
                kernel
            } else {
                &self.apply_gate
            };

            kernel.set_arg(1, target).unwrap();

            kernel.set_arg(2, gate.u00).unwrap();
            kernel.set_arg(3, gate.u01).unwrap();
            kernel.set_arg(4, gate.u10).unwrap();
            kernel.set_arg(5, gate.u11).unwrap();

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
            let mut worksize: usize = 1 << (self.size - 1);

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

        {
            let mut prng = MWC64X::new(seed.into());
            // Skips the first few numbers as they tend to be of poorer quality
            prng.skip(1000);

            let mut buffer = vec![0; MEASUREMENTS_BLOCK];
            let mut results = HashMap::with_capacity(program.samples);
            let mut remaining = program.samples;

            while remaining != 0 {
                let measures = std::cmp::min(remaining, MEASUREMENTS_BLOCK);
                remaining -= measures;

                prng.skip(MEASUREMENTS_BLOCK as u64);
                self.do_measurements.set_arg(3, prng.state()).unwrap();

                unsafe {
                    self.do_measurements.enq()
                        .expect("Cannot call kernel `do_measurements`");
                }

                self.measurements_buffer.read(&mut buffer)
                    .enq()
                    .expect("Cannot read from buffer `measurements`");

                for state in buffer.iter().take(measures) {
                    if let Some(freq) = results.get_mut(state) {
                        *freq += 1;
                    } else {
                        results.insert(*state, 1);
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

/*
/// Represents a quantum computer, with it's memory and capabilities.
pub struct Computer {
    pub(crate) size: Address,
    pub(crate) gates: HashMap<&'static str, Gate>,
    pub(crate) gates_inverses: HashMap<&'static str, Gate>,
    main_buffer: Buffer<c64>,
    measurements_buffer: Buffer<u64>,
    apply_gate: Kernel,
    apply_controlled_gate: Kernel,
    calculate_probabilities: Kernel,
    reduce_distribution: Kernel,
    do_measurements: Kernel,
}
*/

impl fmt::Display for Computer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, 
            "[\n  [Computer of size {}],\n  [Memory usage: {} bytes],\n  [Available gates: {:?}]\n]",
            self.size,
            ((1usize << self.size) + MEASUREMENTS_BLOCK) * 8,
            self.gates.keys().map(|s| *s).collect::<Box<[&'static str]>>(),
        )
    }
}