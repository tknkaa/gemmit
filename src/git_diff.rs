use std::io;
use std::process::Command;

pub fn get_git_diff() -> Result<String, io::Error> {
    let output = Command::new("git")
        .args(["diff", "--cached"])
        .output()
        .expect("Failed to execute `git diff --cached` command");

    let diff = String::from_utf8_lossy(&output.stdout).to_string();

    Ok(diff)
}
