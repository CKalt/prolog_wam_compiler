use prolog_wam_compiler::{WamEmulator, Term};

fn main() {
    let term = Term::Atom("example".to_string());

    let mut emulator = WamEmulator::new();
    let index = emulator.push_term(&term);

    println!("Term pushed to heap at index: {}", index);
}
