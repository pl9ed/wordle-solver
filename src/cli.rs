use clap::{Parser};

/// Wordle Solver CLI options
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to a newline-delimited wordbank file
    #[arg(short = 'i', long = "input")]
    pub wordbank_path: Option<String>,
}

pub fn parse_cli() -> Cli {
    Cli::parse()
}

