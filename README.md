<img align="left" alt="" src="logo.svg" height="150" />

# Trident

Blazingly fast, GPU-enabled, super simple and flexible api for quantum computer emulation in rust.

Compatible with Windows, MacOS and Linux.

Built on top of [ocl](https://github.com/cogciprocate/ocl) (OpenCL bindings for rust).

## Features

+ Easy to learn and well documented api.
+ Fast simulation of dozens of qbits thanks to a GPU implementation.
+ Default gates (Identity, Hadamard, X, Y, Z) and custom gates generators (Phase shift, unitary, ...).
+ Subroutine system to reuse circuits inside a program.
+ Automatic generation of controlled versions of gates.
+ Automatic generation of inverse gates and subroutines allowing easy uncomputation.
+ O(n) algorithm for measurements making sampling quick and effective.

## Getting started

You will have to install an OpenCL library in order to compile the [ocl](https://github.com/cogciprocate/ocl) crate. They are available on all major desktop OS.

## Examples

You can find many more examples in the `examples/` directory of this repository.

Here is Hadamard's transform applied to a 3 qbits state, a very simple example:
```rust
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
```
Output:
```
[Measurements obtained in 3 ms]
[Sample count of 5000]
[Top results:
    |011> ~> 12.64%,
    |000> ~> 12.60%,
    |100> ~> 12.60%,
    |010> ~> 12.50%,
    |111> ~> 12.48%,
    |110> ~> 12.44%,
    |101> ~> 12.42%,
    |001> ~> 12.32%,
]
```

## References

Adam Kelly, [Simulating Quantum Computers Using OpenCL](https://arxiv.org/pdf/1805.00988.pdf) (pdf)

David B. Thomas, [The MWC64X Random Number Generator](http://cas.ee.ic.ac.uk/people/dt10/research/rngs-gpu-mwc64x.html)