// TODO

// Phase shift gate constructor
// Impl Display for computer, program...
// Bench and maybe improve random generator
// Tests
// Examples
// Comment code
// Doc
// Two times controlled gates ?
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
