// TODO

// Make Program creation bouded to computer and check validity here
// Comment code
// Various optimizations
// Add standard gates
// Pretty print computer, program...
// Tests
// Examples
// Doc
// Benches ?
// Constify ?
// Do kernel computation with double precision ?
// Publish ??

extern crate ocl;

// Modules
mod complex;
mod computer;
mod gates;
mod measure;
mod program;

// Returns true if x and y are equals with EPSILON precision
#[inline]
pub(crate) fn approx_eq(x: f32, y: f32) -> bool {
    const EPSILON: f32 = 1e-7f32;
    (x - y).abs() < EPSILON
}

// Exports
pub use complex::c64;
pub use computer::{Computer, ComputerBuilder};
pub use gates::Gate;
pub use measure::Measurements;
pub use program::{Program, ProgramBuilder};