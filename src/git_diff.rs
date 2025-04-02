use std::process::{Command, exit};

pub fn get_git_diff() -> String {
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
