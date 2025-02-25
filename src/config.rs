use std::env;

pub fn load_api_key() -> String {
    env::var("GEMINI_API_KEY").expect("API key not found")
}
