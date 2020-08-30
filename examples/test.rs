use trident::{InstructionChain, Computer, Gate};

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

    let program = computer.new_program("|0000000000>")
        .apply("H", 0, None)
        .apply_iter("H", 1..5, None)
        .apply_iter("H", vec![6, 7, 9, 8], None)
        .measure(1024);

    let mut result = computer.run(program, None);

    result.format_options(None, 20);
    println!("{}", result);
}
