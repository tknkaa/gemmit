mod ai;
mod config;
mod git_diff;

use ai::ask_ai;
use config::load_api_key;
use git_diff::get_git_diff;
use std::process::Command;
use tokio;

#[tokio::main]
async fn main() {
    /*     let prompt_diff = match get_git_diff() {
        Ok(diff) => diff,
        Err(_) => {
            println!("Failed to get git diff");
            return;
        }
    };

    let api_key = load_api_key();
    let prompt = format!(
        "Based on the following changes, suggest a concise and appropriate commit message: {}. Just the commit message, please.",
        prompt_diff
    );

    match ask_ai(&api_key, &prompt).await {
        Ok(response) => println!("{response}"),
        Err(_) => println!("Failed to ask AI"),
    } */

    let message = "test";
    let output = Command::new("git")
        .args(["commit", "-m", message])
        .output()
        .unwrap();

    if output.status.success() {
        println!("Commit successful");
    } else {
        eprintln!("Commit failed: {}", String::from_utf8_lossy(&output.stderr));
    }
}
