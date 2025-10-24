//! TUI (Terminal User Interface) module for Wordle Solver
//!
//! This module provides an interactive terminal interface using Ratatui.
//!
//! # Architecture
//! - `TuiInterface`: Core UI component handling rendering and input
//! - `TuiWrapper`: Wrapper that integrates with game loop
//!
//! # State Machine
//! The UI follows these state transitions:
//! - `EnteringGuess` → `MarkingFeedback` → `ConfirmingFeedback` → `WaitingForNext` → back to `EnteringGuess`
//! - Terminal states: `Computing`, `GameOver`

use crate::game_state::{GameInterface, Recommendation, StartingWordsInfo, UserAction};
use crate::solver::Feedback;
use crate::{debug_log, info_log};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use std::io;

const MAX_GUESSES: usize = 6;
const WORD_LENGTH: usize = 5;
const MAX_CANDIDATES_DISPLAY: usize = 10;
const EVENT_POLL_TIMEOUT_MS: u64 = 100;
const COMPUTING_POLL_TIMEOUT_MS: u64 = 10;
const ROW_SPACING: u16 = 2;
const ASCII_CONTROL_CHAR_THRESHOLD: u32 = 32;

// Style constants for consistent UI
const HEADER_STYLE: Style = Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD);
const ERROR_STYLE: Style = Style::new().fg(Color::Red);
const SUCCESS_STYLE: Style = Style::new().fg(Color::Green).add_modifier(Modifier::BOLD);
const INFO_STYLE: Style = Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD);
const MESSAGE_STYLE: Style = Style::new().fg(Color::Cyan);

#[derive(Clone, Copy, PartialEq, Debug)]
enum LetterState {
    Empty,
    Entered,
    Match,        // Green
    PartialMatch, // Yellow
    NoMatch,      // Gray
}

#[derive(Debug)]
struct GuessRow {
    letters: [char; 5],
    states: [LetterState; 5],
}

impl GuessRow {
    fn new() -> Self {
        Self {
            letters: [' '; WORD_LENGTH],
            states: [LetterState::Empty; WORD_LENGTH],
        }
    }

    fn from_guess(guess: &str) -> Self {
        let mut row = Self::new();
        for (i, ch) in guess.chars().enumerate().take(WORD_LENGTH) {
            row.letters[i] = ch;
            row.states[i] = LetterState::Entered;
        }
        row
    }
}

impl LetterState {
    fn colors(self) -> (Color, Color) {
        match self {
            Self::Empty | Self::Entered => (Color::DarkGray, Color::White),
            Self::Match => (Color::Green, Color::Black),
            Self::PartialMatch => (Color::Yellow, Color::Black),
            Self::NoMatch => (Color::Gray, Color::White),
        }
    }

    fn to_feedback(self) -> Feedback {
        match self {
            Self::Match => Feedback::Match,
            Self::PartialMatch => Feedback::PartialMatch,
            Self::NoMatch | Self::Empty | Self::Entered => Feedback::NoMatch,
        }
    }
}

#[derive(Debug)]
enum TuiState {
    EnteringGuess,
    MarkingFeedback {
        marking_index: usize,
    },
    ConfirmingFeedback,
    Computing,
    WaitingForNext,
    /// Game has ended (solution found or no candidates) - message stored in interface.message
    GameOver,
}

/// Context for rendering the UI - groups related parameters to avoid too many function arguments.
struct RenderContext<'a> {
    guesses: &'a [GuessRow],
    current_input: &'a str,
    state: &'a TuiState,
    candidates_display: &'a [String],
    recommendation: Option<&'a Recommendation>,
    starting_words: &'a [String],
    message: &'a str,
    error_message: &'a str,
    status: &'a str,
}

/// Main TUI interface component.
///
/// Manages terminal rendering, input handling, and game state display.
pub struct TuiInterface {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    guesses: Vec<GuessRow>,
    current_input: String,
    state: TuiState,
    candidates_display: Vec<String>,
    recommendation: Option<Recommendation>,
    starting_words: Vec<String>,
    message: String,
    error_message: String,
    status: String,
}

