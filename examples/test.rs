use trident::{Computer, Gate, Program};

// cargo run --example test --release

fn main() {
    let hadamard_gate = { 
        let sqrt2inv = 2f32.sqrt().recip();
        Gate::new(
            sqrt2inv, sqrt2inv,
            sqrt2inv, -sqrt2inv,
        )
    };

    let mut computer = Computer::new(10)
        .add_gate("H", hadamard_gate)
        .build();

    let program = Program::new(&computer, "|0000000000>")
        .apply_range(0..10, "H", None)
        .measure(1024);

    let mut result = computer.run(program, None);

    result.format_options(None, 0);

    println!("{}", result);
}
