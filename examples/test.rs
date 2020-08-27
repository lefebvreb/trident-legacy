use qrust::{Computer, Gate, Program};

// cargo run --example test --release

fn main() {
    let hadamard_gate = { 
        let sqrt2inv = 2f32.sqrt().recip();
        Gate::new(
            sqrt2inv, sqrt2inv,
            sqrt2inv, -sqrt2inv,
        )
    };

    let mut computer = Computer::new(4)
        .add_gate("H", hadamard_gate)
        .build();

    let program = Program::new(0b000)
        .apply(0, "H")
        .apply(1, "H")
        .apply(2, "H")
        .apply(3, "H")
        .measure(16);

    let result = computer.compile_and_run(program, None);

    for r in result.iter() {
        println!("|{:04b}>", *r);
    }
}