impl TuiInterface {
    pub fn new() -> Result<Self, io::Error> {
        info_log!("TuiInterface::new() - Initializing TUI");
        enable_raw_mode()?;
        info_log!("Raw mode enabled");
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
        info_log!("Terminal setup complete: alternate screen, mouse capture, cursor hidden");
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        info_log!("Terminal backend created");

        Ok(Self {
            terminal,
            guesses: Vec::new(),
            current_input: String::new(),
            state: TuiState::EnteringGuess,
            candidates_display: Vec::new(),
            recommendation: None,
            starting_words: Vec::new(),
            message: String::new(),
            error_message: String::new(),
            status: "Ready to start".to_string(),
        })
    }

    pub fn cleanup(&mut self) -> Result<(), io::Error> {
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            cursor::Show
        )?;
        Ok(())
    }

    /// Draw the current UI state to the terminal.
    ///
    /// Returns an error if rendering fails.
    fn draw(&mut self) -> Result<(), io::Error> {
        let ctx = RenderContext {
            guesses: &self.guesses,
            current_input: &self.current_input,
            state: &self.state,
            candidates_display: &self.candidates_display,
            recommendation: self.recommendation.as_ref(),
            starting_words: &self.starting_words,
            message: &self.message,
            error_message: &self.error_message,
            status: &self.status,
        };

        self.terminal.draw(|f| {
            Self::render_static(f, &ctx);
        })?;
        Ok(())
    }

    /// Helper method to check if current input should be displayed
    fn should_show_current_input(&self) -> bool {
        matches!(self.state, TuiState::EnteringGuess) && self.guesses.len() < MAX_GUESSES
    }

    /// Log and handle draw errors appropriately
    fn draw_or_log(&mut self) {
        if let Err(e) = self.draw() {
            debug_log!("Draw error: {}", e);
        }
    }

    /// Render the complete UI layout using the provided context.
    fn render_static(f: &mut Frame, ctx: &RenderContext) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Length(14), // Game board (more compact)
                Constraint::Min(8),     // Info panel (takes remaining space)
                Constraint::Length(3),  // Status line
                Constraint::Length(3),  // Instructions
            ])
            .split(f.area());

        Self::render_title(f, chunks[0]);
        Self::render_board(f, chunks[1], ctx.guesses, ctx.current_input, ctx.state);
        Self::render_info(
            f,
            chunks[2],
            ctx.candidates_display,
            ctx.recommendation,
            ctx.starting_words,
            ctx.message,
            ctx.error_message,
        );
        Self::render_status(f, chunks[3], ctx.status);
        Self::render_instructions(f, chunks[4], ctx.state);
    }

    fn render_title(f: &mut Frame, area: Rect) {
        let title = Paragraph::new("WORDLE SOLVER")
            .style(HEADER_STYLE)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, area);
    }

    fn render_board(
        f: &mut Frame,
        area: Rect,
        guesses: &[GuessRow],
        current_input: &str,
        state: &TuiState,
    ) {
        let block = Block::default()
            .title("Guesses")
            .borders(Borders::ALL)
            .style(Style::default());

        let inner = block.inner(area);
        f.render_widget(block, area);

        // Calculate how many rows can fit in the available space
        let available_rows = (inner.height / ROW_SPACING) as usize;

        // Determine if we need to show current input
        let showing_current_input =
            matches!(state, TuiState::EnteringGuess) && guesses.len() < MAX_GUESSES;
        let rows_needed = if showing_current_input {
            guesses.len() + 1
        } else {
            guesses.len()
        };

        // Calculate which guesses to show (prioritize most recent)
        let skip_count = rows_needed.saturating_sub(available_rows);

        // Render visible guesses (skip oldest ones if needed)
        // Fixed: Remove confusing double enumerate - display_index is now clear
        for (display_index, guess) in guesses.iter().skip(skip_count).enumerate() {
            Self::render_guess_row(
                f,
                guess,
                display_index,
                inner,
                state,
                guesses.len() - skip_count,
            );
        }

        // Render current input if entering a guess
        if showing_current_input {
            let display_row = if rows_needed > available_rows {
                available_rows - 1
            } else {
                guesses.len() - skip_count
            };
            Self::render_current_input(f, display_row, inner, current_input);
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn render_guess_row(
        f: &mut Frame,
        guess: &GuessRow,
        row_index: usize,
        area: Rect,
        state: &TuiState,
        guesses_len: usize,
    ) {
        let y = area.y + (row_index as u16 * ROW_SPACING);
        if y >= area.y + area.height {
            return;
        }

        let mut spans = vec![Span::raw("  ")];
        for i in 0..WORD_LENGTH {
            let (bg_color, fg_color) = guess.states[i].colors();
            let letter = guess.letters[i];

            spans.push(Span::styled(
                format!(" {letter} "),
                Style::default().fg(fg_color).bg(bg_color),
            ));
            spans.push(Span::raw(" "));
        }

        // Highlight the letter being marked
        if let TuiState::MarkingFeedback { marking_index } = state
            && row_index == guesses_len - 1
        {
            spans.push(Span::raw(format!(
                " <- Marking letter {} (G/Y/X)",
                marking_index + 1
            )));
        }

        Self::render_line(f, area, y, spans);
    }

    fn render_line(f: &mut Frame, area: Rect, y: u16, spans: Vec<Span>) {
        let line = Line::from(spans);
        let paragraph = Paragraph::new(line);
        f.render_widget(
            paragraph,
            Rect {
                x: area.x,
                y,
                width: area.width,
                height: 1,
            },
        );
    }

    #[allow(clippy::cast_possible_truncation)]
    fn render_current_input(f: &mut Frame, row_index: usize, area: Rect, current_input: &str) {
        let y = area.y + (row_index as u16 * ROW_SPACING);
        if y >= area.y + area.height {
            return;
        }

        let mut spans = vec![Span::raw("  ")];
        for i in 0..WORD_LENGTH {
            let letter = current_input.chars().nth(i).unwrap_or(' ');
            spans.push(Span::styled(
                format!(" {letter} "),
                Style::default().fg(Color::White).bg(Color::DarkGray),
            ));
            spans.push(Span::raw(" "));
        }

        Self::render_line(f, area, y, spans);
    }

    fn render_info(
        f: &mut Frame,
        area: Rect,
        candidates_display: &[String],
        recommendation: Option<&Recommendation>,
        starting_words: &[String],
        message: &str,
        error_message: &str,
    ) {
        let mut lines = Vec::new();

        // Starting words
        if !starting_words.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "Suggested Starting Words:",
                HEADER_STYLE,
            )]));
            for (i, word) in starting_words.iter().take(3).enumerate() {
                let num = i + 1;
                lines.push(Line::from(format!("  {num}. {word}")));
            }
            lines.push(Line::from(""));
        }

        // Recommendation
        if let Some(rec) = recommendation {
            let category = if rec.is_candidate {
                "solution candidate"
            } else {
                "information-gathering"
            };
            lines.push(Line::from(vec![Span::styled(
                format!(
                    "Recommended: {} (score: {:.2}) [{}]",
                    rec.guess, rec.score, category
                ),
                SUCCESS_STYLE,
            )]));
            lines.push(Line::from(""));
        }

        // Candidates
        if !candidates_display.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                format!("Possible candidates ({}):", candidates_display.len()),
                INFO_STYLE,
            )]));
            for word in candidates_display.iter().take(MAX_CANDIDATES_DISPLAY) {
                lines.push(Line::from(format!("  {word}")));
            }
            if candidates_display.len() > MAX_CANDIDATES_DISPLAY {
                lines.push(Line::from(format!(
                    "  ... and {} more",
                    candidates_display.len() - MAX_CANDIDATES_DISPLAY
                )));
            }
            lines.push(Line::from(""));
        }

        // Messages
        if !message.is_empty() {
            lines.push(Line::from(vec![Span::styled(message, MESSAGE_STYLE)]));
        }

        // Error messages
        if !error_message.is_empty() {
            lines.push(Line::from(vec![Span::styled(error_message, ERROR_STYLE)]));
        }

        let paragraph = Paragraph::new(lines)
            .block(Block::default().title("Information").borders(Borders::ALL))
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);
    }

    fn render_instructions(f: &mut Frame, area: Rect, state: &TuiState) {
        let text = match state {
            TuiState::EnteringGuess => "Type your 5-letter guess | ENTER: Submit | ESC: Quit",
            TuiState::MarkingFeedback { .. } => {
                "G: Green (correct) | Y: Yellow (wrong position) | X: Gray (not in word) | BACKSPACE: Go back"
            }
            TuiState::ConfirmingFeedback => "ENTER: Confirm feedback | BACKSPACE: Go back and edit",
            TuiState::Computing => "Computing optimal next guess...",
            TuiState::WaitingForNext => "Press any key to continue | ESC: Quit",
            TuiState::GameOver => "N: New Game | ESC: Quit",
        };

        let paragraph = Paragraph::new(text)
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(paragraph, area);
    }

    fn render_status(f: &mut Frame, area: Rect, status: &str) {
        let status_text = if status.is_empty() { "Ready" } else { status };
        let paragraph = Paragraph::new(status_text)
            .style(HEADER_STYLE)
            .block(Block::default().borders(Borders::ALL).title("Status"));
        f.render_widget(paragraph, area);
    }

    fn handle_input(&mut self) -> Result<Option<UserAction>, io::Error> {
        // For Computing state, use non-blocking poll to avoid hanging
        if matches!(self.state, TuiState::Computing) {
            debug_log!("handle_input() - In Computing state, using non-blocking poll");
            // Check if there's an event available without blocking
            if event::poll(std::time::Duration::from_millis(COMPUTING_POLL_TIMEOUT_MS))?
                && let Event::Key(_) = event::read()?
            {
                debug_log!("handle_input() - Ignoring key during Computing state");
                // Ignore any input during computing
            }
            return Ok(None);
        }

        // For all other states, use blocking read to ensure we only get one event

        // Poll with a timeout to check if events are available
        let poll_result = event::poll(std::time::Duration::from_millis(EVENT_POLL_TIMEOUT_MS))?;

        if !poll_result {
            // No event available, return None to continue the loop
            return Ok(None);
        }

        let event = event::read()?;
        debug_log!("handle_input() - Event received: {:?}", event);

        // Filter out non-key events (mouse, focus, etc.)
        match event {
            Event::Mouse(_) => {
                debug_log!("handle_input() - Ignoring mouse event");
                Ok(None)
            }
            Event::FocusGained | Event::FocusLost => {
                debug_log!("handle_input() - Ignoring focus event");
                Ok(None)
            }
            Event::Paste(_) => {
                debug_log!("handle_input() - Ignoring paste event");
                Ok(None)
            }
            Event::Resize(_, _) => {
                debug_log!("handle_input() - Ignoring resize event");
                Ok(None)
            }
            Event::Key(key) => {
                // Only process Press events, ignore Release and Repeat to avoid double input
                if key.kind != event::KeyEventKind::Press {
                    debug_log!(
                        "handle_input() - Ignoring non-Press key event: {:?}",
                        key.kind
                    );
                    return Ok(None);
                }

                // Filter out invalid characters that come from terminal focus events (alt-tab)
                // These show up as replacement characters (�), control characters, or other garbage
                if let KeyCode::Char(c) = key.code {
                    // Ignore replacement characters, control characters, and other non-printable chars
                    // that might come from escape sequences when alt-tabbing
                    if c == '\u{FFFD}'
                        || (c as u32) < ASCII_CONTROL_CHAR_THRESHOLD
                            && c != '\t'
                            && c != '\n'
                            && c != '\r'
                    {
                        debug_log!(
                            "handle_input() - Ignoring invalid character from escape sequence: {:?}",
                            c
                        );
                        return Ok(None);
                    }
                }

                debug_log!(
                    "handle_input() - Key event received: code={:?}, modifiers={:?}",
                    key.code,
                    key.modifiers
                );
                match &self.state {
                    TuiState::EnteringGuess => {
                        debug_log!("handle_input() - Processing in EnteringGuess state");
                        return Ok(self.handle_guess_input(key));
                    }
                    TuiState::MarkingFeedback { .. } => {
                        debug_log!("handle_input() - Processing in MarkingFeedback state");
                        self.handle_feedback_input(key);
                    }
                    TuiState::ConfirmingFeedback => {
                        debug_log!("handle_input() - Processing in ConfirmingFeedback state");
                        self.handle_confirming_feedback_input(key);
                    }
                    TuiState::WaitingForNext => {
                        debug_log!("handle_input() - Processing in WaitingForNext state");
                        return Ok(self.handle_waiting_input(key));
                    }
                    TuiState::GameOver => {
                        debug_log!("handle_input() - Processing in GameOver state");
                        return Ok(Self::handle_game_over_input(key));
                    }
                    TuiState::Computing => {}
                }
                Ok(None)
            }
        }
    }

    fn handle_guess_input(&mut self, key: KeyEvent) -> Option<UserAction> {
        self.error_message.clear();
        debug_log!(
            "handle_guess_input() - Processing key: {:?}, current_input: '{}'",
            key.code,
            self.current_input
        );

        match key.code {
            KeyCode::Char(c) if c.is_ascii_alphabetic() && self.current_input.len() < 5 => {
                // Ignore characters with Alt, Control, or other modifiers (Shift is ok for uppercase)
                let has_alt = key.modifiers.contains(event::KeyModifiers::ALT);
                let has_ctrl = key.modifiers.contains(event::KeyModifiers::CONTROL);
                if has_alt || has_ctrl {
                    debug_log!(
                        "handle_guess_input() - Ignoring character with modifier: {:?}",
                        key.modifiers
                    );
                } else {
                    self.current_input.push(c.to_ascii_uppercase());
                    info_log!(
                        "handle_guess_input() - Added '{}' to input, now: '{}'",
                        c.to_ascii_uppercase(),
                        self.current_input
                    );
                }
            }
            KeyCode::Backspace if !self.current_input.is_empty() => {
                self.current_input.pop();
                info_log!(
                    "handle_guess_input() - Removed character, now: '{}'",
                    self.current_input
                );
            }
            KeyCode::Enter if self.current_input.len() == 5 => {
                let guess = self.current_input.clone();
                self.current_input.clear();
                info_log!(
                    "handle_guess_input() - Enter pressed with valid guess: '{}', returning Guess action",
                    guess
                );
                return Some(UserAction::Guess(guess));
            }
            KeyCode::Enter => {
                self.error_message = "Guess must be exactly 5 letters!".to_string();
                info_log!(
                    "handle_guess_input() - Enter pressed but input length is {}, showing error",
                    self.current_input.len()
                );
            }
            KeyCode::Esc => {
                info_log!("handle_guess_input() - ESC pressed, returning Exit");
                return Some(UserAction::Exit);
            }
            KeyCode::Char(c) if !c.is_ascii_alphabetic() => {
                // Explicitly reject non-alphabetic characters
                self.error_message = format!("Only letters are allowed! ('{c}' is not a letter)");
                debug_log!(
                    "handle_guess_input() - Rejecting non-alphabetic character: '{}'",
                    c
                );
            }
            _ => {
                debug_log!("handle_guess_input() - Ignoring key: {:?}", key.code);
            }
        }
        None
    }

    fn handle_feedback_input(&mut self, key: KeyEvent) -> Option<UserAction> {
        if let TuiState::MarkingFeedback { marking_index } = self.state {
            // Ignore inputs with Alt or Control modifiers to prevent alt-tab issues
            if Self::has_modifier_keys(&key) {
                debug_log!(
                    "handle_feedback_input() - Ignoring input with modifier: {:?}",
                    key.modifiers
                );
                return None;
            }

            let last_guess = self.guesses.last_mut().unwrap();

            match key.code {
                KeyCode::Esc => {
                    info_log!("handle_feedback_input() - ESC pressed, returning Exit");
                    return Some(UserAction::Exit);
                }
                KeyCode::Char('g' | 'G') => {
                    last_guess.states[marking_index] = LetterState::Match;
                    self.advance_feedback_marking(marking_index);
                }
                KeyCode::Char('y' | 'Y') => {
                    last_guess.states[marking_index] = LetterState::PartialMatch;
                    self.advance_feedback_marking(marking_index);
                }
                KeyCode::Char('x' | 'X') => {
                    last_guess.states[marking_index] = LetterState::NoMatch;
                    self.advance_feedback_marking(marking_index);
                }
                KeyCode::Backspace if marking_index > 0 => {
                    // Reset the state of the previous letter before going back
                    last_guess.states[marking_index - 1] = LetterState::Entered;
                    self.state = TuiState::MarkingFeedback {
                        marking_index: marking_index - 1,
                    };
                }
                KeyCode::Char(c) if c.is_ascii_alphabetic() => {
                    self.set_feedback_error(&format!(
                        "Invalid feedback! Use G (green), Y (yellow), or X (gray). ('{}' is not valid)",
                        c.to_ascii_uppercase()
                    ));
                }
                KeyCode::Char(c) => {
                    self.set_feedback_error(&format!(
                        "Only letters G, Y, or X are allowed! ('{c}' is not valid)"
                    ));
                }
                _ => {
                    debug_log!(
                        "handle_feedback_input() - Ignoring non-character key: {:?}",
                        key.code
                    );
                }
            }
        }
        None
    }

    fn handle_confirming_feedback_input(&mut self, key: KeyEvent) -> Option<UserAction> {
        match key.code {
            KeyCode::Esc => {
                info_log!("handle_confirming_feedback_input() - ESC pressed, returning Exit");
                Some(UserAction::Exit)
            }
            KeyCode::Enter => {
                // Confirm the feedback and proceed
                self.state = TuiState::WaitingForNext;
                info_log!("handle_confirming_feedback_input() - Feedback confirmed");
                None
            }
            KeyCode::Backspace => {
                // Go back to editing the last letter
                if let Some(last_guess) = self.guesses.last_mut() {
                    last_guess.states[WORD_LENGTH - 1] = LetterState::Entered;
                    self.state = TuiState::MarkingFeedback {
                        marking_index: WORD_LENGTH - 1,
                    };
                    info_log!(
                        "handle_confirming_feedback_input() - Going back to edit last letter"
                    );
                }
                None
            }
            _ => {
                debug_log!(
                    "handle_confirming_feedback_input() - Ignoring key: {:?}",
                    key.code
                );
                None
            }
        }
    }

    fn has_modifier_keys(key: &KeyEvent) -> bool {
        key.modifiers.contains(event::KeyModifiers::ALT)
            || key.modifiers.contains(event::KeyModifiers::CONTROL)
    }

    fn advance_feedback_marking(&mut self, current_index: usize) {
        if current_index < WORD_LENGTH - 1 {
            self.state = TuiState::MarkingFeedback {
                marking_index: current_index + 1,
            };
        } else {
            self.state = TuiState::ConfirmingFeedback;
        }
    }

    fn set_feedback_error(&mut self, message: &str) {
        self.error_message = message.to_string();
        debug_log!("handle_feedback_input() - {}", message);
    }

    fn handle_waiting_input(&mut self, key: KeyEvent) -> Option<UserAction> {
        if key.code == KeyCode::Esc {
            Some(UserAction::Exit)
        } else {
            self.state = TuiState::EnteringGuess;
            None
        }
    }

    fn handle_game_over_input(key: KeyEvent) -> Option<UserAction> {
        match key.code {
            KeyCode::Char('n' | 'N') => Some(UserAction::NewGame),
            KeyCode::Esc => Some(UserAction::Exit),
            _ => None,
        }
    }

    fn get_feedback_from_last_guess(&self) -> Option<Vec<Feedback>> {
        let last_guess = self.guesses.last()?;
        let feedback: Vec<Feedback> = last_guess
            .states
            .iter()
            .copied()
            .map(LetterState::to_feedback)
            .collect();
        Some(feedback)
    }

    /// Transition to the `MarkingFeedback` state
    fn transition_to_marking_feedback(&mut self, guess: &str) {
        self.state = TuiState::MarkingFeedback { marking_index: 0 };
        self.status = format!("Guess entered: {guess} - Now mark feedback");
    }

    /// Transition to the `EnteringGuess` state
    fn transition_to_entering_guess(&mut self) {
        self.state = TuiState::EnteringGuess;
    }

    /// Transition to the `GameOver` state
    fn transition_to_game_over(&mut self) {
        self.state = TuiState::GameOver;
    }
}

