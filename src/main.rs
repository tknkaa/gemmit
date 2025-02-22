mod ai;
mod config;

use ai::ask_ai;
use config::load_api_key;
use tokio;

#[tokio::main]
async fn main() {
    let api_key = load_api_key();
    let prompt = "Explain how AI works in one sentence.";

    match ask_ai(&api_key, prompt).await {
        Ok(response) => println!("{response}"),
        Err(_) => println!("Failed to ask AI"),
    }
}
