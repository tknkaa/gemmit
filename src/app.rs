use std::sync::mpsc;
use std::thread;

use crossterm::event::{KeyCode, KeyModifiers};

use crate::gemini::call_gemini;
use crate::git::{git_commit, git_prepare, BgMsg};

pub const SPINNER_FRAMES: &[&str] = &["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];

pub enum State {
    Loading,
    Warning(Vec<String>),
    Confirm(String),
    Committing,
    Done,
    Cancelled,
    Error(String),
}

pub struct App {
    pub state: State,
    pub spinner_frame: usize,
    tx: mpsc::SyncSender<BgMsg>,
    rx: mpsc::Receiver<BgMsg>,
    prompt: String,
    commit_message: String,
}

impl App {
    pub fn new() -> Self {
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

    pub fn spawn_prepare(&self) {
        let tx = self.tx.clone();
        thread::spawn(move || {
            tx.send(git_prepare()).ok();
        });
    }

    pub fn spawn_generate(&self) {
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

    pub fn spawn_commit(&self) {
        let message = self.commit_message.clone();
        let tx = self.tx.clone();
        thread::spawn(move || {
            tx.send(git_commit(&message)).ok();
        });
    }

    /// Advance spinner and drain background messages. Returns true if a
    /// terminal state was just reached.
    pub fn tick(&mut self) -> bool {
        self.spinner_frame = (self.spinner_frame + 1) % SPINNER_FRAMES.len();
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                BgMsg::Prepared {
                    prompt,
                    warning_dirs,
                } => {
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
    pub fn handle_key(&mut self, code: KeyCode, mods: KeyModifiers) -> bool {
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

    pub fn is_done(&self) -> bool {
        matches!(self.state, State::Done | State::Cancelled | State::Error(_))
    }
}
