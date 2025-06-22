use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
pub enum Format {
    Casual,
    Formal,
}

#[derive(Parser, Debug)]
#[command(version, about = env!("CARGO_PKG_DESCRIPTION"), long_about = None)]
pub struct Args {
    #[arg(
        short,
        long,
        help = "Specify a word for your commit message to start with"
    )]
    pub start: Option<String>,
    #[arg(short, long, help = "Provide words for your commit message to include", num_args = 0..)]
    pub include: Option<Vec<String>>,
    #[arg(
        short,
        long,
        help = "Set the language of your commit message",
        default_value = "English"
    )]
    pub lang: Option<String>,
    #[arg(
        short,
        long,
        help = "Choose the format of your commit message: casula or formal",
        default_value = "formal"
    )]
    pub format: Option<Format>,
}
