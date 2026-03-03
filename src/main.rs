use std::io::{self, Write};
use std::process::{Command, exit};
use std::time::Duration;

use colored::Colorize;
use crossterm::{
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

const WARNING_DIRS: &[&str] = &["node_modules/", ".direnv/"];
const LOCK_FILES: &[&str] = &[
    "go.sum",
    "go.mod",
    "package-lock.json",
    "yarn.lock",
    "pnpm-lock.yaml",
    "bun.lock",
    "Cargo.lock",
    "poetry.lock",
    "uv.lock",
    "Gemfile.lock",
    "flake.lock",
];

// --- Gemini API types ---

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
}

#[derive(Serialize, Deserialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Serialize, Deserialize)]
struct GeminiPart {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
}

// --- Gemini call ---

fn call_gemini(prompt: &str) -> Result<String, String> {
    let api_key = std::env::var("GEMINI_API_KEY")
        .map_err(|_| "GEMINI_API_KEY environment variable not set".to_string())?;

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
        api_key
    );

    let request = GeminiRequest {
        contents: vec![GeminiContent {
            parts: vec![GeminiPart {
                text: prompt.to_string(),
            }],
        }],
    };

    let response = Client::new()
        .post(&url)
        .json(&request)
        .send()
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        return Err(format!("Gemini API error {}: {}", status, body));
    }

    let resp: GeminiResponse = response
        .json()
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    resp.candidates
        .into_iter()
        .next()
        .and_then(|c| c.content.parts.into_iter().next())
        .map(|p| p.text.trim().to_string())
        .ok_or_else(|| "Empty response from Gemini".to_string())
}

// --- Git helpers ---

fn git_staged_files() -> Result<String, String> {
    let out = Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .output()
        .map_err(|e| format!("Failed to run git: {}", e))?;
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

fn git_diff_no_locks() -> Result<String, String> {
    let out = Command::new("git")
        .args([
            "diff",
            "--cached",
            "--",
            ":(exclude)go.sum",
            ":(exclude)go.mod",
            ":(exclude)package-lock.json",
            ":(exclude)yarn.lock",
            ":(exclude)pnpm-lock.yaml",
            ":(exclude)bun.lock",
            ":(exclude)Cargo.lock",
            ":(exclude)poetry.lock",
            ":(exclude)uv.lock",
            ":(exclude)Gemfile.lock",
            ":(exclude)flake.lock",
        ])
        .output()
        .map_err(|e| format!("Failed to get git diff: {}", e))?;
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

fn changed_lock_files() -> Vec<String> {
    LOCK_FILES
        .iter()
        .filter(|&&lf| {
            Command::new("git")
                .args(["diff", "--cached", "--name-only", "--", lf])
                .output()
                .map(|o| !o.stdout.is_empty())
                .unwrap_or(false)
        })
        .map(|&s| s.to_string())
        .collect()
}

fn build_prompt(diff: &str, lock_files: &[String]) -> String {
    let mut prompt = "Generate a concise conventional commit message (feat/fix/chore prefix) for this diff. \
        Keep it short and to the point - ideally one line. Return only the commit message, no explanations or formatting:\n"
        .to_string();
    if !lock_files.is_empty() {
        prompt += &format!(
            "\nNote: The following lock/dependency files were also changed (diff not shown): {}\n",
            lock_files.join(", ")
        );
    }
    prompt += &format!("\n{}", diff);
    prompt
}

// --- UI helpers ---

fn spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"])
            .template("{spinner:.magenta} {msg}")
            .unwrap(),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

/// Read a single y/N keypress without requiring Enter.
fn read_yn(prompt_text: &str) -> bool {
    print!("{}", prompt_text);
    io::stdout().flush().ok();
    enable_raw_mode().ok();
    let result = loop {
        match read() {
            Ok(Event::Key(KeyEvent { code, modifiers, .. })) => match code {
                KeyCode::Char('y') | KeyCode::Char('Y') => break true,
                KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => break false,
                _ => break false,
            },
            _ => break false,
        }
    };
    disable_raw_mode().ok();
    println!();
    result
}

fn die(msg: &str) -> ! {
    eprintln!("{}", format!("❌ Error: {}", msg).red().bold());
    exit(1);
}

// --- Main ---

fn main() {
    let pb = spinner("Thinking...");

    let staged = match git_staged_files() {
        Ok(s) if s.trim().is_empty() => {
            pb.finish_and_clear();
            die("No changes are staged");
        }
        Ok(s) => s,
        Err(e) => {
            pb.finish_and_clear();
            die(&e);
        }
    };

    let warning_dirs: Vec<&str> = WARNING_DIRS
        .iter()
        .filter(|&&dir| staged.contains(dir))
        .copied()
        .collect();

    let diff = match git_diff_no_locks() {
        Ok(d) => d,
        Err(e) => {
            pb.finish_and_clear();
            die(&e);
        }
    };

    let locks = changed_lock_files();
    let prompt = build_prompt(&diff, &locks);

    // Warn about staged dirs that should not be committed
    if !warning_dirs.is_empty() {
        pb.finish_and_clear();
        println!();
        println!("{}", "⚠️  WARNING".yellow().bold());
        println!();
        println!("{}", "The following directories are in your staged changes:".yellow().bold());
        for dir in &warning_dirs {
            println!("{}", format!("  • {}", dir).red().bold());
        }
        println!();
        println!("{}", "These directories should typically not be committed!".yellow().bold());
        println!();
        if !read_yn(&format!("{}", "Continue anyway? (y/N): ".cyan().bold())) {
            println!("{}", "🚫 commit canceled".yellow().bold());
            exit(0);
        }
        let pb2 = spinner("Thinking...");
        let msg = match call_gemini(&prompt) {
            Ok(m) => {
                pb2.finish_and_clear();
                m
            }
            Err(e) => {
                pb2.finish_and_clear();
                die(&e);
            }
        };
        confirm_and_commit(msg);
    } else {
        let msg = match call_gemini(&prompt) {
            Ok(m) => {
                pb.finish_and_clear();
                m
            }
            Err(e) => {
                pb.finish_and_clear();
                die(&e);
            }
        };
        confirm_and_commit(msg);
    }
}

fn confirm_and_commit(commit_message: String) {
    println!();
    println!("{}", "✨ Gemini suggested:".bold().blue());
    println!();
    println!("{}", commit_message.yellow());
    println!();

    if !read_yn(&format!("{}", "Commit with this message? (y/N): ".cyan().bold())) {
        println!("{}", "🚫 commit canceled".yellow().bold());
        exit(0);
    }

    let pb = spinner("Committing...");
    match Command::new("git")
        .args(["commit", "-m", &commit_message])
        .output()
    {
        Ok(o) if o.status.success() => {
            pb.finish_and_clear();
            println!("{}", "✅ Committed successfully!".green().bold());
        }
        Ok(o) => {
            pb.finish_and_clear();
            die(&String::from_utf8_lossy(&o.stderr));
        }
        Err(e) => {
            pb.finish_and_clear();
            die(&e.to_string());
        }
    }
}
