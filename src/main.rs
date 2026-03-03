use std::io::{self, Write};
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(name = "gemmit", about = "Generate conventional commit messages using Gemini AI", version)]
struct Cli {}

const WARNING_DIRS: &[&str] = &["node_modules/", ".direnv/"];
const LOCK_EXCLUDES: &[&str] = &[
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
];
const SPINNER_FRAMES: &[&str] = &["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];

// ── Gemini API ────────────────────────────────────────────────────────────────

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

fn call_gemini(prompt: &str) -> Result<String, String> {
    let api_key = std::env::var("GEMINI_API_KEY")
        .map_err(|_| "GEMINI_API_KEY environment variable not set".to_string())?;

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
        api_key
    );

    let body = GeminiRequest {
        contents: vec![GeminiContent {
            parts: vec![GeminiPart { text: prompt.to_string() }],
        }],
    };

    let resp = Client::new()
        .post(&url)
        .json(&body)
        .send()
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().unwrap_or_default();
        return Err(format!("Gemini API error {status}: {text}"));
    }

    let data: GeminiResponse =
        resp.json().map_err(|e| format!("Failed to parse response: {e}"))?;

    data.candidates
        .into_iter()
        .next()
        .and_then(|c| c.content.parts.into_iter().next())
        .map(|p| p.text.trim().to_string())
        .ok_or_else(|| "Empty response from Gemini".to_string())
}

// ── Background work ───────────────────────────────────────────────────────────

enum BgMsg {
    /// Git info gathered; prompt is ready, caller decides what to do next.
    Prepared { prompt: String, warning_dirs: Vec<String> },
    /// Gemini returned a commit message.
    Generated(String),
    GenerateErr(String),
    CommitDone,
    CommitErr(String),
}

fn git_prepare() -> BgMsg {
    let staged = match Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .output()
    {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        Err(e) => return BgMsg::GenerateErr(format!("Failed to run git: {e}")),
    };

    if staged.trim().is_empty() {
        return BgMsg::GenerateErr("no changes are staged".to_string());
    }

    let mut diff_args = vec!["diff", "--cached", "--"];
    diff_args.extend_from_slice(LOCK_EXCLUDES);

    let diff = match Command::new("git").args(&diff_args).output() {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        Err(e) => return BgMsg::GenerateErr(format!("Failed to get git diff: {e}")),
    };

    let prompt = format!(
        "Generate a concise conventional commit message (feat/fix/chore prefix) for this diff. \
         Keep it short and to the point - ideally one line. \
         Return only the commit message, no explanations or formatting:\n\n{diff}"
    );

    let warning_dirs: Vec<String> = WARNING_DIRS
        .iter()
        .filter(|&&d| staged.contains(d))
        .map(|&s| s.to_string())
        .collect();

    BgMsg::Prepared { prompt, warning_dirs }
}

// ── App ───────────────────────────────────────────────────────────────────────

enum State {
    Loading,
    Warning(Vec<String>),
    Confirm(String),
    Committing,
    Done,
    Cancelled,
    Error(String),
}

struct App {
    state: State,
    spinner_frame: usize,
    tx: mpsc::SyncSender<BgMsg>,
    rx: mpsc::Receiver<BgMsg>,
    prompt: String,
    commit_message: String,
}

impl App {
    fn new() -> Self {
        let (tx, rx) = mpsc::sync_channel(8);
        Self {
            state: State::Loading,
            spinner_frame: 0,
            tx,
            rx,
            prompt: String::new(),
            commit_message: String::new(),
        }
    }

    fn spawn_prepare(&self) {
        let tx = self.tx.clone();
        thread::spawn(move || { tx.send(git_prepare()).ok(); });
    }

    fn spawn_generate(&self) {
        let prompt = self.prompt.clone();
        let tx = self.tx.clone();
        thread::spawn(move || {
            let msg = match call_gemini(&prompt) {
                Ok(m) => BgMsg::Generated(m),
                Err(e) => BgMsg::GenerateErr(e),
            };
            tx.send(msg).ok();
        });
    }

    fn spawn_commit(&self) {
        let message = self.commit_message.clone();
        let tx = self.tx.clone();
        thread::spawn(move || {
            let result = Command::new("git").args(["commit", "-m", &message]).output();
            let msg = match result {
                Ok(o) if o.status.success() => BgMsg::CommitDone,
                Ok(o) => BgMsg::CommitErr(String::from_utf8_lossy(&o.stderr).to_string()),
                Err(e) => BgMsg::CommitErr(e.to_string()),
            };
            tx.send(msg).ok();
        });
    }

