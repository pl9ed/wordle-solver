// Library interface for wordle-solver
// This allows integration tests to access internal modules

pub mod cli;
pub mod game_state;
pub mod logging;
pub mod solver;
pub mod wordbank;

// Re-export commonly used functions for easier testing
pub use game_state::game_loop;
pub use solver::{
    Feedback, best_information_guess, compute_best_starting_words, filter_candidates, get_feedback,
};
pub use wordbank::{load_wordbank_from_file, load_wordbank_from_str};
