use trident::{Computer, InstructionChain};

fn main() {
    // Creates a new computer with 3 qbits.
    let mut computer = Computer::new(20)
        .add_default_gates()
        .build();

    println!("{}\n", computer);

    // Initialize the state with |000> and applies the
    // Hadamard gate to all three qbits.
    // Performs 5000 measurements.
    let program = computer.new_program("|00000000000000000000>")
        .apply_iter("H", 0..20, None)
        .measure(1024);

    // Runs the program and prints the results.
    let results = computer.run(program, None);
    println!("{}", results);
}
