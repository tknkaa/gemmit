mod ai;
mod config;
mod git_commit;
mod git_diff;

use ai::ask_ai;
use config::load_api_key;
use git_diff::get_git_diff;
use git2::opts::get_mwindow_file_limit;
use reqwest::get;
use std::io::{self, Write};
use tokio;

#[tokio::main]
async fn main() {
    println!("Gemini suggested the following commit message.");
    let prompt_diff = match get_git_diff() {
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

    let mut message = String::new();
    match ask_ai(&api_key, &prompt).await {
        Ok(response) => {
            message.push_str(&response);
            println!("{response}");
        }
        Err(_) => println!("Failed to ask AI"),
    }

    print!("Do you want to commit with this message? [Y/n] ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    let input = input.trim().to_lowercase();

    if input == "y" || input.is_empty() {
        git_commit::commit(&message);
    } else {
        println!("Commit canceled.");
    }
}
