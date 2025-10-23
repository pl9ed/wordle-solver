use crate::wordbank::{get_wordle_start_path, read_starting_words, write_starting_words};
use crate::solver::{filter_candidates, best_information_guess, compute_best_starting_words, Feedback};
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
    let (starting_words, used_cache) = load_or_compute_starting_words(initial_wordbank, start_path.as_ref());
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

        let Some(feedback) = read_feedback(&mut reader) else { continue };

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
        && let Some(words) = read_starting_words(path) {
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
        let feedback: Option<Vec<Feedback>> = input.chars()
            .map(Feedback::from_char)
            .collect();
        
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

