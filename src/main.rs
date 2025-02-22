use dotenvy::dotenv;
use reqwest::{self, Client, Error};
use std::env;
use tokio;

#[tokio::main]
async fn main() {
    dotenv().expect(".env not found");
    let api_key = match env::var("GEMINI_API_KEY") {
        Ok(key) => key,
        Err(_) => panic!("api key not found"),
    };

    //println!("{api_key}");

    let prompt = String::from("Explain how AI works");

    match ask_ai(&api_key, &prompt).await {
        Ok(res) => {
            println!("{}", res);
        }
        Err(_) => {
            println!("failed to ask ai");
        }
    }
}

async fn ask_ai(api_key: &str, prompt: &str) -> Result<String, Error> {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key={}",
        api_key
    );

    let client = Client::new();
    let body = "{
        \"contents\": [{
          \"parts\":[{\"text\": \"Explain how AI works\"}]
          }]
         }";

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await?
        .text()
        .await?;

    println!("{}", response);
    Ok(response)
}
