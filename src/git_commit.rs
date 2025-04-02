use std::{process, process::Command};

pub fn commit(message: &str, input: &str) {
    if input == "y" || input.is_empty() {
        match Command::new("git")
            .args(["commit", "-m", &message])
            .output()
        {
            Ok(_) => {
                println!("Commit successful.");
            }
            Err(err) => {
                eprintln!("Error: {}", err);
                process::exit(1);
            }
        }
    } else {
        eprintln!("Commit canceled.");
        process::exit(1);
    }
}
