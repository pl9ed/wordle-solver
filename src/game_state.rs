use crate::solver::{
    Feedback, best_information_guess, compute_best_starting_words, filter_candidates,
};
use crate::wordbank::{get_wordle_start_path, read_starting_words, write_starting_words};
use std::path::PathBuf;

enum GameState {
    Continue,
    Solved,
    NoSolution,
}

/// User action from input
#[derive(Debug)]
pub enum UserAction {
    Guess(String),
    Exit,
    NewGame,
}

/// Information about starting words to display
pub struct StartingWordsInfo {
    pub words: Vec<String>,
    pub used_cache: bool,
    pub cache_path: Option<PathBuf>,
}

/// Recommendation for the next guess
#[derive(Clone)]
pub struct Recommendation {
    pub guess: String,
    pub score: f64,
    pub is_candidate: bool,
}

/// Trait that abstracts the UI layer from game logic
/// Implement this trait for different UIs: CLI, TUI, GUI, API, etc.
pub trait GameInterface {
    /// Display the optimal starting words
    fn display_starting_words(&mut self, info: &StartingWordsInfo);

    /// Read the user's guess, returns None if input was invalid and should retry
    fn read_guess(&mut self) -> Option<UserAction>;

    /// Read feedback for a guess, returns None if input was invalid and should retry
    fn read_feedback(&mut self) -> Option<Vec<Feedback>>;

    /// Display the current candidate words
    fn display_candidates(&mut self, candidates: &[String]);

    /// Display a recommendation for the next guess
    fn display_recommendation(&mut self, recommendation: &Recommendation);

    /// Display a message when computing
    fn display_computing_message(&mut self);

    /// Display a message when no candidates remain
    fn display_no_candidates_message(&mut self);

    /// Display the solution when found
    fn display_solution_found(&mut self, solution: &str);

    /// Display exit message
    fn display_exit_message(&mut self);

    /// Display new game started message
    fn display_new_game_message(&mut self, word_count: usize);
}

pub fn game_loop<I: GameInterface>(initial_wordbank: &[String], interface: &mut I) {
    let start_path = get_wordle_start_path();
    let (starting_words, used_cache) =
        load_or_compute_starting_words(initial_wordbank, start_path.as_ref());

    let info = StartingWordsInfo {
        words: starting_words.clone(),
        used_cache,
        cache_path: start_path.clone(),
    };
    interface.display_starting_words(&info);

    let mut candidates = initial_wordbank.to_vec();

    loop {
        let action = loop {
            if let Some(action) = interface.read_guess() {
                break action;
            }
        };

        match action {
            UserAction::Exit => {
                interface.display_exit_message();
                break;
            }
            UserAction::NewGame => {
                candidates = initial_wordbank.to_vec();
                interface.display_new_game_message(candidates.len());
                let info = StartingWordsInfo {
                    words: starting_words.clone(),
                    used_cache: true,
                    cache_path: start_path.clone(),
                };
                interface.display_starting_words(&info);
            }
            UserAction::Guess(guess) => {
                let feedback = loop {
                    if let Some(fb) = interface.read_feedback() {
                        break fb;
                    }
                };

                candidates = filter_candidates(&candidates, &guess, &feedback);
                interface.display_candidates(&candidates);

                match check_game_state(&candidates, interface) {
                    GameState::Solved | GameState::NoSolution => {
                        // Don't break, let the loop continue so user can start a new game
                        // The game is now in GameOver state and will wait for N or ESC
                    }
                    GameState::Continue => {
                        interface.display_computing_message();
                        let (info_guess, info_score, is_candidate) =
                            best_information_guess(initial_wordbank, &candidates);
                        let recommendation = Recommendation {
                            guess: info_guess.to_string(),
                            score: info_score,
                            is_candidate,
                        };
                        interface.display_recommendation(&recommendation);
                    }
                }
            }
        }
    }
}

fn load_or_compute_starting_words(
    wordbank: &[String],
    start_path: Option<&PathBuf>,
) -> (Vec<String>, bool) {
    if let Some(path) = start_path
        && let Some(words) = read_starting_words(path)
    {
        return (words, true);
    }

    println!("Computing optimal starting words, please wait...");
    let words = compute_best_starting_words(wordbank);

    if let Some(path) = start_path {
        write_starting_words(path, &words);
    }

    (words, false)
}

