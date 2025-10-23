use clap::Parser;

/// Wordle Solver CLI options
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to a newline-delimited wordbank file
    #[arg(short = 'i', long = "input")]
    pub wordbank_path: Option<String>,
}

#[must_use]
pub fn parse_cli() -> Cli {
    Cli::parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cli_no_args() {
        // Test parsing with no custom wordbank
        let cli = Cli {
            wordbank_path: None,
        };
        assert_eq!(cli.wordbank_path, None);
    }

    #[test]
    fn test_parse_cli_with_path() {
        // Test parsing with a wordbank path
        let cli = Cli {
            wordbank_path: Some("custom_wordbank.txt".to_string()),
        };
        assert_eq!(cli.wordbank_path, Some("custom_wordbank.txt".to_string()));
    }

    #[test]
    fn test_cli_structure() {
        // Verify CLI structure can be created and accessed
        let cli = Cli {
            wordbank_path: Some("/path/to/words.txt".to_string()),
        };

        match cli.wordbank_path {
            Some(path) => assert_eq!(path, "/path/to/words.txt"),
            None => panic!("Expected Some path"),
        }
    }
}
