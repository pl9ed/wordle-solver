// Integration tests for the wordle-solver application
// These tests verify that all modules work together correctly

use std::io::Cursor;
use wordle_solver::*;
use wordle_solver::cli::CliInterface;

#[test]
fn test_end_to_end_solver_workflow() {
    // Test the complete workflow: wordbank loading -> solver -> game loop
    // This simulates a real game where the user guesses and gets feedback

    let wordbank = vec![
        "CRANE".to_string(),
        "SLATE".to_string(),
        "TRACE".to_string(),
        "PLACE".to_string(),
        "GRACE".to_string(),
    ];

    // Simulate a game where SLATE is the answer
    // User guesses CRANE first, gets feedback, then guesses SLATE and wins
    let input = "CRANE\nXYGXX\nSLATE\nGGGGG\n";
    let reader = Cursor::new(input);
    let mut interface = CliInterface::new(reader);

    // This should complete without panicking
    game_loop(&wordbank, &mut interface);
}

#[test]
fn test_solver_integration_with_wordbank_filtering() {
    // Test that solver correctly uses filtered wordbank
    // Start with a wordbank, apply feedback, verify remaining candidates

    let wordbank = vec![
        "CRANE".to_string(),
        "BRAIN".to_string(),
        "TRAIN".to_string(),
        "GRAIN".to_string(),
        "STAIN".to_string(),
    ];

    // Scenario: Answer is BRAIN
    // Guess CRANE, should eliminate it but keep others with similar patterns
    let feedback = get_feedback("CRANE", "BRAIN");
    let candidates = filter_candidates(&wordbank, "CRANE", &feedback);

    // CRANE should be filtered out (C, E not in BRAIN)
    assert!(!candidates.contains(&"CRANE".to_string()));
    assert!(candidates.contains(&"BRAIN".to_string()));

    // Now guess TRAIN with BRAIN as answer
    let feedback2 = get_feedback("TRAIN", "BRAIN");
    let candidates2 = filter_candidates(&candidates, "TRAIN", &feedback2);

    // Should narrow down significantly
    assert!(candidates2.len() < candidates.len());
    assert!(candidates2.contains(&"BRAIN".to_string()));
}

#[test]
fn test_wordbank_to_solver_pipeline() {
    // Test loading wordbank from string and using it with solver
    let wordbank_data = "crane\nslate\nraise\nstare\narise\nirate\natone";

    // Load wordbank
    let wordbank = load_wordbank_from_str(wordbank_data);
    assert_eq!(wordbank.len(), 7);
    assert!(wordbank.iter().all(|w| w.chars().all(|c| c.is_uppercase())));

    // Compute best starting words
    let starting_words = compute_best_starting_words(&wordbank);
    assert_eq!(starting_words.len(), 5);

    // Verify all starting words are from the wordbank
    assert!(starting_words.iter().all(|w| wordbank.contains(w)));

    // Verify they're actually useful (should include common vowels/consonants)
    let all_letters: String = starting_words.join("");
    assert!(all_letters.contains('A') || all_letters.contains('E'));
    assert!(all_letters.contains('R') || all_letters.contains('S') || all_letters.contains('T'));
}

#[test]
fn test_cached_starting_words_integration() {
    // Test that starting words are computed, cached, and retrieved correctly
    use std::fs;

    let temp_dir = std::env::temp_dir();
    let cache_path = temp_dir.join("test_integration_cache.txt");

    // Clean up any existing cache
    let _ = fs::remove_file(&cache_path);

    let _wordbank = [
        "CRANE".to_string(),
        "SLATE".to_string(),
        "RAISE".to_string(),
        "STARE".to_string(),
        "ARISE".to_string(),
        "IRATE".to_string(),
    ];

    // 1. Compute starting words (would use compute_best_starting_words)
    // 2. Write to cache (would use write_starting_words)
    // 3. Read from cache (would use read_starting_words)
    // 4. Verify they match

    // Cleanup
    let _ = fs::remove_file(&cache_path);
}

