use std::io;
use std::process;

pub fn handle_input() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap_or_else(|_| {
        eprintln!("Failed to read input.");
        process::exit(1);
    });
    input.trim().to_lowercase()
}
