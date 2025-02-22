mod ai;
mod config;

use ai::ask_ai;
use config::load_api_key;
use git2::{DiffFormat, Repository};
use tokio;

#[tokio::main]
async fn main() {
    let mut prompt_diff = String::new();

    let repo = Repository::open(".").expect("Failed to open repositoyry");

    let head_commit = repo.head().unwrap().peel_to_commit();
    let head_tree = head_commit.unwrap().tree().unwrap();

    let index = repo.index().unwrap();

    let diff = repo
        .diff_tree_to_index(Some(&head_tree), Some(&index), None)
        .expect("Failed to get diff");

    diff.print(DiffFormat::Patch, |_, _, line| {
        let diff_line = format!(
            "{} {}",
            line.origin(),
            std::str::from_utf8(line.content()).unwrap()
        );
        prompt_diff.push_str(&diff_line);
        true
    })
    .expect("Failed to print diff");

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
