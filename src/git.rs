use std::process;
use std::process::{Command, exit};

pub fn get_git_diff_output() -> String {
    let output = match Command::new("git").args(["diff", "--cached"]).output() {
        Ok(output) => output,
        Err(err) => {
            eprintln!("git diff --cached failed with status: {}", err);
            exit(1);
        }
    };

    let diff = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if diff.is_empty() {
        eprintln!("no changes added to commit");
        exit(1);
    }

    diff
}

pub fn run_git_commit(message: &str) {
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
}
