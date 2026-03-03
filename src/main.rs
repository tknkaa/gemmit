mod app;
mod gemini;
mod git;
mod ui;

use std::io::{self, stdout};
use std::thread;
use std::time::Duration;

use clap::Parser;
use crossterm::{
    event::{self, Event},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, TerminalOptions, Viewport, backend::CrosstermBackend};

use app::App;
use ui::render;

#[derive(Parser)]
#[command(
    name = "gemmit",
    about = "Generate conventional commit messages using Gemini AI",
    version
)]
struct Cli {}

fn main() -> io::Result<()> {
    let _cli = Cli::parse();

    enable_raw_mode()?;
    let mut terminal = Terminal::with_options(
        CrosstermBackend::new(stdout()),
        TerminalOptions {
            viewport: Viewport::Inline(8),
        },
    )?;

    let mut app = App::new();
    app.spawn_prepare();

    loop {
        render(&mut terminal, &app)?;

        let done = app.tick();

        if done {
            render(&mut terminal, &app)?;
            thread::sleep(Duration::from_millis(600));
            break;
        }

        if event::poll(Duration::from_millis(80))? {
            if let Event::Key(key) = event::read()? {
                if app.handle_key(key.code, key.modifiers) {
                    render(&mut terminal, &app)?;
                    thread::sleep(Duration::from_millis(600));
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    println!();

    Ok(())
}
