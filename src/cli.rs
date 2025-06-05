use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about = env!("CARGO_PKG_DESCRIPTION"), long_about = None)]
pub struct Args {
    #[arg(short, long, default_value_t = None)]
    start: Option<String>,
}
