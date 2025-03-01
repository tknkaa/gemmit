mod ai;
mod config;
mod git_commit;
mod git_diff;

use ai::ask_ai;
use config::load_api_key;
use git_diff::get_git_diff;
use std::io::{self, Write};
use tokio;

#[tokio::main]
async fn main() {
    let prompt_diff = match get_git_diff() {
        Ok(diff) => {
            if diff.trim().is_empty() {
                panic!("Git diff is empty. No changes detected.");
            } else {
                diff
            }
        }
        Err(_) => {
            panic!("Failed to get git diff");
        }
    };

    let api_key = load_api_key();
    let prompt = format!(
        "Based on the following changes, suggest a concise and appropriate commit message: {}. Just the commit message, please.",
        prompt_diff
    );

    let mut message = String::new();
    match ask_ai(&api_key, &prompt).await {
        Ok(response) => {
            println!("Gemini suggested the following commit message.");
            message.push_str(&response);
            print!("{response}");
        }
        Err(_) => panic!("Failed to ask Gemini"),
    }

    print!("Do you want to commit with this message? [Y/n] ");
    io::stdout().flush().expect("Failed to flush stdout.");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    let input = input.trim().to_lowercase();

    if input == "y" || input.is_empty() {
        match git_commit::commit(&message) {
            Ok(()) => {
                println!("Commit successful");
            }
            Err(e) => {
                panic!("Error: {}", e);
            }
        }
    } else {
        panic!("Commit canceled.");
    }
}