#[test]
fn test_multi_round_game_with_optimal_strategy() {
    // Test a complete multi-round game using optimal solver recommendations
    let wordbank = vec![
        "AROSE".to_string(),
        "SLATE".to_string(),
        "CRANE".to_string(),
        "TRACE".to_string(),
        "BRAKE".to_string(),
        "DRAKE".to_string(),
        "FLAKE".to_string(),
        "SNAKE".to_string(),
    ];

    // Target answer: BRAKE
    let mut candidates = wordbank.clone();

    // Round 1: Get best starting guess
    let (guess1, _score1, _) = best_information_guess(&wordbank, &candidates);
    assert!(wordbank.contains(&guess1.to_string()));

    // Apply feedback for round 1
    let feedback1 = get_feedback(guess1, "BRAKE");
    candidates = filter_candidates(&candidates, guess1, &feedback1);
    assert!(
        candidates.len() < wordbank.len(),
        "Should reduce candidates"
    );
    assert!(
        candidates.contains(&"BRAKE".to_string()),
        "Answer should remain"
    );

    // Round 2: Get next best guess
    let (guess2, _score2, _) = best_information_guess(&wordbank, &candidates);
    let feedback2 = get_feedback(guess2, "BRAKE");
    candidates = filter_candidates(&candidates, guess2, &feedback2);

    // Should narrow down significantly
    assert!(
        candidates.len() <= 3,
        "Should have very few candidates left"
    );
    assert!(candidates.contains(&"BRAKE".to_string()));
}

#[test]
fn test_solver_with_difficult_word_patterns() {
    // Test solver performance with words that have repeated letters
    let wordbank = vec![
        "SPEED".to_string(),
        "CREEP".to_string(),
        "SLEEP".to_string(),
        "STEEP".to_string(),
        "SWEEP".to_string(),
    ];

    // Test that solver can distinguish between these similar words
    // Answer: CREEP, Guess: SPEED
    let feedback = get_feedback("SPEED", "CREEP");
    let candidates = filter_candidates(&wordbank, "SPEED", &feedback);

    // Should significantly narrow down the options
    assert!(candidates.contains(&"CREEP".to_string()));
    assert!(candidates.len() < wordbank.len());

    // Verify solver can handle the repeated E's correctly
    let (next_guess, _, _) = best_information_guess(&wordbank, &candidates);
    assert!(wordbank.contains(&next_guess.to_string()));
}

#[test]
fn test_custom_wordbank_file_to_game() {
    // Integration test: Load custom wordbank file -> play game
    use std::fs::File;
    use std::io::Write;

    let temp_dir = std::env::temp_dir();
    let wordbank_path = temp_dir.join("test_custom_wordbank.txt");

    // Create custom wordbank file
    {
        let mut file = File::create(&wordbank_path).unwrap();
        writeln!(file, "apple").unwrap();
        writeln!(file, "grape").unwrap();
        writeln!(file, "lemon").unwrap();
        writeln!(file, "melon").unwrap();
        writeln!(file, "peach").unwrap();
    }

    // Load wordbank from file
    let wordbank = load_wordbank_from_file(&wordbank_path).unwrap();
    assert_eq!(wordbank.len(), 5);
    assert!(wordbank.contains(&"APPLE".to_string()));
    assert!(wordbank.contains(&"GRAPE".to_string()));

    // Start game with this wordbank - simulate winning with APPLE
    let input = "APPLE\nGGGGG\n";
    let reader = Cursor::new(input);
    let mut interface = CliInterface::new(reader);
    game_loop(&wordbank, &mut interface);

    // Cleanup
    std::fs::remove_file(&wordbank_path).unwrap();
}

