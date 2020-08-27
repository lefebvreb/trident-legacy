// TODO

// Make Program creation bouded to computer and check validity here
// Result type for computation with display and all that jazz
// Comment code
// Various optimizations
// Constify ?

mod complex;
mod computer;
mod gates;
mod program;

#[inline]
pub(crate) fn approx_eq(x: f32, y: f32) -> bool {
    const EPSILON: f32 = 1e-7f32;
    (x - y).abs() < EPSILON
}

pub use complex::c64;
pub use computer::{Computer, ComputerBuilder};
pub use gates::Gate;
pub use program::{Program, ProgramBuilder};