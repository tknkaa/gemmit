mod ai;
mod config;
mod git_commit;
mod git_diff;

use ai::ask_ai;
use config::load_api_key;
use git_diff::get_git_diff;
use std::io::{self, Write};
use std::process;
use tokio;

#[tokio::main]
async fn main() {
    let prompt_diff = get_git_diff();
    let api_key = load_api_key();
    let prompt = format!(
        "Given the following changes, provide a concise and meaningful commit message: {}. Please only return the commit message.",
        prompt_diff
    );

    let response = ask_ai(&api_key, &prompt).await;

    println!("Gemini suggested the following commit message");
    println!("> {response}");
    print!("Do you want to commit with this message? [Y/n] ");
    io::stdout().flush().expect("Failed to flush stdout.");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    let input = input.trim().to_lowercase();

    if input == "y" || input.is_empty() {
        match git_commit::commit(&response) {
            Ok(()) => {
                println!("Commit successful");
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
    } else {
        eprintln!("Commit canceled.");
        process::exit(1);
    }
}