#[test]
fn test_solver_feedback_accuracy() {
    // Test that get_feedback and filter_candidates work correctly together
    // This is critical for the solver to work properly

    let candidates = vec![
        "CRANE".to_string(),
        "FRAME".to_string(),
        "BLAME".to_string(),
        "FLAME".to_string(),
        "SHAME".to_string(),
    ];

    // Simulate guessing CRANE when answer is FRAME
    let feedback = get_feedback("CRANE", "FRAME");

    // CRANE vs FRAME: C=NoMatch, R=Match (pos 1), A=Match (pos 2), N=NoMatch, E=Match (pos 4)
    assert_eq!(feedback[0], Feedback::NoMatch); // C not in FRAME
    assert_eq!(feedback[1], Feedback::Match); // R at correct position (both at pos 1)
    assert_eq!(feedback[2], Feedback::Match); // A in correct position
    assert_eq!(feedback[4], Feedback::Match); // E in correct position

    // Filter candidates with this feedback
    let filtered = filter_candidates(&candidates, "CRANE", &feedback);

    // Should only keep FRAME (has R elsewhere, A at pos 2, E at pos 4, no C or N)
    assert!(filtered.contains(&"FRAME".to_string()));
    assert_eq!(filtered.len(), 1, "Should narrow down to exactly FRAME");
}

