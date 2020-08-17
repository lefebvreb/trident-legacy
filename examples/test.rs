use quantum::register::Register;

fn main() {
    let register = Register::new("|011010101011>").unwrap();

    println!("{:?}", register);
}