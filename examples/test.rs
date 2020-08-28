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

    let mut computer = Computer::new(3)
        .add_gate("H", hadamard_gate)
        .build();

    let program = Program::new(0b000)
        .apply_range(0..3, "H", None)
        .measure(4096);

    let mut result = computer.compile_and_run(program, None);

    result.format_options(0.0, Some(3));

    println!("{}", result);
}
