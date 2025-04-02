use std::env;
use std::process;

pub fn load_api_key() -> String {
    match env::var("GEMINI_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            eprintln!("GEMINI_API_KEY not found");
            process::exit(1);
        }
    }
}
