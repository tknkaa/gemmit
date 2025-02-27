use std::process::Command;
use std::io;

pub fn get_git_diff() -> Result<String, io::Error> {
    let output = Command::new("git")
    .args(["diff", "--cached"])
    .output()
    .unwrap();

    let diff = String::from_utf8_lossy(&output.stdout);
    println!("{:#?}", diff);

    Ok(String::from("hello"))
}
