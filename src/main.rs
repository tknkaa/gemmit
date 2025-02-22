use dotenvy::dotenv;
use reqwest::{self, Client, Error};
use serde_json::json;
use std::env;
use tokio;

#[tokio::main]
async fn main() {
    dotenv().expect(".env not found");
    let api_key = match env::var("GEMINI_API_KEY") {
        Ok(key) => key,
        Err(_) => panic!("api key not found"),
    };

    let prompt = String::from("Explain how AI works in one sentence.");

    match ask_ai(&api_key, &prompt).await {
        Ok(res) => {
            let v: serde_json::Value = serde_json::from_str(&res).unwrap();

            let ai_answer = v["candidates"]
                .as_array()
                .and_then(|candidates| candidates.get(0)) //
                .and_then(|candidate| candidate["content"]["parts"].as_array())
                .and_then(|parts| parts.get(0))
                .and_then(|part| part["text"].as_str())
                .unwrap_or("No response from AI");
            println!("{ai_answer}")
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
    let body = json!({
        "contents": [{
            "parts": [{"text": prompt}]
        }]
    });

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send()
        .await?
        .text()
        .await?;

    Ok(response)
}
