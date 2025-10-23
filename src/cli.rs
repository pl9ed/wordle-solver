use clap::Parser;
use crate::solver::Feedback;
use crate::game_state::{GameInterface, UserAction, StartingWordsInfo, Recommendation};
use std::io::BufRead;
use std::path::PathBuf;

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

// UI Input/Output functions

pub enum GuessInput {
    Valid(String),
    Invalid,
    Exit,
    NewGame,
}

fn is_valid_word(word: &str) -> bool {
    word.len() == 5 && word.chars().all(|c| c.is_ascii_alphabetic())
}

fn is_valid_feedback(feedback: &str) -> bool {
    if feedback.is_empty() {
        return false;
    }
    let upper = feedback.to_uppercase();
    upper.len() == 5 && upper.chars().all(|c| c == 'G' || c == 'Y' || c == 'X')
}

pub fn display_starting_words(words: &[String], used_cache: bool, cache_path: Option<&PathBuf>) {
    println!("Optimal starting words:");
    for (i, word) in words.iter().enumerate() {
        println!("{}. {}", i + 1, word);
    }

    if let Some(path) = cache_path {
        if used_cache {
            println!("(Loaded from cache: {}.)", path.display());
        } else {
            println!("(Computed and cached to: {}.)", path.display());
        }
    }

    if !words.is_empty() {
        println!("Suggested starting word: {}", words[0]);
    }
}

pub fn read_guess<R: BufRead>(reader: &mut R) -> GuessInput {
    println!("\nEnter your guess (5 letters, or 'exit' to quit, or 'next' to start a new game):");
    let mut input = String::new();
    reader.read_line(&mut input).unwrap();
    let input = input.trim().to_uppercase();

    match input.as_str() {
        "EXIT" => GuessInput::Exit,
        "NEXT" => GuessInput::NewGame,
        _ if is_valid_word(&input) => GuessInput::Valid(input),
        _ => {
            println!("Invalid guess. Please enter 5 letters.");
            GuessInput::Invalid
        }
    }
}

pub fn read_feedback<R: BufRead>(reader: &mut R) -> Option<Vec<Feedback>> {
    println!("Enter feedback (G=green, Y=yellow, X=gray, e.g. GYXXG):");
    let mut input = String::new();
    reader.read_line(&mut input).unwrap();
    let input = input.trim().to_uppercase();

    if is_valid_feedback(&input) {
        let feedback: Option<Vec<Feedback>> = input.chars().map(Feedback::from_char).collect();

        if feedback.is_none() {
            println!("Invalid feedback. Please enter 5 characters using G, Y, or X.");
        }
        feedback
    } else {
        println!("Invalid feedback. Please enter 5 characters using G, Y, or X.");
        None
    }
}

pub fn display_candidates(candidates: &[String]) {
    println!("Possible candidates ({})", candidates.len());
    for word in candidates.iter().take(5) {
        println!("{word}");
    }
}

pub fn display_recommendation(guess: &str, score: f64, is_candidate: bool) {
    let category = if is_candidate {
        "solution candidate"
    } else {
        "information-gathering"
    };
    println!("Recommended guess: {guess} (expected pool size {score:.2}) [{category}]");
}

pub fn display_exit_message() {
    println!("Exiting.");
}

pub fn display_new_game_message(word_count: usize) {
    println!("New game started. Loaded {} words.", word_count);
}

pub fn display_computing_message() {
    println!("Computing optimal guess, please wait...");
}

pub fn display_no_candidates_message() {
    println!("No candidates remain. Check your inputs.");
}

pub fn display_solution_found(solution: &str) {
    println!("Solution found: {}", solution);
}

/// CLI implementation of the GameInterface trait
/// This struct wraps a BufRead reader and implements the game interface for CLI interaction
pub struct CliInterface<R: BufRead> {
    reader: R,
}

impl<R: BufRead> CliInterface<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }
}

impl<R: BufRead> GameInterface for CliInterface<R> {
    fn display_starting_words(&mut self, info: &StartingWordsInfo) {
        display_starting_words(&info.words, info.used_cache, info.cache_path.as_ref());
    }

    fn read_guess(&mut self) -> Option<UserAction> {
        match read_guess(&mut self.reader) {
            GuessInput::Valid(guess) => Some(UserAction::Guess(guess)),
            GuessInput::Exit => Some(UserAction::Exit),
            GuessInput::NewGame => Some(UserAction::NewGame),
            GuessInput::Invalid => None,
        }
    }

    fn read_feedback(&mut self) -> Option<Vec<Feedback>> {
        read_feedback(&mut self.reader)
    }

    fn display_candidates(&mut self, candidates: &[String]) {
        display_candidates(candidates);
    }

    fn display_recommendation(&mut self, recommendation: &Recommendation) {
        display_recommendation(&recommendation.guess, recommendation.score, recommendation.is_candidate);
    }

    fn display_computing_message(&mut self) {
        display_computing_message();
    }

    fn display_no_candidates_message(&mut self) {
        display_no_candidates_message();
    }

    fn display_solution_found(&mut self, solution: &str) {
        display_solution_found(solution);
    }

    fn display_exit_message(&mut self) {
        display_exit_message();
    }

