mod ai;
mod config;
mod git_commit;
mod git_diff;
mod handle_input;

use ai::ask_ai;
use config::load_api_key;
use git_diff::get_git_diff;
use handle_input::handle_input;
use std::io::{self, Write};
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

    let input = handle_input();

    git_commit::commit(&response, &input);
}
