use std::io;

use ratatui::{
    Terminal, backend::CrosstermBackend,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::{App, SPINNER_FRAMES, State};

pub fn render(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &App) -> io::Result<()> {
    let spinner = SPINNER_FRAMES[app.spinner_frame];
    terminal.draw(|frame| {
        let area = frame.area();
        let bold = |style: Style| style.add_modifier(Modifier::BOLD);
        let lines: Vec<Line> = match &app.state {
            State::Loading => vec![Line::from(vec![
                Span::styled(spinner, Style::default().fg(Color::Magenta)),
                Span::raw(" "),
                Span::styled("Thinking...", bold(Style::default().fg(Color::Cyan))),
            ])],

            State::Warning(dirs) => {
                let mut lines = vec![
                    Line::from(Span::styled(
                        "⚠  WARNING",
                        bold(Style::default().fg(Color::Yellow)),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "The following directories are staged and should typically not be committed:",
                        bold(Style::default().fg(Color::Yellow)),
                    )),
                ];
                for d in dirs {
                    lines.push(Line::from(Span::styled(
                        format!("  • {d}"),
                        bold(Style::default().fg(Color::Red)),
                    )));
                }
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "Continue anyway? (y/N)",
                    bold(Style::default().fg(Color::Cyan)),
                )));
                lines
            }

            State::Confirm(msg) => vec![
                Line::from(Span::styled(
                    "✨ Gemini suggested:",
                    bold(Style::default().fg(Color::Blue)),
                )),
                Line::from(""),
                Line::from(Span::styled(msg.as_str(), Style::default().fg(Color::Yellow))),
                Line::from(""),
                Line::from(Span::styled(
                    "Commit with this message? (y/N)",
                    bold(Style::default().fg(Color::Cyan)),
                )),
            ],

            State::Committing => vec![Line::from(vec![
                Span::styled(spinner, Style::default().fg(Color::Magenta)),
                Span::raw(" "),
                Span::styled("Committing...", bold(Style::default().fg(Color::Cyan))),
            ])],

            State::Done => vec![Line::from(Span::styled(
                "✅ Committed successfully!",
                bold(Style::default().fg(Color::Green)),
            ))],

            State::Cancelled => vec![Line::from(Span::styled(
                "🚫 Commit canceled",
                bold(Style::default().fg(Color::Yellow)),
            ))],

            State::Error(e) => vec![Line::from(Span::styled(
                format!("❌ Error: {e}"),
                bold(Style::default().fg(Color::Red)),
            ))],
        };
        frame.render_widget(Paragraph::new(lines), area);
    })?;
    Ok(())
}
