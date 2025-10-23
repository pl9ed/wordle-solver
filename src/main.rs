use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::env;

const EMBEDDED_WORDBANK: &str = include_str!("resources/wordbank.txt");

fn load_wordbank_from_str(data: &str) -> Vec<String> {
    data.lines()
        .map(|line| line.trim().to_uppercase())
        .filter(|word| word.len() == 5 && word.chars().all(|c| c.is_ascii_alphabetic()))
        .collect()
}

fn load_wordbank_from_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut words = Vec::new();
    for line in reader.lines() {
        let word = line?.trim().to_uppercase();
        if word.len() == 5 && word.chars().all(|c| c.is_ascii_alphabetic()) {
            words.push(word);
        }
    }
    Ok(words)
}

fn get_wordbank() -> io::Result<Vec<String>> {
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "-i" {
            if let Some(path) = args.next() {
                return load_wordbank_from_file(path);
            } else {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "Missing path after -i"));
            }
        }
    }
    Ok(load_wordbank_from_str(EMBEDDED_WORDBANK))
}

fn filter_candidates(
    candidates: &[String],
    guess: &str,
    feedback: &str,
) -> Vec<String> {
    let mut filtered = Vec::new();
    'word: for word in candidates {
        // First pass: check greens
        for (i, (g, f)) in guess.chars().zip(feedback.chars()).enumerate() {
            if f == 'G' && word.chars().nth(i).unwrap() != g {
                continue 'word;
            }
        }
        // Second pass: check yellows
        for (i, (g, f)) in guess.chars().zip(feedback.chars()).enumerate() {
            if f == 'Y' {
                if word.chars().nth(i).unwrap() == g {
                    continue 'word;
                }
                if !word.contains(g) {
                    continue 'word;
                }
            }
        }
        // Third pass: check greys (X)
        for (i, (g, f)) in guess.chars().zip(feedback.chars()).enumerate() {
            if f == 'X' {
                let elsewhere = guess.chars().enumerate().any(|(j, gc)| {
                    gc == g && (feedback.chars().nth(j).unwrap() == 'G' || feedback.chars().nth(j).unwrap() == 'Y')
                });
                if elsewhere {
                    // Must not be at this position
                    if word.chars().nth(i).unwrap() == g {
                        continue 'word;
                    }
                } else {
                    // Must not be anywhere
                    if word.contains(g) {
                        continue 'word;
                    }
                }
            }
        }
        filtered.push(word.clone());
    }
    filtered
}

fn build_freq_chart(words: &[String]) -> [[usize; 26]; 5] {
    let mut freq = [[0; 26]; 5];
    for word in words {
        for (i, c) in word.chars().enumerate() {
            let idx = (c as u8 - b'A') as usize;
            freq[i][idx] += 1;
        }
    }
    freq
}

fn score_word(word: &str, freq: &[[usize; 26]; 5]) -> usize {
    word.chars().enumerate().map(|(i, c)| {
        let idx = (c as u8 - b'A') as usize;
        freq[i][idx]
    }).sum()
}

fn recommend_guess(candidates: &[String]) -> Option<&String> {
    let freq = build_freq_chart(candidates);
    let mut best_score = 0;
    let mut best_word = None;
    for word in candidates {
        let score = score_word(word, &freq);
        if score > best_score {
            best_score = score;
            best_word = Some(word);
        }
    }
    best_word
}

fn get_feedback(guess: &str, solution: &str) -> String {
    let mut feedback = ['X'; 5];
    let mut solution_chars: Vec<char> = solution.chars().collect();
    let guess_chars: Vec<char> = guess.chars().collect();
    // First pass: greens
    for i in 0..5 {
        if guess_chars[i] == solution_chars[i] {
            feedback[i] = 'G';
            solution_chars[i] = '_'; // Mark as used
        }
    }
    // Second pass: yellows
    for i in 0..5 {
        if feedback[i] == 'G' { continue; }
        if let Some(pos) = solution_chars.iter().position(|&c| c == guess_chars[i]) {
            feedback[i] = 'Y';
            solution_chars[pos] = '_'; // Mark as used
        }
    }
    feedback.iter().collect()
}

fn expected_pool_size(guess: &str, candidates: &[String]) -> f64 {
    use std::collections::HashMap;
    let mut pattern_counts: HashMap<String, usize> = HashMap::new();
    for solution in candidates {
        let pattern = get_feedback(guess, solution);
        *pattern_counts.entry(pattern).or_insert(0) += 1;
    }
    let total = candidates.len() as f64;
    pattern_counts.values().map(|&count| (count as f64).powi(2)).sum::<f64>() / total
}

fn best_information_guess<'a>(wordbank: &'a [String], candidates: &'a [String]) -> (&'a String, f64, bool) {
    let mut best_word = &wordbank[0];
    let mut best_score = f64::INFINITY;
    let mut is_candidate = false;
    for guess in wordbank {
        let score = expected_pool_size(guess, candidates);
        if score < best_score {
            best_word = guess;
            best_score = score;
            is_candidate = candidates.contains(guess);
        }
    }
    (best_word, best_score, is_candidate)
}

fn main() {
    let initial_wordbank = match get_wordbank() {
        Ok(words) => words,
        Err(e) => {
            eprintln!("Failed to load word bank: {}", e);
            return;
        }
    };
    let mut candidates = initial_wordbank.clone();
    println!("Loaded {} words.", candidates.len());
    if let Some(start_word) = recommend_guess(&candidates) {
        println!("Suggested starting word: {}", start_word);
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
            candidates = initial_wordbank.clone();
            println!("New game started. Loaded {} words.", candidates.len());
            if let Some(start_word) = recommend_guess(&candidates) {
                println!("Suggested starting word: {}", start_word);
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
            println!("{}", word);
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
