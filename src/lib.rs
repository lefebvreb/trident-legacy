// TODO

// DEBUGGER !!!

// Replace swap by take
// Remove useless 'static
// Tests
// Examples
// Comment code
// Doc
// Two times controlled gates ?
// Benches ?
// Constify ?
// Publish ??

// cargo doc --no-deps --open

extern crate ocl;
extern crate num_complex;

// Modules
mod complex;
mod computer;
mod gates;
mod measure;
mod program;
mod random;

const MEASUREMENTS_BLOCK: usize = 16;

#[inline]
pub(crate) fn approx_eq(x: f32, y: f32) -> bool {
    (x - y).abs() < f32::EPSILON
}

// Exports
pub use num_complex::Complex32;
pub use computer::{Address, Computer, ComputerBuilder};
pub use gates::Gate;
pub use measure::Measurements;
pub use program::{InstructionChain, Program, ProgramBuilder};
