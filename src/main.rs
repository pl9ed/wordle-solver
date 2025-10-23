mod cli;
mod wordbank;
mod solver;
mod game_state;

use cli::parse_cli;
use wordbank::load_wordbank;
use game_state::game_loop;
use std::io;

fn main() {
    let cli = parse_cli();
    app(cli.wordbank_path);
}

fn app(wordbank_path: Option<String>) {
    let initial_wordbank = load_wordbank(wordbank_path);
    let stdin = io::stdin();
    game_loop(&initial_wordbank, stdin.lock());
}


