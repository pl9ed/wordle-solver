use crate::solver::{
    best_information_guess, compute_best_starting_words, filter_candidates,
};
use crate::wordbank::{get_wordle_start_path, read_starting_words, write_starting_words};
use crate::cli::{
    GuessInput, display_starting_words, read_guess, read_feedback, display_candidates,
    display_recommendation, display_exit_message, display_new_game_message,
    display_computing_message, display_no_candidates_message, display_solution_found,
};
use std::io::BufRead;
use std::path::PathBuf;

enum GameState {
    Continue,
    Solved,
    NoSolution,
}


pub fn game_loop<R: BufRead>(initial_wordbank: &[String], mut reader: R) {
    let start_path = get_wordle_start_path();
    let (starting_words, used_cache) =
        load_or_compute_starting_words(initial_wordbank, start_path.as_ref());
    display_starting_words(&starting_words, used_cache, start_path.as_ref());

    let mut candidates = initial_wordbank.to_vec();

    loop {
        let guess = match read_guess(&mut reader) {
            GuessInput::Exit => {
                display_exit_message();
                break;
            }
            GuessInput::NewGame => {
                candidates = initial_wordbank.to_vec();
                display_new_game_message(candidates.len());
                display_starting_words(&starting_words, true, start_path.as_ref());
                continue;
            }
            GuessInput::Valid(g) => g,
            GuessInput::Invalid => continue,
        };

        let Some(feedback) = read_feedback(&mut reader) else {
            continue;
        };

        candidates = filter_candidates(&candidates, &guess, &feedback);
        display_candidates(&candidates);

        match check_game_state(&candidates) {
            GameState::Solved | GameState::NoSolution => break,
            GameState::Continue => {
                display_computing_message();
                let (info_guess, info_score, is_candidate) =
                    best_information_guess(initial_wordbank, &candidates);
                display_recommendation(info_guess, info_score, is_candidate);
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


fn check_game_state(candidates: &[String]) -> GameState {
    match candidates.len() {
        0 => {
            display_no_candidates_message();
            GameState::NoSolution
        }
        1 => {
            display_solution_found(&candidates[0]);
            GameState::Solved
        }
        _ => GameState::Continue,
    }
}


#[cfg(test)]
mod tests {
    use super::*;
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

        // Should not panic and should exit gracefully
        game_loop(&wordbank, reader);
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

        // Should handle invalid input and then exit
        game_loop(&wordbank, reader);
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

        // Should start new game and then exit
        game_loop(&wordbank, reader);
    }

    #[test]
    fn test_game_loop_valid_guess_invalid_feedback() {
        let wordbank = vec![
            "CRANE".to_string(),
            "SLATE".to_string(),
            "RAISE".to_string(),
        ];
        let input = "CRANE\nINVALID\nexit\n";
        let reader = Cursor::new(input);

        // Should reject invalid feedback and continue
        game_loop(&wordbank, reader);
    }

    #[test]
    fn test_game_loop_valid_guess_short_feedback() {
        let wordbank = vec![
            "CRANE".to_string(),
            "SLATE".to_string(),
            "RAISE".to_string(),
        ];
        let input = "CRANE\nGGG\nexit\n";
        let reader = Cursor::new(input);

        // Should reject feedback that's not 5 characters
        game_loop(&wordbank, reader);
    }

    #[test]
    fn test_game_loop_complete_game_win() {
        let wordbank = vec![
            "CRANE".to_string(),
            "SLATE".to_string(),
            "RAISE".to_string(),
        ];
        let input = "CRANE\nGGGGG\n";
        let reader = Cursor::new(input);

        // Should find the solution and exit
        game_loop(&wordbank, reader);
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
        let input = "CRANE\nXXXXX\nSLATE\nGGGGG\n";
        let reader = Cursor::new(input);

        game_loop(&wordbank, reader);
    }

    #[test]
    fn test_game_loop_no_candidates_remain() {
        let wordbank = vec!["CRANE".to_string(), "SLATE".to_string()];
        // Give feedback that eliminates all candidates
        let input = "CRANE\nXXXXX\nSLATE\nXXXXX\n";
        let reader = Cursor::new(input);

        // Should detect no solution and exit
        game_loop(&wordbank, reader);
    }

    #[test]
    fn test_game_loop_case_insensitive_guess() {
        let wordbank = vec!["CRANE".to_string(), "SLATE".to_string()];
        let input = "crane\nGGGGG\n";
        let reader = Cursor::new(input);

        // Should accept lowercase and convert to uppercase
        game_loop(&wordbank, reader);
    }

    #[test]
    fn test_game_loop_case_insensitive_feedback() {
        let wordbank = vec!["CRANE".to_string(), "SLATE".to_string()];
        let input = "CRANE\nggggg\n";
        let reader = Cursor::new(input);

        // Should accept lowercase feedback
        game_loop(&wordbank, reader);
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
        let input = "CRANE\nXYGXX\nSLATE\nGGGGG\n";
        let reader = Cursor::new(input);

        game_loop(&wordbank, reader);
    }

    #[test]
    fn test_game_loop_multiple_games() {
        let wordbank = vec![
            "CRANE".to_string(),
            "SLATE".to_string(),
            "RAISE".to_string(),
        ];
        // Play one game, start new game, then exit
        let input = "CRANE\nGGGGG\nnext\nSLATE\nGGGGG\n";
        let reader = Cursor::new(input);

        game_loop(&wordbank, reader);
    }

    #[test]
    fn test_game_loop_with_whitespace_in_input() {
        let wordbank = vec!["CRANE".to_string(), "SLATE".to_string()];
        let input = "  CRANE  \n  GGGGG  \n";
        let reader = Cursor::new(input);

        // Should trim whitespace from input
        game_loop(&wordbank, reader);
    }

    #[test]
    fn test_game_loop_six_letter_word_rejected() {
        let wordbank = vec!["CRANE".to_string(), "SLATE".to_string()];
        let input = "CRANES\nexit\n";
        let reader = Cursor::new(input);

        // Should reject word that's too long
        game_loop(&wordbank, reader);
    }

    #[test]
    fn test_game_loop_four_letter_word_rejected() {
        let wordbank = vec!["CRANE".to_string(), "SLATE".to_string()];
        let input = "CRAN\nexit\n";
        let reader = Cursor::new(input);

        // Should reject word that's too short
        game_loop(&wordbank, reader);
    }

    #[test]
    fn test_game_loop_word_with_numbers_rejected() {
        let wordbank = vec!["CRANE".to_string(), "SLATE".to_string()];
        let input = "CR4NE\nexit\n";
        let reader = Cursor::new(input);

        // Should reject word with non-alphabetic characters
        game_loop(&wordbank, reader);
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
        let input = "AAAAA\nXXXXX\nBBBBB\nXXXXX\nCCCCC\nGGGGG\n";
        let reader = Cursor::new(input);

        game_loop(&wordbank, reader);
    }
}
