use dotenvy::dotenv;
use std::env;

fn main() {
    dotenv().expect(".env not found");
    let api_key = match env::var("GEMINI_API_KEY") {
        Ok(key) => key,
        Err(_) => panic!("api key not found"),
    };

    println!("{api_key}");
}