impl GameInterface for TuiInterface {
    fn display_starting_words(&mut self, info: &StartingWordsInfo) {
        self.starting_words.clone_from(&info.words);
        if !info.words.is_empty() {
            self.message = format!("Suggested starting word: {}", info.words[0]);
        }
        self.status = "Ready - Enter your first 5-letter guess".to_string();
        self.draw_or_log();
    }

    fn read_guess(&mut self) -> Option<UserAction> {
        info_log!("read_guess() - Starting guess input loop");
        loop {
            // Draw the current state
            if self.draw().is_err() {
                info_log!("read_guess() - Draw failed, returning Exit");
                return Some(UserAction::Exit);
            }

            // Handle input - this will block until an event is available
            match self.handle_input() {
                Ok(Some(action)) => {
                    info_log!("read_guess() - Action received: {:?}", action);
                    return Some(action);
                }
                Ok(None) => {
                    // No action yet, continue the loop (character was added or ignored)
                }
                Err(_e) => {
                    info_log!("read_guess() - Error handling input, returning Exit");
                    return Some(UserAction::Exit);
                }
            }
        }
    }

    fn read_feedback(&mut self) -> Option<Vec<Feedback>> {
        // Transition to marking state
        self.state = TuiState::MarkingFeedback { marking_index: 0 };
        self.error_message.clear();
        self.status = "Mark each letter: G (green), Y (yellow), or X (gray)".to_string();

        // Draw once before entering loop to show the updated state
        if self.draw().is_err() {
            debug_log!("read_feedback() - Initial draw failed");
            return None;
        }

        loop {
            // Update status if we're in confirming state
            if matches!(self.state, TuiState::ConfirmingFeedback) {
                self.status = "Press ENTER to confirm feedback".to_string();
            }

            // Use handle_input which now properly handles state-based input
            match self.handle_input() {
                Ok(Some(action)) => {
                    // Handle exit during feedback marking
                    match action {
                        UserAction::Exit | UserAction::NewGame => {
                            // Return dummy feedback to allow the action to be processed
                            return Some(vec![Feedback::NoMatch; 5]);
                        }
                        UserAction::Guess(_) => {}
                    }
                }
                Ok(None) => {
                    // Check if we've finished marking and confirmed
                    if matches!(self.state, TuiState::WaitingForNext) {
                        self.status = "Feedback recorded".to_string();
                        self.draw_or_log();
                        return self.get_feedback_from_last_guess();
                    }
                }
                Err(e) => {
                    debug_log!("read_feedback() - Input error: {}", e);
                    return None;
                }
            }

            // Redraw after each input
            if self.draw().is_err() {
                debug_log!("read_feedback() - Draw failed in loop");
                return None;
            }
        }
    }

