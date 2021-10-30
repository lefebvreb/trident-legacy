// TODO

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

// Modules
mod complex;
mod computer;
mod gates;
mod measure;
mod program;
mod random;

const MEASUREMENTS_BLOCK: usize = 1024;

#[inline]
pub(crate) fn approx_eq(x: f32, y: f32) -> bool {
    (x - y).abs() < f32::EPSILON
}

// Exports
pub use complex::c64;
pub use computer::{Address, Computer, ComputerBuilder};
pub use gates::Gate;
pub use measure::Measurements;
pub use program::{InstructionChain, Program, ProgramBuilder};
