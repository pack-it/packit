use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::{
    io::{self, Read, Write},
    time::Duration,
};
use thiserror::Error;

/// The errors that occur during display.
#[derive(Error, Debug)]
pub enum DisplayError {
    #[error("Could not display: {0}")]
    IOError(#[from] std::io::Error),
}

/// Custom reader which updates a progress bar.
pub struct ReaderWithProgress<R: Read> {
    reader: R,
    bar: ProgressBar,
    total: u64,
}

impl<R: Read> ReaderWithProgress<R> {
    /// Creates a new custom reader and sets the style of the progress bar.
    pub fn new(reader: R, size: u64) -> Self {
        let bar = ProgressBar::new(size);
        let style = ProgressStyle::with_template("[{wide_bar:.white}] [{percent}%]")
            .expect("Expected progress style")
            .progress_chars("=> ");
        bar.set_style(style);
        Self { reader, bar, total: 0 }
    }
}

impl<R: Read> Read for ReaderWithProgress<R> {
    /// Implements read, so it updates the progress bar.
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let n_bytes = self.reader.read(buffer)?;
        self.total += n_bytes as u64;
        self.bar.set_position(self.total);

        // Finish the bar if the bar is full
        if self.bar.length().expect("Expected bar length") <= self.total {
            self.bar.finish();
        }

        Ok(n_bytes)
    }
}

/// Displays a spinner for a load process.
pub struct DisplayLoad {
    progress_bar: ProgressBar,
}

impl DisplayLoad {
    /// Creates a new load display and sets the spinner style.
    pub fn new() -> Self {
        let progress_bar = ProgressBar::new_spinner();
        progress_bar.set_style(
            ProgressStyle::with_template("{msg} {spinner:.white}")
                .unwrap()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ "),
        );

        Self { progress_bar }
    }

    /// Shows the loading message.
    pub fn show(&self, message: String) {
        self.progress_bar.set_message(message);
        self.progress_bar.enable_steady_tick(Duration::from_millis(100));
    }

    /// Terminates the spinner and shows a final message (which replaces the original load message).
    pub fn show_finish(&self, message: String) {
        self.progress_bar.finish_with_message(message);
    }
}

// Asks user yes or no questions
pub fn ask_user(question: &str, default: bool) -> Result<bool, DisplayError> {
    // Make default bold
    if default {
        print!("{question} ({}/n): ", "y".bold());
    } else {
        print!("{question} (y/{}): ", "n".bold());
    }

    // Get user input
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();

    // Return default on empty input
    if input == "" {
        return Ok(default);
    }

    if input == "y" || input == "yes" {
        return Ok(true);
    }

    Ok(false)
}

pub fn display_warning(message: &str) {
    println!("{}", "WARNING".yellow().to_string() + ": " + message);
}

// TODO: Do we also want our own error formatting (red colors)?