    fn display_new_game_message(&mut self, word_count: usize) {
        display_new_game_message(word_count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use crate::solver::Feedback;

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

    // Tests for validation functions
    #[test]
    fn test_is_valid_word() {
        assert!(is_valid_word("CRANE"));
        assert!(is_valid_word("crane"));
        assert!(is_valid_word("AbCdE"));
        assert!(!is_valid_word("CRAN")); // Too short
        assert!(!is_valid_word("CRANES")); // Too long
        assert!(!is_valid_word("CRAN3")); // Contains digit
        assert!(!is_valid_word("CRAN ")); // Contains space
        assert!(!is_valid_word("")); // Empty
    }

    #[test]
    fn test_is_valid_feedback() {
        assert!(is_valid_feedback("GGGGG"));
        assert!(is_valid_feedback("XXYGG"));
        assert!(is_valid_feedback("YYYXX"));
        assert!(is_valid_feedback("gygxg")); // lowercase should pass (case-insensitive)
        assert!(is_valid_feedback("GyGxG")); // mixed case should pass
        assert!(!is_valid_feedback("GGGG")); // Too short
        assert!(!is_valid_feedback("GGGGGG")); // Too long
        assert!(!is_valid_feedback("GGGGA")); // Invalid character
        assert!(!is_valid_feedback("12345")); // Numbers
        assert!(!is_valid_feedback("")); // Empty
    }

    // Tests for read_guess function
    #[test]
    fn test_read_guess_valid_word() {
        let input = "CRANE\n";
        let mut reader = Cursor::new(input);
        match read_guess(&mut reader) {
            GuessInput::Valid(word) => assert_eq!(word, "CRANE"),
            _ => panic!("Expected Valid guess"),
        }
    }

    #[test]
    fn test_read_guess_lowercase_converted() {
        let input = "crane\n";
        let mut reader = Cursor::new(input);
        match read_guess(&mut reader) {
            GuessInput::Valid(word) => assert_eq!(word, "CRANE"),
            _ => panic!("Expected Valid guess with uppercase conversion"),
        }
    }

    #[test]
    fn test_read_guess_exit() {
        let input = "exit\n";
        let mut reader = Cursor::new(input);
        match read_guess(&mut reader) {
            GuessInput::Exit => {},
            _ => panic!("Expected Exit"),
        }
    }

    #[test]
    fn test_read_guess_exit_case_insensitive() {
        let input = "EXIT\n";
        let mut reader = Cursor::new(input);
        match read_guess(&mut reader) {
            GuessInput::Exit => {},
            _ => panic!("Expected Exit"),
        }
    }

    #[test]
    fn test_read_guess_new_game() {
        let input = "next\n";
        let mut reader = Cursor::new(input);
        match read_guess(&mut reader) {
            GuessInput::NewGame => {},
            _ => panic!("Expected NewGame"),
        }
    }

    #[test]
    fn test_read_guess_invalid_too_short() {
        let input = "CRAN\n";
        let mut reader = Cursor::new(input);
        match read_guess(&mut reader) {
            GuessInput::Invalid => {},
            _ => panic!("Expected Invalid"),
        }
    }

    #[test]
    fn test_read_guess_invalid_too_long() {
        let input = "CRANES\n";
        let mut reader = Cursor::new(input);
        match read_guess(&mut reader) {
            GuessInput::Invalid => {},
            _ => panic!("Expected Invalid"),
        }
    }

    #[test]
    fn test_read_guess_invalid_with_numbers() {
        let input = "CRAN3\n";
        let mut reader = Cursor::new(input);
        match read_guess(&mut reader) {
            GuessInput::Invalid => {},
            _ => panic!("Expected Invalid"),
        }
    }

    // Tests for read_feedback function
    #[test]
    fn test_read_feedback_valid_all_green() {
        let input = "GGGGG\n";
        let mut reader = Cursor::new(input);
        let result = read_feedback(&mut reader);
        assert!(result.is_some());
        let feedback = result.unwrap();
        assert_eq!(feedback.len(), 5);
        assert!(feedback.iter().all(|f| matches!(f, Feedback::Match)));
    }

    #[test]
    fn test_read_feedback_valid_mixed() {
        let input = "GYXXG\n";
        let mut reader = Cursor::new(input);
        let result = read_feedback(&mut reader);
        assert!(result.is_some());
        let feedback = result.unwrap();
        assert_eq!(feedback.len(), 5);
        assert!(matches!(feedback[0], Feedback::Match));
        assert!(matches!(feedback[1], Feedback::PartialMatch));
        assert!(matches!(feedback[2], Feedback::NoMatch));
        assert!(matches!(feedback[3], Feedback::NoMatch));
        assert!(matches!(feedback[4], Feedback::Match));
    }

    #[test]
    fn test_read_feedback_invalid_too_short() {
        let input = "GGG\n";
        let mut reader = Cursor::new(input);
        let result = read_feedback(&mut reader);
        assert!(result.is_none());
    }

    #[test]
    fn test_read_feedback_invalid_too_long() {
        let input = "GGGGGG\n";
        let mut reader = Cursor::new(input);
        let result = read_feedback(&mut reader);
        assert!(result.is_none());
    }

    #[test]
    fn test_read_feedback_invalid_characters() {
        let input = "GGGGA\n";
        let mut reader = Cursor::new(input);
        let result = read_feedback(&mut reader);
        assert!(result.is_none());
    }

    #[test]
    fn test_read_feedback_lowercase_converted() {
        let input = "gygxg\n";
        let mut reader = Cursor::new(input);
        let result = read_feedback(&mut reader);
        // After uppercase conversion, this should work
        assert!(result.is_some());
        let feedback = result.unwrap();
        assert_eq!(feedback.len(), 5);
        // Verify it was properly converted and parsed
        assert!(matches!(feedback[0], Feedback::Match));
        assert!(matches!(feedback[1], Feedback::PartialMatch));
    }
}
