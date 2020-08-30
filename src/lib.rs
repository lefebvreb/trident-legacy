// TODO

// Add standard gates
// Impl Display for computer, program...
// Bench and maybe improve random generator
// State type
// Tests
// Examples
// Comment code
// Doc
// Benches ?
// Constify ?
// Do kernel computation with double precision ?
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

// Returns true if x and y are equals with EPSILON precision
#[inline]
pub(crate) fn approx_eq(x: f32, y: f32) -> bool {
    const EPSILON: f32 = 1e-7f32;
    (x - y).abs() < EPSILON
}

// Exports
pub use complex::c64;
pub use computer::{Address, Computer, ComputerBuilder};
pub use gates::Gate;
pub use measure::Measurements;
pub use program::{InstructionChain, Program, ProgramBuilder};
