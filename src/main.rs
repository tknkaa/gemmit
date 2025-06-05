use gemmit::{Args, config, git, llm, prompt};
use std::{io, process};

fn main() {
    let args = Args::parse();
    let diff = git::get_git_diff_output();
    let filepath = "../assets/prompt.txt";
    let prompt = prompt::create_prompt(&diff, filepath);
    let commit_message = llm::get_commit_message(&api_key, &prompt);
    print!(
        "Gemini suggested the following message.
        \n {commit_message}
        \n Do you want to commit with this message? [Y/n]"
    );
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();
    if config::confirm_commit(&buffer) {
        git::run_git_commit(&commit_message);
    } else {
        println!("commit canceled.");
        process::exit(1);
    }
}
