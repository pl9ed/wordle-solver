# Wordle Solver

Vibe coded wordle solver to experiment with GenAI tools.

[example.webm](https://github.com/user-attachments/assets/97236fb0-84c0-4354-a2b7-578247ea5d67)

## Table of Contents

- [Features](#features)
- [Installation](#installation)
  - [Prerequisites](#prerequisites)
  - [Build from Source](#build-from-source)
- [Usage](#usage)
  - [Basic Usage](#basic-usage)
  - [Custom Wordbank](#custom-wordbank)
  - [Interactive Gameplay](#interactive-gameplay)
  - [Commands](#commands)
- [Example Session](#example-session)
- [How It Works](#how-it-works)
  - [Algorithm](#algorithm)
  - [Starting Word Computation](#starting-word-computation)
- [Project Structure](#project-structure)

## Features

- **Optimal Starting Words**: Automatically computes and caches the best starting words based on expected information gain
- **Smart Recommendations**: Suggests the next best guess to minimize the remaining candidate pool
- **Interactive Gameplay**: Step-by-step guidance through the solving process
- **Custom Word Banks**: Support for custom word lists or use the embedded default wordbank
- **Fast Computation**: Efficient algorithms for analyzing thousands of word combinations
- **Persistent Cache**: Saves computed starting words to `~/.wordle_start` for faster subsequent runs

## Installation

### Prerequisites

- Rust 1.70 or later (uses Rust 2024 edition)

### Build from Source

```bash
git clone <repository-url>
cd wordle-solver
cargo build --release
```

The compiled binary will be available at `target/release/wordle-solver.exe` (Windows) or `target/release/wordle-solver` (Unix).

## Usage

### Basic Usage

Run the solver with the default embedded wordbank:

```bash
cargo run --release
```

Or use the compiled binary:

```bash
./target/release/wordle-solver
```

### Custom Wordbank

Use your own word list (newline-delimited, 5-letter words):

```bash
cargo run --release -- --input path/to/wordbank.txt
```

Or:

```bash
cargo run --release -- -i path/to/wordbank.txt
```

### Interactive Gameplay

1. **Start the Game**: The solver displays optimal starting words and suggests the best first guess.

2. **Enter Your Guess**: Type the 5-letter word you guessed in the actual Wordle game.

3. **Provide Feedback**: Enter the feedback from Wordle using:
   - `G` = Green (correct letter in correct position)
   - `Y` = Yellow (correct letter in wrong position)
   - `X` = Gray (letter not in the word)
   
   Example: `GYXXG` means positions 1 and 5 are green, 2 is yellow, and 3-4 are gray.

4. **Follow Recommendations**: The solver will:
   - Filter remaining candidates
   - Display up to 5 possible words
   - Compute and suggest the next optimal guess
   - Indicate whether the guess is a "solution candidate" or an "information-gathering" word

5. **Repeat**: Continue until the solution is found or no candidates remain.

### Commands

During gameplay, you can enter:
- Any 5-letter word as your guess
- `exit` - Quit the application
- `next` - Start a new game

## Example Session

```
Optimal starting words:
1. SALET
2. RALES
3. SOARE
4. AROSE
5. RAISE
(Computed and cached to: C:\Users\username\.wordle_start.)
Suggested starting word: SALET

Enter your guess (5 letters, or 'exit' to quit, or 'next' to start a new game):
SALET
Enter feedback (G=green, Y=yellow, X=gray, e.g. GYXXG):
XYGXX
Possible candidates (120)
RATIO
RADIO
RAPID
RAINY
RAINS
Computing optimal guess, please wait...
Recommended guess: ROUND (expected pool size 8.43) [information-gathering]

Enter your guess (5 letters, or 'exit' to quit, or 'next' to start a new game):
ROUND
Enter feedback (G=green, Y=yellow, X=gray, e.g. GYXXG):
YXXXX
Possible candidates (4)
FAIRY
HAIRY
CHAIR
FLAIR
Computing optimal guess, please wait...
Recommended guess: FAIRY (expected pool size 1.25) [solution candidate]

Enter your guess (5 letters, or 'exit' to quit, or 'next' to start a new game):
FAIRY
Enter feedback (G=green, Y=yellow, X=gray, e.g. GYXXG):
GGGGG
Possible candidates (1)
FAIRY
Solution found: FAIRY
```

## How It Works

### Algorithm

The solver uses an **information-theoretic approach** to minimize the expected size of the remaining candidate pool:

1. **Feedback Patterns**: For each potential guess, simulate all possible feedback patterns against remaining candidates.

2. **Expected Pool Size**: Calculate the weighted average of remaining candidates for each feedback pattern.

3. **Optimal Selection**: Choose the guess that minimizes the expected pool size.

4. **Filtering**: Apply the actual feedback received to filter candidates based on:
   - Green matches (correct position)
   - Yellow matches (wrong position but letter exists)
   - Gray matches (letter not in word, or no more instances)

### Starting Word Computation

On first run, the solver computes the 5 best starting words by evaluating every word in the wordbank against all possible solutions. This takes time initially but is cached to `~/.wordle_start` for instant loading in future sessions.

## Project Structure

```
wordle-solver/
├── src/
│   ├── main.rs          # Binary entry point
│   ├── lib.rs           # Library interface for testing
│   ├── cli.rs           # Command-line argument parsing (with unit tests)
│   ├── game_state.rs    # Game loop and user interaction (with unit tests)
│   ├── solver.rs        # Core solving algorithms (with unit tests)
│   ├── wordbank.rs      # Word list loading and caching (with unit tests)
│   └── resources/
│       └── wordbank.txt # Embedded default word list
├── tests/
│   └── integration_tests.rs  # Integration tests
├── Cargo.toml           # Project configuration
├── LICENSE              # License file
└── README.md            # This file
```
