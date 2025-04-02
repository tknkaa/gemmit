use std::io;
use std::process::Command;

pub fn get_git_diff() -> Result<String, io::Error> {
    let output = Command::new("git").args(["diff", "--cached"]).output()?;

    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("git diff --cached failed with status: {}", output.status),
        ));
    }

    let diff = String::from_utf8_lossy(&output.stdout).to_string();

    Ok(diff)
}