    fn display_candidates(&mut self, candidates: &[String]) {
        self.candidates_display = candidates.to_vec();
        // If we're in WaitingForNext state, transition out of it
        // This happens after feedback is entered
        if matches!(self.state, TuiState::WaitingForNext) {
            self.state = TuiState::Computing;
        }
        self.status = format!("Filtering candidates... {} remaining", candidates.len());
        self.draw_or_log();
    }

    fn display_recommendation(&mut self, recommendation: &Recommendation) {
        self.recommendation = Some(recommendation.clone());
        self.transition_to_entering_guess();
        self.status = format!("Recommendation ready: {}", recommendation.guess);
        // Clear starting words once we have a recommendation from gameplay
        self.starting_words.clear();
        self.draw_or_log();
    }

    fn display_computing_message(&mut self) {
        // Just update the message, don't block or change to Computing state
        // The Computing state doesn't accept input which causes hangs
        self.message = "Computing optimal guess...".to_string();
        self.status = "Computing optimal next guess...".to_string();
        self.draw_or_log();
    }

    fn display_no_candidates_message(&mut self) {
        self.transition_to_game_over();
        self.message = "No candidates remain. Check your inputs.".to_string();
        self.status = "Error: No valid candidates found".to_string();
        self.draw_or_log();
    }

