mod cli;
mod game_state;
mod solver;
mod tui;
mod wordbank;

use cli::{CliInterface, UiMode, parse_cli};
use game_state::game_loop;
use std::io;
use tui::TuiWrapper;
use wordbank::load_wordbank;

// Conditional logging macros - only active in debug builds
#[cfg(debug_assertions)]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        log::debug!($($arg)*);
    };
}

#[cfg(not(debug_assertions))]
macro_rules! debug_log {
    ($($arg:tt)*) => {{}};
}

#[cfg(debug_assertions)]
macro_rules! info_log {
    ($($arg:tt)*) => {
        log::info!($($arg)*);
    };
}

#[cfg(not(debug_assertions))]
macro_rules! info_log {
    ($($arg:tt)*) => {{}};
}

fn main() {
    // Initialize logger only in debug builds
    #[cfg(debug_assertions)]
    {
        use std::io::Write;

        let log_file = std::fs::File::create("output.txt").expect("Failed to create log file");
        let log_file = std::io::LineWriter::new(log_file); // Use LineWriter for immediate flushing

        env_logger::Builder::from_default_env()
            .target(env_logger::Target::Pipe(Box::new(log_file)))
            .filter_level(log::LevelFilter::Debug)
            .format(|buf, record| {
                writeln!(
                    buf,
                    "[{} {} {}:{}] {}",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                    record.level(),
                    record.file().unwrap_or("unknown"),
                    record.line().unwrap_or(0),
                    record.args()
                )
            })
            .init();

        info_log!("Application started");
    }

    let cli = parse_cli();
    info_log!(
        "CLI parsed: ui_mode={:?}, wordbank_path={:?}",
        cli.ui_mode,
        cli.wordbank_path
    );

    match cli.ui_mode {
        UiMode::Cli => {
            info_log!("Starting CLI mode");
            // Use CLI mode
            app_cli(cli.wordbank_path);
        }
        UiMode::Tui => {
            info_log!("Starting TUI mode");
            // Use TUI mode (default)
            let wordbank_path = cli.wordbank_path;
            if let Err(e) = app_tui(wordbank_path.clone()) {
                eprintln!("TUI Error: {e}. Falling back to CLI mode.");
                info_log!("TUI failed with error: {}, falling back to CLI", e);
                app_cli(wordbank_path);
            }
        }
    }

    info_log!("Application exiting");
}

fn app_cli(wordbank_path: Option<String>) {
    let initial_wordbank = load_wordbank(wordbank_path);
    info_log!("Loaded {} words for CLI", initial_wordbank.len());
    let stdin = io::stdin();
    let mut interface = CliInterface::new(stdin.lock());
    game_loop(&initial_wordbank, &mut interface);
}

fn app_tui(wordbank_path: Option<String>) -> Result<(), io::Error> {
    let initial_wordbank = load_wordbank(wordbank_path);
    info_log!("Loaded {} words for TUI", initial_wordbank.len());
    let mut interface = TuiWrapper::new()?;
    info_log!("TUI interface initialized");
    game_loop(&initial_wordbank, &mut interface);
    Ok(())
}
