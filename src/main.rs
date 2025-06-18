mod cli;
mod config;
mod git;
mod llm;
mod prompt;

use clap::Parser;
use cli::Args;
use std::{io, process};

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let api_key = config::load_api_key();
    let diff = git::get_git_diff_output();
    let filepath = "assets/prompt.txt";
    let prompt = prompt::create_prompt(&diff, filepath, &args);
    println!("loading...");
    let raw_message = llm::get_commit_message(&api_key, &prompt).await;
    let commit_message = raw_message.trim();
    println!("Gemini suggested the following message.");
    println!("{commit_message}");
    println!("Do you want to commit with this message? [Y/n]");
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();
    if config::confirm_commit(&buffer) {
        git::run_git_commit(&commit_message);
    } else {
        println!("commit canceled.");
        process::exit(1);
    }
}
