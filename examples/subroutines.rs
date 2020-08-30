use trident::Computer;

fn main() {
    let mut computer = Computer::new(3)
        .add_default_gates()
        .build();
}