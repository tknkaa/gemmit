use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
pub enum Format {
    Casual,
    Formal,
}

#[derive(Parser, Debug)]
#[command(version, about = env!("CARGO_PKG_DESCRIPTION"), long_about = None)]
pub struct Args {
    #[arg(short, long, help = "a word for your commit message to start with")]
    pub start: Option<String>,
    #[arg(short, long, help = "words for your commit message to include", num_args = 0..)]
    pub include: Option<Vec<String>>,
    #[arg(
        short,
        long,
        help = "language of your commit message",
        default_value = "English"
    )]
    pub lang: Option<String>,
    #[arg(
        short,
        long,
        help = "format of your commit message: casula or formal",
        default_value = "format"
    )]
    pub format: Option<Format>,
}
