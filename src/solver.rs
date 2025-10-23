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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feedback_from_char() {
        assert_eq!(Feedback::from_char('G'), Some(Feedback::Match));
        assert_eq!(Feedback::from_char('Y'), Some(Feedback::PartialMatch));
        assert_eq!(Feedback::from_char('X'), Some(Feedback::NoMatch));
        assert_eq!(Feedback::from_char('Z'), None);
        assert_eq!(Feedback::from_char('g'), None);
    }

    #[test]
    fn test_feedback_as_char() {
        assert_eq!(Feedback::Match.as_char(), 'G');
        assert_eq!(Feedback::PartialMatch.as_char(), 'Y');
        assert_eq!(Feedback::NoMatch.as_char(), 'X');
    }

    #[test]
    fn test_get_feedback_all_correct() {
        let feedback = get_feedback("CRANE", "CRANE");
        assert_eq!(feedback, vec![
            Feedback::Match,
            Feedback::Match,
            Feedback::Match,
            Feedback::Match,
            Feedback::Match
        ]);
    }

    #[test]
    fn test_get_feedback_all_wrong() {
        let feedback = get_feedback("CRANE", "BOILS");
        assert_eq!(feedback, vec![
            Feedback::NoMatch,
            Feedback::NoMatch,
            Feedback::NoMatch,
            Feedback::NoMatch,
            Feedback::NoMatch
        ]);
    }

    #[test]
    fn test_get_feedback_partial_matches() {
        let feedback = get_feedback("CRANE", "NACRE");
        assert_eq!(feedback, vec![
            Feedback::PartialMatch, // C is in solution but wrong position
            Feedback::PartialMatch, // R is in solution but wrong position
            Feedback::PartialMatch, // A is in solution but wrong position
            Feedback::PartialMatch, // N is in solution but wrong position
            Feedback::Match         // E is in correct position
        ]);
    }

    #[test]
    fn test_get_feedback_mixed() {
        let feedback = get_feedback("RAISE", "AROSE");
        assert_eq!(feedback, vec![
            Feedback::PartialMatch, // R is in solution but wrong position
            Feedback::PartialMatch, // A is in solution but wrong position
            Feedback::NoMatch,      // I not in solution
            Feedback::Match,        // S is correct
            Feedback::Match         // E is correct
        ]);
    }

    #[test]
    fn test_get_feedback_duplicate_letters_both_present() {
        // Guess has three E's, solution has two E's (ELEGY = E_E__)
        let feedback = get_feedback("EERIE", "ELEGY");
        assert_eq!(feedback, vec![
            Feedback::Match,        // E correct position
            Feedback::PartialMatch, // E in solution but wrong position (matches position 3)
            Feedback::NoMatch,      // R not in solution
            Feedback::NoMatch,      // I not in solution
            Feedback::NoMatch       // E already used (only 2 E's in solution)
        ]);
    }

    #[test]
    fn test_get_feedback_duplicate_letters_one_correct() {
        // Guess has two L's, solution has one L at position 1
        let feedback = get_feedback("SKILL", "SLATE");
        assert_eq!(feedback, vec![
            Feedback::Match,        // S correct
            Feedback::NoMatch,      // K not in solution
            Feedback::NoMatch,      // I not in solution
            Feedback::PartialMatch, // L in solution but wrong position
            Feedback::NoMatch       // L already used (only one L in solution)
        ]);
    }

    #[test]
    fn test_get_feedback_duplicate_letters_one_yellow() {
        // Guess has two O's, solution has one O at position 1
        let feedback = get_feedback("ROBOT", "WORLD");
        assert_eq!(feedback, vec![
            Feedback::PartialMatch, // R in solution but wrong position
            Feedback::Match,        // O correct position
            Feedback::NoMatch,      // B not in solution
            Feedback::NoMatch,      // O already used (only one O in WORLD)
            Feedback::NoMatch       // T not in solution
        ]);
    }

    #[test]
    fn test_filter_candidates_all_green() {
        let candidates = vec!["CRANE".to_string(), "TRAIN".to_string(), "BRAIN".to_string()];
        let feedback = vec![
            Feedback::NoMatch,      // T not at position 0
            Feedback::Match,        // R at position 1
            Feedback::Match,        // A at position 2
            Feedback::Match,        // I at position 3
            Feedback::Match         // N at position 4
        ];
        let result = filter_candidates(&candidates, "TRAIN", &feedback);
        // Only BRAIN matches: _RAIN pattern with no T
        assert_eq!(result, vec!["BRAIN"]);
    }

    #[test]
    fn test_filter_candidates_yellow() {
        let candidates = vec![
            "BRAKE".to_string(),
            "TRACE".to_string(),
            "GRACE".to_string(),
            "CRAVE".to_string()
        ];
        let feedback = vec![
            Feedback::PartialMatch, // C in word but not position 0
            Feedback::PartialMatch, // R in word but not position 1
            Feedback::Match,        // A at position 2
            Feedback::NoMatch,      // N not in word
            Feedback::Match         // E at position 4
        ];
        let result = filter_candidates(&candidates, "CRANE", &feedback);
        // We need words with C elsewhere (not pos 0), R elsewhere (not pos 1), A at 2, E at 4
        assert_eq!(result.len(), 0); // None of these candidates should match
    }

    #[test]
    fn test_filter_candidates_gray_eliminates() {
        let candidates = vec![
            "CRANE".to_string(),
            "BRAIN".to_string(),
            "STAIN".to_string(),
            "PLAIN".to_string()
        ];
        let feedback = vec![
            Feedback::NoMatch,
            Feedback::NoMatch,
            Feedback::NoMatch,
            Feedback::NoMatch,
            Feedback::NoMatch
        ];
        let result = filter_candidates(&candidates, "CRANE", &feedback);
        // Should eliminate any word containing C, R, A, N, or E
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_filter_candidates_complex_scenario() {
        let candidates = vec![
            "BEAST".to_string(),
            "LEAST".to_string(),
            "FEAST".to_string(),
            "YEAST".to_string(),
            "TOAST".to_string()
        ];
        let feedback = vec![
            Feedback::NoMatch,      // R not in word
            Feedback::Match,        // E correct position
            Feedback::PartialMatch, // A in word but wrong position
            Feedback::NoMatch,      // I not in word
            Feedback::NoMatch       // S not in word
        ];
        let result = filter_candidates(&candidates, "REAIS", &feedback);
        // Should keep words with E at position 1, A elsewhere, no R/I/S
        assert!(result.iter().all(|w| w.chars().nth(1).unwrap() == 'E'));
        assert!(result.iter().all(|w| w.contains('A')));
    }

    #[test]
    fn test_filter_candidates_gray_with_duplicate() {
        // If a letter appears twice in guess, and one is green/yellow and one is gray,
        // the word should not have MORE instances of that letter
        let candidates = vec![
            "SPEED".to_string(),
            "CREEP".to_string(),
            "SHELF".to_string()
        ];
        let feedback = vec![
            Feedback::Match,    // S correct
            Feedback::NoMatch,  // K not in word
            Feedback::NoMatch,  // I not in word
            Feedback::Match,    // L correct
            Feedback::NoMatch   // Second L is gray (only one L in solution)
        ];
        let result = filter_candidates(&candidates, "SKILL", &feedback);
        // Should keep only words with S at position 0, L at position 3, and no extra L
        assert_eq!(result, vec!["SHELF"]);
    }

    #[test]
    fn test_expected_pool_size_single_candidate() {
        let candidates = vec!["CRANE".to_string()];
        let score = expected_pool_size("CRANE", &candidates);
        // With one candidate, any guess should result in score of 1.0
        assert_eq!(score, 1.0);
    }

    #[test]
    fn test_expected_pool_size_multiple_candidates() {
        let candidates = vec![
            "CRANE".to_string(),
            "CRATE".to_string(),
            "CRAZE".to_string()
        ];
        let score = expected_pool_size("CRATE", &candidates);
        // Score should be > 0 and < candidates.len()
        assert!(score > 0.0);
        assert!(score <= candidates.len() as f64);
    }

    #[test]
    fn test_expected_pool_size_worst_case() {
        // If all candidates give the same feedback, score equals number of candidates
        let candidates = vec![
            "AAAAA".to_string(),
            "AAAAA".to_string(),
            "AAAAA".to_string()
        ];
        let score = expected_pool_size("BBBBB", &candidates);
        // All give same feedback (all gray), so pool size is 3.0
        assert_eq!(score, 3.0);
    }

    #[test]
    fn test_best_information_guess_finds_optimal() {
        let wordbank = vec![
            "CRANE".to_string(),
            "SLATE".to_string(),
            "RAISE".to_string(),
            "STARE".to_string()
        ];
        let candidates = vec![
            "CRANE".to_string(),
            "SLATE".to_string()
        ];
        let (guess, score, is_candidate) = best_information_guess(&wordbank, &candidates);

        // Should return a valid word from wordbank
        assert!(wordbank.contains(&guess.to_string()));
        // Score should be positive and reasonable
        assert!(score > 0.0);
        assert!(score <= candidates.len() as f64);
        // Should indicate if it's a candidate or not
        assert_eq!(is_candidate, candidates.contains(guess));
    }

    #[test]
    fn test_best_information_guess_prefers_lower_score() {
        let wordbank = vec![
            "AAAAA".to_string(),
            "BBBBB".to_string(),
            "CCCCC".to_string(),
            "CRANE".to_string(),
            "TRAIN".to_string(),
            "BRAIN".to_string()
        ];
        let candidates = vec![
            "CRANE".to_string(),
            "TRAIN".to_string(),
            "BRAIN".to_string()
        ];
        let (guess, _, _) = best_information_guess(&wordbank, &candidates);

        // One of the actual candidates should be better than words with no shared letters
        assert!(
            guess == "CRANE" || guess == "TRAIN" || guess == "BRAIN",
            "Expected a candidate word but got: {}", guess
        );
    }

    #[test]
    fn test_compute_best_starting_words_returns_five() {
        let wordbank = vec![
            "CRANE".to_string(),
            "SLATE".to_string(),
            "RAISE".to_string(),
            "STARE".to_string(),
            "ARISE".to_string(),
            "ATONE".to_string(),
            "IRATE".to_string()
        ];
        let starting_words = compute_best_starting_words(&wordbank);

        assert_eq!(starting_words.len(), 5);
        // All should be from the wordbank
        assert!(starting_words.iter().all(|w| wordbank.contains(w)));
    }

    #[test]
    fn test_compute_best_starting_words_with_small_wordbank() {
        let wordbank = vec![
            "CRANE".to_string(),
            "SLATE".to_string()
        ];
        let starting_words = compute_best_starting_words(&wordbank);

        // Should return at most 5, but only 2 available
        assert_eq!(starting_words.len(), 2);
    }
}

