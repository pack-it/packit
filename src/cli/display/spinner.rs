// SPDX-License-Identifier: GPL-3.0-only
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Represents a spinner for a load process.
pub struct Spinner {
    message: String,
    progress_bar: ProgressBar,
}

impl Spinner {
    /// Creates a new spinner.
    pub fn new(message: String) -> Self {
        let progress_bar = ProgressBar::new_spinner();

        // Set the style of the spinner
        let style = ProgressStyle::with_template("{msg} {spinner:.white}")
            .expect("Expected template to be correct.")
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ");
        progress_bar.set_style(style);

        Self { message, progress_bar }
    }

    /// Shows the loading message.
    pub fn show(&self) {
        self.progress_bar.set_message(self.message.clone());
        self.progress_bar.enable_steady_tick(Duration::from_millis(100));
    }

    // Adjusts the spinner message with the new message.
    pub fn adjust_message(&mut self, new_message: String) {
        self.message = new_message;
        self.show();
    }

    /// Finishes the spinner and shows a final message, replacing the original load message.
    pub fn finish(&self) {
        let success_message = format!("{} successful", self.message).bold().green();
        self.progress_bar.finish_with_message(success_message.to_string());
    }
}
