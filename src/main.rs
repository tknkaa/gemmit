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
    let prompt_diff = match get_git_diff() {
        Ok(diff) => {
            if diff.trim().is_empty() {
                eprintln!("Git diff is empty. No changes detected.");
                process::exit(1);
            } else {
                diff
            }
        }
        Err(_) => {
            eprintln!("Failed to get git diff");
            process::exit(1);
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
        Err(_) => {
            eprintln!("Failed to ask Gemini");
            process::exit(1);
        }
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
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
    } else {
        eprintln!("Commit canceled.");
        process::exit(1);
    }
}
