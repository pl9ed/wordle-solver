use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

pub const EMBEDDED_WORDBANK: &str = include_str!("resources/wordbank.txt");

fn is_valid_word(word: &str) -> bool {
    word.len() == 5 && word.chars().all(|c| c.is_ascii_alphabetic())
}

#[must_use]
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

#[must_use]
pub fn load_wordbank_from_str(data: &str) -> Vec<String> {
    data.lines()
        .map(|line| line.trim().to_uppercase())
        .filter(|word| is_valid_word(word))
        .collect()
}

/// # Errors
/// Returns an error if the file cannot be read or accessed.
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

#[must_use]
pub fn get_wordle_start_path() -> Option<PathBuf> {
    dirs::home_dir().map(|mut path| {
        path.push(".wordle_start");
        path
    })
}

pub fn read_starting_words(path: &Path) -> Option<Vec<String>> {
    if let Ok(file) = File::open(path) {
        let reader = BufReader::new(file);
        let words: Vec<String> = reader
            .lines()
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
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
    {
        for word in words.iter().take(5) {
            let _ = writeln!(file, "{word}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_wordbank_from_str_valid() {
        let data = "crane\nslate\nraise\nstare\narise";
        let words = load_wordbank_from_str(data);

        assert_eq!(words.len(), 5);
        assert_eq!(words, vec!["CRANE", "SLATE", "RAISE", "STARE", "ARISE"]);
    }

    #[test]
    fn test_load_wordbank_from_str_filters_invalid() {
        let data = "crane\nslate\ntoo\ntoolong\n12345\nraise";
        let words = load_wordbank_from_str(data);

        // Should only include valid 5-letter words
        assert_eq!(words.len(), 3);
        assert_eq!(words, vec!["CRANE", "SLATE", "RAISE"]);
    }

    #[test]
    fn test_load_wordbank_from_str_trims_whitespace() {
        let data = "  crane  \n slate\t\n\nraise  ";
        let words = load_wordbank_from_str(data);

        assert_eq!(words.len(), 3);
        assert_eq!(words, vec!["CRANE", "SLATE", "RAISE"]);
    }

    #[test]
    fn test_load_wordbank_from_str_uppercase_conversion() {
        let data = "crane\nSlAtE\nRAISE\nmixed";
        let words = load_wordbank_from_str(data);

        assert_eq!(words.len(), 4);
        assert!(words.iter().all(|w| w.chars().all(|c| c.is_uppercase())));
    }

    #[test]
    fn test_load_wordbank_from_str_empty() {
        let data = "";
        let words = load_wordbank_from_str(data);

        assert_eq!(words.len(), 0);
    }

    #[test]
    fn test_load_wordbank_from_str_filters_non_alphabetic() {
        let data = "crane\nsl@te\nra1se\nstare";
        let words = load_wordbank_from_str(data);

        // Should filter out words with non-alphabetic characters
        assert_eq!(words.len(), 2);
        assert_eq!(words, vec!["CRANE", "STARE"]);
    }

    #[test]
    fn test_load_wordbank_from_file_valid() {
        // Create a temporary file
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_wordbank.txt");

        {
            let mut file = File::create(&file_path).unwrap();
            writeln!(file, "crane").unwrap();
            writeln!(file, "slate").unwrap();
            writeln!(file, "raise").unwrap();
        }

        let words = load_wordbank_from_file(&file_path).unwrap();

        assert_eq!(words.len(), 3);
        assert_eq!(words, vec!["CRANE", "SLATE", "RAISE"]);

        // Cleanup
        std::fs::remove_file(&file_path).unwrap();
    }

    #[test]
    fn test_load_wordbank_from_file_nonexistent() {
        let result = load_wordbank_from_file("nonexistent_file.txt");

        assert!(result.is_err());
    }

    #[test]
    fn test_load_wordbank_from_file_filters_invalid() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_wordbank_invalid.txt");

        {
            let mut file = File::create(&file_path).unwrap();
            writeln!(file, "crane").unwrap();
            writeln!(file, "ab").unwrap();
            writeln!(file, "toolong").unwrap();
            writeln!(file, "slate").unwrap();
        }

        let words = load_wordbank_from_file(&file_path).unwrap();

        assert_eq!(words.len(), 2);
        assert_eq!(words, vec!["CRANE", "SLATE"]);

        std::fs::remove_file(&file_path).unwrap();
    }

    #[test]
    fn test_read_starting_words_valid() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_wordle_start.txt");

        {
            let mut file = File::create(&file_path).unwrap();
            writeln!(file, "crane").unwrap();
            writeln!(file, "slate").unwrap();
            writeln!(file, "raise").unwrap();
            writeln!(file, "stare").unwrap();
            writeln!(file, "arise").unwrap();
        }

        let words = read_starting_words(&file_path);

        assert!(words.is_some());
        let words = words.unwrap();
        assert_eq!(words.len(), 5);
        assert_eq!(words, vec!["CRANE", "SLATE", "RAISE", "STARE", "ARISE"]);

        std::fs::remove_file(&file_path).unwrap();
    }

    #[test]
    fn test_read_starting_words_insufficient() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_wordle_start_short.txt");

        {
            let mut file = File::create(&file_path).unwrap();
            writeln!(file, "crane").unwrap();
            writeln!(file, "slate").unwrap();
        }

        let words = read_starting_words(&file_path);

        // Should return None if less than 5 words
        assert!(words.is_none());

        std::fs::remove_file(&file_path).unwrap();
    }

    #[test]
    fn test_read_starting_words_nonexistent() {
        let file_path = PathBuf::from("nonexistent_start_file.txt");
        let words = read_starting_words(&file_path);

        assert!(words.is_none());
    }

    #[test]
    fn test_read_starting_words_takes_only_five() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_wordle_start_long.txt");

        {
            let mut file = File::create(&file_path).unwrap();
            writeln!(file, "crane").unwrap();
            writeln!(file, "slate").unwrap();
            writeln!(file, "raise").unwrap();
            writeln!(file, "stare").unwrap();
            writeln!(file, "arise").unwrap();
            writeln!(file, "irate").unwrap();
            writeln!(file, "atone").unwrap();
        }

        let words = read_starting_words(&file_path);

        assert!(words.is_some());
        let words = words.unwrap();
        assert_eq!(words.len(), 5);
        // Should only take first 5
        assert_eq!(words, vec!["CRANE", "SLATE", "RAISE", "STARE", "ARISE"]);

        std::fs::remove_file(&file_path).unwrap();
    }

    #[test]
    fn test_write_starting_words() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_write_start.txt");

        let words = vec![
            "CRANE".to_string(),
            "SLATE".to_string(),
            "RAISE".to_string(),
            "STARE".to_string(),
            "ARISE".to_string(),
        ];

        write_starting_words(&file_path, &words);

        // Verify the file was written correctly
        let content = std::fs::read_to_string(&file_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        assert_eq!(lines.len(), 5);
        assert_eq!(lines, vec!["CRANE", "SLATE", "RAISE", "STARE", "ARISE"]);

        std::fs::remove_file(&file_path).unwrap();
    }

    #[test]
    fn test_write_starting_words_more_than_five() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_write_start_long.txt");

        let words = vec![
            "CRANE".to_string(),
            "SLATE".to_string(),
            "RAISE".to_string(),
            "STARE".to_string(),
            "ARISE".to_string(),
            "IRATE".to_string(),
            "ATONE".to_string(),
        ];

        write_starting_words(&file_path, &words);

        // Should only write first 5
        let content = std::fs::read_to_string(&file_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        assert_eq!(lines.len(), 5);

        std::fs::remove_file(&file_path).unwrap();
    }

    #[test]
    fn test_write_then_read_starting_words_roundtrip() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_roundtrip.txt");

        let original_words = vec![
            "CRANE".to_string(),
            "SLATE".to_string(),
            "RAISE".to_string(),
            "STARE".to_string(),
            "ARISE".to_string(),
        ];

        write_starting_words(&file_path, &original_words);
        let read_words = read_starting_words(&file_path).unwrap();

        assert_eq!(original_words, read_words);

        std::fs::remove_file(&file_path).unwrap();
    }

    #[test]
    fn test_get_wordle_start_path() {
        let path = get_wordle_start_path();

        // Should return Some path
        assert!(path.is_some());

        if let Some(path) = path {
            // Should end with .wordle_start
            assert!(path.to_string_lossy().ends_with(".wordle_start"));
        }
    }

    #[test]
    fn test_embedded_wordbank_not_empty() {
        assert!(!EMBEDDED_WORDBANK.is_empty());

        // Test that embedded wordbank can be loaded
        let words = load_wordbank_from_str(EMBEDDED_WORDBANK);
        assert!(words.len() > 0);

        // All words should be 5 letters and uppercase
        assert!(words.iter().all(|w| w.len() == 5));
        assert!(words.iter().all(|w| w.chars().all(|c| c.is_uppercase())));
    }
}
