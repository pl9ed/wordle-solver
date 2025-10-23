use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Feedback {
    Match,        // Green ('G') - correct letter in correct position
    PartialMatch, // Yellow ('Y') - correct letter in wrong position
    NoMatch,      // Gray ('X') - letter not in word
}

impl Feedback {
    /// Convert this feedback to its character representation
    #[allow(dead_code)]
    pub const fn as_char(self) -> char {
        match self {
            Self::Match => 'G',
            Self::PartialMatch => 'Y',
            Self::NoMatch => 'X',
        }
    }

    /// Parse a character into a Feedback variant
    pub const fn from_char(c: char) -> Option<Self> {
        match c {
            'G' => Some(Self::Match),
            'Y' => Some(Self::PartialMatch),
            'X' => Some(Self::NoMatch),
            _ => None,
        }
    }
}

pub fn filter_candidates(
    candidates: &[String],
    guess: &str,
    feedback: &[Feedback],
) -> Vec<String> {
    let guess_chars: Vec<char> = guess.chars().collect();

    let mut filtered = Vec::new();
    'word: for word in candidates {
        let word_chars: Vec<char> = word.chars().collect();

        // First pass: check matches (green)
        for (i, (&g, &f)) in guess_chars.iter().zip(feedback.iter()).enumerate() {
            if f == Feedback::Match && word_chars[i] != g {
                continue 'word;
            }
        }
        // Second pass: check partial matches (yellow)
        for (i, (&g, &f)) in guess_chars.iter().zip(feedback.iter()).enumerate() {
            if f == Feedback::PartialMatch {
                if word_chars[i] == g {
                    continue 'word;
                }
                if !word_chars.contains(&g) {
                    continue 'word;
                }
            }
        }
        // Third pass: check no matches (gray)
        for (i, (&g, &f)) in guess_chars.iter().zip(feedback.iter()).enumerate() {
            if f == Feedback::NoMatch {
                let elsewhere = guess_chars.iter().zip(feedback.iter()).any(|(&gc, &fc)| {
                    gc == g && (fc == Feedback::Match || fc == Feedback::PartialMatch)
                });
                if elsewhere {
                    // Must not be at this position
                    if word_chars[i] == g {
                        continue 'word;
                    }
                } else {
                    // Must not be anywhere
                    if word_chars.contains(&g) {
                        continue 'word;
                    }
                }
            }
        }
        filtered.push(word.clone());
    }
    filtered
}

pub fn get_feedback(guess: &str, solution: &str) -> Vec<Feedback> {
    let mut feedback = [Feedback::NoMatch; 5];
    let mut solution_chars: Vec<char> = solution.chars().collect();
    let guess_chars: Vec<char> = guess.chars().collect();
    // First pass: matches (green)
    for i in 0..5 {
        if guess_chars[i] == solution_chars[i] {
            feedback[i] = Feedback::Match;
            solution_chars[i] = '_'; // Mark as used
        }
    }
    // Second pass: partial matches (yellow)
    for i in 0..5 {
        if feedback[i] == Feedback::Match { continue; }
        if let Some(pos) = solution_chars.iter().position(|&c| c == guess_chars[i]) {
            feedback[i] = Feedback::PartialMatch;
            solution_chars[pos] = '_'; // Mark as used
        }
    }
    feedback.to_vec()
}

#[allow(clippy::cast_precision_loss)] // don't care about this
pub fn expected_pool_size(guess: &str, candidates: &[String]) -> f64 {
    let mut pattern_counts: HashMap<Vec<Feedback>, usize> = HashMap::new();
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

pub fn compute_best_starting_words(wordbank: &[String]) -> Vec<String> {
    let mut scored: Vec<(String, f64)> = wordbank.iter()
        .map(|w| (w.clone(), expected_pool_size(w, wordbank)))
        .collect();
    scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    scored.into_iter().take(5).map(|(w, _)| w).collect()
}
