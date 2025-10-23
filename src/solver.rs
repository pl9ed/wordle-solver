use std::collections::HashMap;

pub fn filter_candidates(
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

pub fn build_freq_chart(words: &[String]) -> [[usize; 26]; 5] {
    let mut freq = [[0; 26]; 5];
    for word in words {
        for (i, c) in word.chars().enumerate() {
            let idx = (c as u8 - b'A') as usize;
            freq[i][idx] += 1;
        }
    }
    freq
}

pub fn score_word(word: &str, freq: &[[usize; 26]; 5]) -> usize {
    word.chars().enumerate().map(|(i, c)| {
        let idx = (c as u8 - b'A') as usize;
        freq[i][idx]
    }).sum()
}

pub fn recommend_guess(candidates: &[String]) -> Option<&String> {
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

pub fn get_feedback(guess: &str, solution: &str) -> String {
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

pub fn expected_pool_size(guess: &str, candidates: &[String]) -> f64 {
    let mut pattern_counts: HashMap<String, usize> = HashMap::new();
    for solution in candidates {
        let pattern = get_feedback(guess, solution);
        *pattern_counts.entry(pattern).or_insert(0) += 1;
    }
    let total = candidates.len() as f64;
    pattern_counts.values().map(|&count| (count as f64).powi(2)).sum::<f64>() / total
}

pub fn best_information_guess<'a>(wordbank: &'a [String], candidates: &'a [String]) -> (&'a String, f64, bool) {
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