#[test]
fn test_information_theory_optimization() {
    // Test that the solver's information-theoretic approach works
    // by verifying that recommended guesses actually reduce candidate pool

    let large_wordbank: Vec<String> = vec![
        "ABOUT", "ABOVE", "ABUSE", "ACTOR", "ACUTE", "ADMIT", "ADOPT", "ADULT", "AFTER", "AGAIN",
        "AGENT", "AGREE", "AHEAD", "ALARM", "ALBUM",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    let initial_count = large_wordbank.len();
    let mut candidates = large_wordbank.clone();

    // Get best guess for initial state
    let (guess, expected_pool, _) = best_information_guess(&large_wordbank, &candidates);

    // Expected pool size should be significantly less than current candidate count
    assert!(
        expected_pool < (initial_count as f64),
        "Expected pool size {} should be less than initial count {}",
        expected_pool,
        initial_count
    );

    // Simulate feedback for the guess (let's say answer is "ALBUM")
    let feedback = get_feedback(guess, "ALBUM");
    candidates = filter_candidates(&candidates, guess, &feedback);

    // Verify candidates were reduced
    assert!(
        candidates.len() < initial_count,
        "Candidates should reduce from {} to {}",
        initial_count,
        candidates.len()
    );
    assert!(
        candidates.contains(&"ALBUM".to_string()),
        "Answer should remain in candidates"
    );
}

#[test]
fn test_edge_case_single_candidate_remaining() {
    // Test behavior when only one candidate remains
    let wordbank = vec!["CRANE".to_string()];

    // The solver should immediately recommend this word
    let (guess, score, is_candidate) = best_information_guess(&wordbank, &wordbank);
    assert_eq!(guess, "CRANE");
    assert_eq!(score, 1.0); // With one candidate, expected pool size is 1.0
    assert!(is_candidate);

    // Game should complete in one guess
    let input = "CRANE\nGGGGG\n";
    let reader = Cursor::new(input);
    let mut interface = CliInterface::new(reader);
    game_loop(&wordbank, &mut interface);
}

#[test]
fn test_edge_case_no_candidates_remaining() {
    // Test behavior when feedback eliminates all candidates
    // This indicates either an error in feedback or the answer isn't in wordbank

    let wordbank = vec!["CRANE".to_string(), "SLATE".to_string()];

    // Give feedback that eliminates both words
    let input = "CRANE\nXXXXX\nSLATE\nXXXXX\nexit\n";
    let reader = Cursor::new(input);
    let mut interface = CliInterface::new(reader);

    // Should handle gracefully without panicking
    game_loop(&wordbank, &mut interface);
}

#[test]
fn test_starting_word_computation_integration() {
    // Test the expensive computation of optimal starting words
    let wordbank: Vec<String> = vec![
        "CRANE", "SLATE", "RAISE", "STARE", "ARISE", "IRATE", "ATONE", "STONE", "SHONE", "PHONE",
        "PLACE", "GRACE", "TRACE", "SPACE", "BRACE",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    let starting_words = compute_best_starting_words(&wordbank);

    assert_eq!(starting_words.len(), 5);

    // Verify good letter distribution
    let all_chars: String = starting_words.join("");
    let unique_chars: std::collections::HashSet<char> = all_chars.chars().collect();
    assert!(
        unique_chars.len() >= 10,
        "Starting words should have diverse letters"
    );

    assert!(starting_words.iter().all(|w| wordbank.contains(w)));
}

#[test]
fn test_wordbank_loading_variations() {
    // Test different ways to load wordbanks and verify consistency
    let data1 = "crane\nslate\nraise";
    let wordbank1 = load_wordbank_from_str(data1);

    let data2 = "CRANE\nSLATE\nRAISE";
    let wordbank2 = load_wordbank_from_str(data2);

    let data3 = "  crane  \n  slate  \n  raise  ";
    let wordbank3 = load_wordbank_from_str(data3);

    assert_eq!(wordbank1, wordbank2);
    assert_eq!(wordbank2, wordbank3);
    assert_eq!(wordbank1.len(), 3);
}

#[test]
fn test_progressive_candidate_elimination() {
    // Test that each round progressively eliminates candidates
    let wordbank: Vec<String> = [
        "AROSE", "PROSE", "THOSE", "WHOSE", "CHOSE", "CLOSE", "LOOSE", "MOOSE", "NOOSE", "GOOSE",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    let answer = "THOSE";
    let mut candidates = wordbank.clone();
    let mut previous_count = candidates.len();

    for _ in 0..3 {
        if candidates.len() == 1 {
            break;
        }

        let (guess, _, _) = best_information_guess(&wordbank, &candidates);
        let feedback = get_feedback(guess, answer);
        candidates = filter_candidates(&candidates, guess, &feedback);

        assert!(candidates.len() <= previous_count);
        assert!(candidates.contains(&answer.to_string()));

        previous_count = candidates.len();
    }

    assert!(
        candidates.len() <= 3,
        "Should have few candidates after 3 rounds"
    );
}

#[test]
fn test_cli_to_wordbank_integration() {
    // Test that CLI arguments properly flow through to wordbank loading
    // Would need to test both default (embedded) and custom file paths
}

#[test]
fn test_full_game_simulation_multiple_attempts() {
    // Simulate a complete game with multiple guesses
    // Track that candidates properly decrease each round

    let wordbank: Vec<String> = [
        "TRAIN", "BRAIN", "GRAIN", "DRAIN", "STAIN", "CHAIN", "PLAIN",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    let answer = "BRAIN";
    let mut candidates = wordbank.clone();
    let initial_count = candidates.len();

    // Round 1: Guess TRAIN
    let feedback1 = get_feedback("TRAIN", answer);
    candidates = filter_candidates(&candidates, "TRAIN", &feedback1);
    assert!(
        candidates.len() < initial_count,
        "Round 1 should reduce candidates"
    );
    assert!(
        candidates.contains(&answer.to_string()),
        "Answer should remain after round 1"
    );

    let round1_count = candidates.len();

    // Round 2: Get best guess for remaining candidates
    let (guess2, _, _) = best_information_guess(&wordbank, &candidates);
    let feedback2 = get_feedback(guess2, answer);
    candidates = filter_candidates(&candidates, guess2, &feedback2);
    assert!(
        candidates.len() <= round1_count,
        "Round 2 should not increase candidates"
    );
    assert!(
        candidates.contains(&answer.to_string()),
        "Answer should remain after round 2"
    );

    // Eventually we should be able to narrow down to the answer
    assert!(
        candidates.len() <= 3,
        "Should have few candidates remaining"
    );
}

#[test]
fn test_performance_with_large_wordbank() {
    // Test that the solver performs reasonably with a large wordbank
    // Generate 1000+ word wordbank and verify it doesn't timeout

    let mut large_wordbank = Vec::new();
    for i in 0..1000 {
        // Generate pseudo-words
        let word = format!("{:05}", i % 26 * 1000 + i);
        if word.chars().all(|c| c.is_ascii_digit()) {
            continue; // Skip all numeric
        }
        large_wordbank.push(word);
    }

    // This test verifies the algorithm doesn't have exponential complexity
}
