mod ai;
mod config;
mod git_diff;

use ai::ask_ai;
use config::load_api_key;
use git_diff::get_git_diff;
use tokio;

#[tokio::main]
async fn main() {
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

    match ask_ai(&api_key, &prompt).await {
        Ok(response) => println!("{response}"),
        Err(_) => println!("Failed to ask AI"),
    }
}
