use std::{io, process::Command};

pub fn commit(message: &str) -> Result<(), io::Error> {
    let output = Command::new("git")
        .args(["commit", "-m", &message])
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            String::from_utf8_lossy(&output.stderr),
        ))
    }
}
