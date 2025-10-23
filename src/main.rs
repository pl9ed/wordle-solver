mod cli;
mod game_state;
mod solver;
mod wordbank;

use cli::parse_cli;
use game_state::game_loop;
use std::io;
use wordbank::load_wordbank;

fn main() {
    let cli = parse_cli();
    app(cli.wordbank_path);
}

fn app(wordbank_path: Option<String>) {
    let initial_wordbank = load_wordbank(wordbank_path);
    let stdin = io::stdin();
    game_loop(&initial_wordbank, stdin.lock());
}