    /// Advance spinner and drain background messages. Returns true if a
    /// terminal state was just reached.
    fn tick(&mut self) -> bool {
        self.spinner_frame = (self.spinner_frame + 1) % SPINNER_FRAMES.len();
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                BgMsg::Prepared { prompt, warning_dirs } => {
                    self.prompt = prompt;
                    if warning_dirs.is_empty() {
                        self.spawn_generate();
                    } else {
                        self.state = State::Warning(warning_dirs);
                    }
                }
                BgMsg::Generated(message) => {
                    self.commit_message = message.clone();
                    self.state = State::Confirm(message);
                }
                BgMsg::GenerateErr(e) => self.state = State::Error(e),
                BgMsg::CommitDone => self.state = State::Done,
                BgMsg::CommitErr(e) => self.state = State::Error(e),
            }
        }
        self.is_done()
    }

    /// Handle a keypress. Returns true when the TUI should exit.
    fn handle_key(&mut self, code: KeyCode, mods: KeyModifiers) -> bool {
        let ctrl_c = code == KeyCode::Char('c') && mods.contains(KeyModifiers::CONTROL);
        let yes = matches!(code, KeyCode::Char('y') | KeyCode::Char('Y'));

        if ctrl_c {
            self.state = State::Cancelled;
            return true;
        }

        match &self.state {
            State::Warning(_) if yes => {
                self.state = State::Loading;
                self.spawn_generate();
            }
            State::Warning(_) => {
                self.state = State::Cancelled;
                return true;
            }
            State::Confirm(_) if yes => {
                self.state = State::Committing;
                self.spawn_commit();
            }
            State::Confirm(_) => {
                self.state = State::Cancelled;
                return true;
            }
            _ if self.is_done() => return true,
            _ => {}
        }
        false
    }

    fn is_done(&self) -> bool {
        matches!(self.state, State::Done | State::Cancelled | State::Error(_))
    }
}

// ── Rendering ─────────────────────────────────────────────────────────────────

fn render(app: &App) {
    let spinner = SPINNER_FRAMES[app.spinner_frame];

    match &app.state {
        State::Loading => {
            println!();
            println!("\x1b[35m{}\x1b[0m \x1b[1;36mThinking...\x1b[0m", spinner);
        }

        State::Warning(dirs) => {
            println!();
            println!("\x1b[1;33m⚠️  WARNING\x1b[0m");
            println!();
            println!("\x1b[1;33mThe following directories are staged and should typically not be committed:\x1b[0m");
            for d in dirs {
                println!("\x1b[1;31m  • {}\x1b[0m", d);
            }
            println!();
            println!("\x1b[1;36mContinue anyway? (y/N): \x1b[0m");
        }

        State::Confirm(msg) => {
            println!();
            println!("\x1b[1;34m✨ Gemini suggested:\x1b[0m");
            println!();
            println!("\x1b[33m{}\x1b[0m", msg);
            println!();
            println!("\x1b[1;36mCommit with this message? (y/N): \x1b[0m");
        }

        State::Committing => {
            println!();
            println!("\x1b[35m{}\x1b[0m \x1b[1;36mCommitting...\x1b[0m", spinner);
        }

        State::Done => {
            println!();
            println!("\x1b[1;32m✅ Committed successfully!\x1b[0m");
            println!();
        }

        State::Cancelled => {
            println!();
            println!("\x1b[1;33m🚫 Commit canceled\x1b[0m");
            println!();
        }

        State::Error(e) => {
            println!();
            println!("\x1b[1;31m❌ Error: {}\x1b[0m", e);
            println!();
        }
    }
    io::stdout().flush().ok();
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() -> io::Result<()> {
    let _cli = Cli::parse();

    enable_raw_mode()?;
    let mut app = App::new();
    app.spawn_prepare();

    loop {
        render(&app);

        let done = app.tick();

        if done {
            render(&app);
            thread::sleep(Duration::from_millis(600));
            break;
        }

        if event::poll(Duration::from_millis(80))? {
            if let Event::Key(key) = event::read()? {
                if app.handle_key(key.code, key.modifiers) {
                    render(&app);
                    thread::sleep(Duration::from_millis(600));
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;

    Ok(())
}
