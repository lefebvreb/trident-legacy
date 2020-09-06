use trident::prelude::*;
use trident::VirtualQuantumComputer;

#[quantum_sub]
fn sub(a: &mut QBits<4>, b: &mut Qbit) {
    a[0..4] *= *(H(b) >> H^2 >> X);
}

#[quantum_main]
fn fft(x: &mut QBits<4>, y: &mut Qbits<4>) {
    x[0..4] >> y[0] *= H^5;
    y[2] *= H(y[3]);
    sub(&mut y[0..4], &mut x[1]);
}

/*
--- after expansion ---

fn sub(a: &mut DynQbits, b: &mut DynQbits) {
    a[0..4] *= !H(b) > H^2 > X;
}

fn fft(__input__: &mut DynQbits) {
    let x = &mut __input__[0..4]; // or split_at_mut or smth idk
    let y = &mut __input__[4..8];
    x[0..4] > y[0] *= !(H^5);
    y[2] *= H(y[3]);
    sub(&mut y[0..4], &mut x[1]);
}
*/

fn main() {
    let mut computer = VirtualQuantumComputer::new(8);

    let results = computer.run(fft, &["|0000>", "|1111>"]); // maybe make run async or a run_async version

    println!("{}", results.n_most(5));
}