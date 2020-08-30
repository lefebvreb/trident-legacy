use trident::{Computer, InstructionChain};

fn main() {
    // Creates a new computer with 3 qbits.
    let mut computer = Computer::new(3)
        .add_default_gates()
        .build();

    // Initialize the state with |000> and applies the
    // Hadamard gate to all three qbits.
    // Performs 5000 measurements.
    let program = computer.new_program("|000>")
        .apply_iter("H", 0..3, None)
        .measure(5000);

    // Runs the program and prints the results.
    let results = computer.run(program, None);
    println!("{}", results);
}
