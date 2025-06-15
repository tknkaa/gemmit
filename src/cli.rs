use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about = env!("CARGO_PKG_DESCRIPTION"), long_about = None)]
pub struct Args {
    #[arg(
        short,
        long,
        help = "Specify a word for your commit message to start with"
    )]
    pub start: Option<String>,
    #[arg(short, long, help = "Specify words for your commit message to include")]
    pub include: Option<Vec<String>>,
}
