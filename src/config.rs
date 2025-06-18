use std::{env, process};

pub fn load_api_key() -> String {
    match env::var("GEMINI_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            eprintln!("GEMINI_API_KEY not found");
            process::exit(1);
        }
    }
}

pub fn confirm_commit(buffer: &str) -> bool {
    let input = buffer.trim();
    if input.to_lowercase().as_str() == "y" || input.is_empty() {
        true
    } else {
        false
    }
}