fn check_game_state<I: GameInterface>(candidates: &[String], interface: &mut I) -> GameState {
    match candidates.len() {
        0 => {
            interface.display_no_candidates_message();
            GameState::NoSolution
        }
        1 => {
            interface.display_solution_found(&candidates[0]);
            GameState::Solved
        }
        _ => GameState::Continue,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::CliInterface;
    use std::io::Cursor;

    #[test]
    fn test_game_loop_immediate_exit() {
        let wordbank = vec![
            "CRANE".to_string(),
            "SLATE".to_string(),
            "RAISE".to_string(),
        ];
        let input = "exit\n";
        let reader = Cursor::new(input);
        let mut interface = CliInterface::new(reader);

        // Should not panic and should exit gracefully
        game_loop(&wordbank, &mut interface);
    }

    #[test]
    fn test_game_loop_invalid_guess_then_exit() {
        let wordbank = vec![
            "CRANE".to_string(),
            "SLATE".to_string(),
            "RAISE".to_string(),
        ];
        let input = "abc\nexit\n";
        let reader = Cursor::new(input);
        let mut interface = CliInterface::new(reader);

        // Should handle invalid input and then exit
        game_loop(&wordbank, &mut interface);
    }

    #[test]
    fn test_game_loop_new_game_command() {
        let wordbank = vec![
            "CRANE".to_string(),
            "SLATE".to_string(),
            "RAISE".to_string(),
        ];
        let input = "next\nexit\n";
        let reader = Cursor::new(input);
        let mut interface = CliInterface::new(reader);

        // Should start new game and then exit
        game_loop(&wordbank, &mut interface);
    }

    #[test]
    fn test_game_loop_valid_guess_invalid_feedback() {
        let wordbank = vec![
            "CRANE".to_string(),
            "SLATE".to_string(),
            "RAISE".to_string(),
        ];
        let input = "CRANE\nINVALID\nXXXXX\nexit\n";
        let reader = Cursor::new(input);
        let mut interface = CliInterface::new(reader);

        // Should reject invalid feedback and continue
        game_loop(&wordbank, &mut interface);
    }

    #[test]
    fn test_game_loop_valid_guess_short_feedback() {
        let wordbank = vec![
            "CRANE".to_string(),
            "SLATE".to_string(),
            "RAISE".to_string(),
        ];
        // After short feedback, provide valid feedback to complete the guess, then exit
        let input = "CRANE\nGGG\nXXXXX\nexit\n";
        let reader = Cursor::new(input);
        let mut interface = CliInterface::new(reader);

        // Should reject feedback that's not 5 characters
        game_loop(&wordbank, &mut interface);
    }

    #[test]
    fn test_game_loop_complete_game_win() {
        let wordbank = vec![
            "CRANE".to_string(),
            "SLATE".to_string(),
            "RAISE".to_string(),
        ];
        let input = "CRANE\nGGGGG\nexit\n";
        let reader = Cursor::new(input);
        let mut interface = CliInterface::new(reader);

        // Should find the solution and exit
        game_loop(&wordbank, &mut interface);
    }

    #[test]
    fn test_game_loop_narrowing_down() {
        let wordbank = vec![
            "CRANE".to_string(),
            "SLATE".to_string(),
            "RAISE".to_string(),
            "STARE".to_string(),
        ];
        // First guess eliminates some candidates, second guess finds solution
        let input = "CRANE\nXXXXX\nSLATE\nGGGGG\nexit\n";
        let reader = Cursor::new(input);
        let mut interface = CliInterface::new(reader);

        game_loop(&wordbank, &mut interface);
    }

    #[test]
    fn test_game_loop_no_candidates_remain() {
        let wordbank = vec!["CRANE".to_string(), "SLATE".to_string()];
        // Give feedback that eliminates all candidates
        let input = "CRANE\nXXXXX\nSLATE\nXXXXX\nexit\n";
        let reader = Cursor::new(input);
        let mut interface = CliInterface::new(reader);

        // Should detect no solution and exit
        game_loop(&wordbank, &mut interface);
    }

    #[test]
    fn test_game_loop_case_insensitive_guess() {
        let wordbank = vec!["CRANE".to_string(), "SLATE".to_string()];
        let input = "crane\nGGGGG\nexit\n";
        let reader = Cursor::new(input);
        let mut interface = CliInterface::new(reader);

        // Should accept lowercase and convert to uppercase
        game_loop(&wordbank, &mut interface);
    }

    #[test]
    fn test_game_loop_case_insensitive_feedback() {
        let wordbank = vec!["CRANE".to_string(), "SLATE".to_string()];
        let input = "CRANE\nggggg\nexit\n";
        let reader = Cursor::new(input);
        let mut interface = CliInterface::new(reader);

        // Should accept lowercase feedback
        game_loop(&wordbank, &mut interface);
    }

    #[test]
    fn test_game_loop_mixed_feedback() {
        let wordbank = vec![
            "CRANE".to_string(),
            "SLATE".to_string(),
            "RAISE".to_string(),
            "STARE".to_string(),
            "SPARE".to_string(),
        ];
        // Give mixed feedback with greens, yellows, and grays
        let input = "CRANE\nXYGXX\nSLATE\nGGGGG\nexit\n";
        let reader = Cursor::new(input);
        let mut interface = CliInterface::new(reader);

        game_loop(&wordbank, &mut interface);
    }

    #[test]
    fn test_game_loop_multiple_games() {
        let wordbank = vec![
            "CRANE".to_string(),
            "SLATE".to_string(),
            "RAISE".to_string(),
        ];
        // Play one game, start new game, then exit
        let input = "CRANE\nGGGGG\nnext\nSLATE\nGGGGG\nexit\n";
        let reader = Cursor::new(input);
        let mut interface = CliInterface::new(reader);

        game_loop(&wordbank, &mut interface);
    }

    #[test]
    fn test_game_loop_with_whitespace_in_input() {
        let wordbank = vec!["CRANE".to_string(), "SLATE".to_string()];
        let input = "  CRANE  \n  GGGGG  \nexit\n";
        let reader = Cursor::new(input);
        let mut interface = CliInterface::new(reader);

        // Should trim whitespace from input
        game_loop(&wordbank, &mut interface);
    }

    #[test]
    fn test_game_loop_six_letter_word_rejected() {
        let wordbank = vec!["CRANE".to_string(), "SLATE".to_string()];
        let input = "CRANES\nexit\n";
        let reader = Cursor::new(input);
        let mut interface = CliInterface::new(reader);

        // Should reject word that's too long
        game_loop(&wordbank, &mut interface);
    }

    #[test]
    fn test_game_loop_four_letter_word_rejected() {
        let wordbank = vec!["CRANE".to_string(), "SLATE".to_string()];
        let input = "CRAN\nexit\n";
        let reader = Cursor::new(input);
        let mut interface = CliInterface::new(reader);

        // Should reject word that's too short
        game_loop(&wordbank, &mut interface);
    }

    #[test]
    fn test_game_loop_word_with_numbers_rejected() {
        let wordbank = vec!["CRANE".to_string(), "SLATE".to_string()];
        let input = "CR4NE\nexit\n";
        let reader = Cursor::new(input);
        let mut interface = CliInterface::new(reader);

        // Should reject word with non-alphabetic characters
        game_loop(&wordbank, &mut interface);
    }

    #[test]
    fn test_game_loop_progressive_narrowing() {
        let wordbank = vec![
            "AAAAA".to_string(),
            "BBBBB".to_string(),
            "CCCCC".to_string(),
            "DDDDD".to_string(),
            "EEEEE".to_string(),
            "FFFFF".to_string(),
        ];
        // Progressively narrow down candidates
        let input = "AAAAA\nXXXXX\nBBBBB\nXXXXX\nCCCCC\nGGGGG\nexit\n";
        let reader = Cursor::new(input);
        let mut interface = CliInterface::new(reader);

        game_loop(&wordbank, &mut interface);
    }
}
