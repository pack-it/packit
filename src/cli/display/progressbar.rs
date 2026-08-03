// SPDX-License-Identifier: GPL-3.0-only
use colored::Colorize;
use indicatif::{ProgressBar as IndicatifProgressBar, ProgressStyle};

/// Represents a progress bar for a load process.
pub struct ProgressBar {
    prefix: String,
    bar: IndicatifProgressBar,
    size: u64,
}

impl ProgressBar {
    /// Creates a new progress bar with the given size.
    pub fn new(size: u64, prefix: String) -> Self {
        let bar = IndicatifProgressBar::new(size);

        // Create instance to use instance methods
        let new_self = Self { prefix, bar, size };

        // Set the style of the progress bar
        new_self.bar.set_style(new_self.create_style());
        new_self.bar.set_position(0);

        new_self
    }

    /// Creates the style of the progress bar.
    fn create_style(&self) -> ProgressStyle {
        let mut prefix = self.prefix.normal();
        let mut percent = "{percent}%".normal();
        if self.bar.position() >= self.size {
            prefix = prefix.bold().green();
            percent = percent.green();
        }

        let template = format!("{prefix} [{{wide_bar}}] [{percent}]");
        ProgressStyle::with_template(&template).expect("Expected template to be correct.").progress_chars("\u{2501}\u{2501} ")
    }

    /// Sets the position of the progress bar.
    pub fn set_position(&mut self, position: u64) {
        self.bar.set_position(position);

        // Finish the bar if the bar is full
        if position >= self.size {
            self.bar.set_style(self.create_style());
            self.bar.finish();
        }
    }

    /// Adjusts the prefix of the progress bar.
    pub fn adjust_prefix(&mut self, prefix: String) {
        self.prefix = prefix;

        self.bar.set_style(self.create_style());
    }
}
