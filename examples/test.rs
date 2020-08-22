use quantum::complex::c64;
use quantum::computer::Computer;
use quantum::program::Program;
use quantum::gates::Gate;

// cargo run --example test --release

fn main() {
    let sqrt2inv = c64::real(2f32.sqrt().recip());
    let h = Gate::new(
        [sqrt2inv, sqrt2inv],
        [sqrt2inv, -sqrt2inv],
    );

    let c = Computer::new(3)
        .add_gate("H", h)
        .build();

    let p = Program::new(5)
        .apply(0, "H")
        .apply(1, "H")
        .apply(2, "H")
        .measure(10);

    let res = c.compile_and_run(p);

    println!("{:?}", res);
}
