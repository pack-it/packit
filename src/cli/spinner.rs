use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

/// Displays a spinner for a load process.
pub struct Spinner {
    progress_bar: ProgressBar,
}

impl Spinner {
    /// Creates a new spinner.
    pub fn new() -> Self {
        let progress_bar = ProgressBar::new_spinner();

        // Set the style of the spinner
        let style = ProgressStyle::with_template("{msg} {spinner:.white}").unwrap().tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ");
        progress_bar.set_style(style);

        Self { progress_bar }
    }

    /// Shows the loading message.
    pub fn show(&self, message: String) {
        self.progress_bar.set_message(message);
        self.progress_bar.enable_steady_tick(Duration::from_millis(100));
    }

    /// Finishes the spinner and shows a final message, replacing the original load message.
    pub fn finish(&self, message: String) {
        self.progress_bar.finish_with_message(message);
    }
}