    fn display_solution_found(&mut self, solution: &str) {
        self.transition_to_game_over();
        self.message = format!("✓ Solution found: {solution}");
        self.status = format!("Game Over - Solution: {solution}");
        self.draw_or_log();
    }

    fn display_exit_message(&mut self) {
        self.message = "Exiting...".to_string();
        self.status = "Exiting application...".to_string();
        self.draw_or_log();
    }

    fn display_new_game_message(&mut self, word_count: usize) {
        self.guesses.clear();
        self.current_input.clear();
        self.candidates_display.clear();
        self.recommendation = None;
        self.transition_to_entering_guess();
        self.message = format!("New game started. Loaded {word_count} words.");
        self.status = "New game - Enter your first guess".to_string();
        self.error_message.clear();
        self.draw_or_log();
    }
}

impl Drop for TuiInterface {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

// Extension trait to add guess recording
impl TuiInterface {
    pub fn record_guess(&mut self, guess: &str) {
        self.guesses.push(GuessRow::from_guess(guess));
    }
}

// We need to intercept guess actions to record them in the TUI
pub struct TuiWrapper {
    interface: TuiInterface,
}

impl TuiWrapper {
    pub fn new() -> Result<Self, io::Error> {
        Ok(Self {
            interface: TuiInterface::new()?,
        })
    }
}

impl GameInterface for TuiWrapper {
    fn display_starting_words(&mut self, info: &StartingWordsInfo) {
        info_log!(
            "TuiWrapper::display_starting_words() - {} words",
            info.words.len()
        );
        self.interface.display_starting_words(info);
    }

