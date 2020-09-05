use trident::{Computer, InstructionChain};

fn main() {
    // Creates a new computer with 3 qbits.
    let mut computer = Computer::new(10)
        .add_default_gates()
        .build();

    // Initialize the state with |000> and applies the
    // Hadamard gate to all three qbits.
    // Performs 5000 measurements.
    let program = computer.new_program("|0000000000>")
        .apply_iter("H", 0..10, None)
        .measure(4096);

    // Runs the program and prints the results.
    let results = computer.run(program, None);
    println!("{}", results);
}
