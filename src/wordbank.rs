use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

pub const EMBEDDED_WORDBANK: &str = include_str!("resources/wordbank.txt");

fn is_valid_word(word: &str) -> bool {
    word.len() == 5 && word.chars().all(|c| c.is_ascii_alphabetic())
}

pub fn load_wordbank(wordbank_path: Option<String>) -> Vec<String> {
    if let Some(path) = wordbank_path {
        match load_wordbank_from_file(&path) {
            Ok(words) => {
                println!("Loaded {} words.", words.len());
                words
            }
            Err(e) => {
                eprintln!("Failed to load word bank from '{path}': {e}");
                std::process::exit(1);
            }
        }
    } else {
        let words = load_wordbank_from_str(EMBEDDED_WORDBANK);
        println!("Loaded {} words.", words.len());
        words
    }
}

pub fn load_wordbank_from_str(data: &str) -> Vec<String> {
    data.lines()
        .map(|line| line.trim().to_uppercase())
        .filter(|word| is_valid_word(word))
        .collect()
}

pub fn load_wordbank_from_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut words = Vec::new();
    for line in reader.lines() {
        let word = line?.trim().to_uppercase();
        if is_valid_word(&word) {
            words.push(word);
        }
    }
    Ok(words)
}

pub fn get_wordle_start_path() -> Option<PathBuf> {
    dirs::home_dir().map(|mut path| {
        path.push(".wordle_start");
        path
    })
}

pub fn read_starting_words(path: &Path) -> Option<Vec<String>> {
    if let Ok(file) = File::open(path) {
        let reader = BufReader::new(file);
        let words: Vec<String> = reader.lines()
            .map_while(Result::ok)
            .map(|w| w.trim().to_uppercase())
            .filter(|w| is_valid_word(w))
            .take(5)
            .collect();
        if words.len() == 5 {
            return Some(words);
        }
    }
    None
}

pub fn write_starting_words(path: &Path, words: &[String]) {
    if let Ok(mut file) = OpenOptions::new().create(true).write(true).truncate(true).open(path) {
        for word in words.iter().take(5) {
            let _ = writeln!(file, "{word}");
        }
    }
}