    fn read_guess(&mut self) -> Option<UserAction> {
        info_log!("TuiWrapper::read_guess() - Called");
        self.interface.status = "Waiting for guess...".to_string();
        self.interface.draw_or_log();

        let action = self.interface.read_guess();
        info_log!("TuiWrapper::read_guess() - Received action: {:?}", action);

        // Record the guess for display
        if let Some(UserAction::Guess(ref guess)) = action {
            info_log!("TuiWrapper::read_guess() - Recording guess: '{}'", guess);
            self.interface.record_guess(guess);
            // Transition to MarkingFeedback state immediately to prevent showing next empty row
            self.interface.transition_to_marking_feedback(guess);
            // Redraw to show the guess before asking for feedback
            // Note: draw() is synchronous and blocks until rendering is complete
            self.interface.draw_or_log();
            info_log!("TuiWrapper::read_guess() - Guess recorded and displayed");
        }
        action
    }

    fn read_feedback(&mut self) -> Option<Vec<Feedback>> {
        info_log!("TuiWrapper::read_feedback() - Called");
        let result = self.interface.read_feedback();
        info_log!(
            "TuiWrapper::read_feedback() - Feedback received: {:?}",
            result
        );
        result
    }

    fn display_candidates(&mut self, candidates: &[String]) {
        self.interface.display_candidates(candidates);
    }

    fn display_recommendation(&mut self, recommendation: &Recommendation) {
        self.interface.display_recommendation(recommendation);
    }

    fn display_computing_message(&mut self) {
        self.interface.display_computing_message();
    }

    fn display_no_candidates_message(&mut self) {
        self.interface.display_no_candidates_message();
    }

    fn display_solution_found(&mut self, solution: &str) {
        self.interface.display_solution_found(solution);
    }

    fn display_exit_message(&mut self) {
        self.interface.display_exit_message();
    }

    fn display_new_game_message(&mut self, word_count: usize) {
        self.interface.display_new_game_message(word_count);
    }
}
