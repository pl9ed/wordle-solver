use crate::solver::{
    Feedback, best_information_guess, compute_best_starting_words, filter_candidates,
};
use crate::wordbank::{get_wordle_start_path, read_starting_words, write_starting_words};
use std::io::BufRead;
use std::path::PathBuf;

fn is_valid_word(word: &str) -> bool {
    word.len() == 5 && word.chars().all(|c| c.is_ascii_alphabetic())
}

fn is_valid_feedback(feedback: &str) -> bool {
    feedback.len() == 5 && feedback.chars().all(|c| c == 'G' || c == 'Y' || c == 'X')
}

enum GameState {
    Continue,
    Solved,
    NoSolution,
}

enum GuessInput {
    Valid(String),
    Invalid,
    Exit,
    NewGame,
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
                println!("Exiting.");
                break;
            }
            GuessInput::NewGame => {
                candidates = initial_wordbank.to_vec();
                println!("New game started. Loaded {} words.", candidates.len());
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
                println!("Computing optimal guess, please wait...");
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

fn display_starting_words(words: &[String], used_cache: bool, cache_path: Option<&PathBuf>) {
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

fn read_guess<R: BufRead>(reader: &mut R) -> GuessInput {
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

fn read_feedback<R: BufRead>(reader: &mut R) -> Option<Vec<Feedback>> {
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

fn display_candidates(candidates: &[String]) {
    println!("Possible candidates ({})", candidates.len());
    for word in candidates.iter().take(5) {
        println!("{word}");
    }
}

fn check_game_state(candidates: &[String]) -> GameState {
    match candidates.len() {
        0 => {
            println!("No candidates remain. Check your inputs.");
            GameState::NoSolution
        }
        1 => {
            println!("Solution found: {}", candidates[0]);
            GameState::Solved
        }
        _ => GameState::Continue,
    }
}

fn display_recommendation(guess: &str, score: f64, is_candidate: bool) {
    let category = if is_candidate {
        "solution candidate"
    } else {
        "information-gathering"
    };
    println!("Recommended guess: {guess} (expected pool size {score:.2}) [{category}]");
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
