use std::process::Command;

pub fn commit(message: &str) {
    let output = Command::new("git")
        .args(["commit", "-m", &message])
        .output()
        .unwrap();

    if output.status.success() {
        println!("Commit successful");
    } else {
        eprintln!("Commit failed: {}", String::from_utf8_lossy(&output.stderr));
    }
}
