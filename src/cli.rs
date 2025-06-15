use clap::Parser;

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
}
