mod cli;
mod wordbank;
mod solver;

use cli::{parse_cli};
use wordbank::{load_wordbank_from_file, load_wordbank_from_str, EMBEDDED_WORDBANK};
use solver::{filter_candidates, build_freq_chart, score_word, recommend_guess, best_information_guess};
use std::io;

fn main() {
    let cli = parse_cli();
    let initial_wordbank = match &cli.wordbank_path {
        Some(path) => match load_wordbank_from_file(path) {
            Ok(words) => words,
            Err(e) => {
                eprintln!("Failed to load word bank from '{path}': {e}");
                return;
            }
        },
        None => load_wordbank_from_str(EMBEDDED_WORDBANK),
    };
    let mut candidates = initial_wordbank.clone();
    println!("Loaded {} words.", candidates.len());
    if let Some(start_word) = recommend_guess(&candidates) {
        println!("Suggested starting word: {start_word}");
    }
    let stdin = io::stdin();
    loop {
        println!("\nEnter your guess (5 letters, or 'exit' to quit, or 'next' to start a new game):");
        let mut guess = String::new();
        stdin.read_line(&mut guess).unwrap();
        let guess = guess.trim().to_uppercase();
        if guess == "EXIT" {
            println!("Exiting.");
            break;
        }
        if guess == "NEXT" {
            candidates.clone_from(&initial_wordbank);
            println!("New game started. Loaded {0} words.", candidates.len());
            if let Some(start_word) = recommend_guess(&candidates) {
                println!("Suggested starting word: {start_word}");
            }
            continue;
        }
        if guess.len() != 5 || !guess.chars().all(|c| c.is_ascii_alphabetic()) {
            println!("Invalid guess. Please enter 5 letters.");
            continue;
        }
        println!("Enter feedback (G=green, Y=yellow, X=gray, e.g. GYXXG):");
        let mut feedback = String::new();
        stdin.read_line(&mut feedback).unwrap();
        let feedback = feedback.trim().to_uppercase();
        if feedback.len() != 5 || !feedback.chars().all(|c| c == 'G' || c == 'Y' || c == 'X') {
            println!("Invalid feedback. Please enter 5 characters using G, Y, or X.");
            continue;
        }
        candidates = filter_candidates(&candidates, &guess, &feedback);
        let freq = build_freq_chart(&candidates);
        let mut scored_candidates: Vec<(String, usize)> = candidates.iter()
            .map(|w| (w.clone(), score_word(w, &freq)))
            .collect();
        scored_candidates.sort_by(|a, b| b.1.cmp(&a.1));
        println!("Possible candidates ({}):", scored_candidates.len());
        for (word, _) in scored_candidates.iter().take(5) {
            println!("{word}");
        }
        if scored_candidates.len() > 5 {
            println!("...and {} more", scored_candidates.len() - 5);
        }
        if candidates.len() == 1 {
            println!("Solution found: {}", candidates[0]);
            break;
        }
        if candidates.is_empty() {
            println!("No candidates remain. Check your inputs.");
            break;
        }
        let (info_guess, info_score, is_candidate) = best_information_guess(&initial_wordbank, &candidates);
        println!("Recommended guess: {} (expected pool size {:.2}) [{}]", info_guess, info_score, if is_candidate { "solution candidate" } else { "information-gathering" });
    }
}
