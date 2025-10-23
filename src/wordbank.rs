use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

pub const EMBEDDED_WORDBANK: &str = include_str!("resources/wordbank.txt");

pub fn load_wordbank_from_str(data: &str) -> Vec<String> {
    data.lines()
        .map(|line| line.trim().to_uppercase())
        .filter(|word| word.len() == 5 && word.chars().all(|c| c.is_ascii_alphabetic()))
        .collect()
}

pub fn load_wordbank_from_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<String>> {
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
