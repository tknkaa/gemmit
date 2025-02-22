use dotenvy::dotenv;
use std::env;

pub fn load_api_key() -> String {
    dotenv().expect(".env not found");
    env::var("GEMINI_API_KEY").expect("API key not found")
}
