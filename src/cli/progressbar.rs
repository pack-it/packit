use indicatif::{ProgressBar as IndicatifProgressBar, ProgressStyle};

/// Displays a progress bar for a load process.
pub struct ProgressBar {
    bar: IndicatifProgressBar,
    size: u64,
}

impl ProgressBar {
    /// Creates a new progress bar with the given size.
    pub fn new(size: u64) -> Self {
        let bar = IndicatifProgressBar::new(size);

        // Set the style of the progress bar
        let style = ProgressStyle::with_template("[{wide_bar:.white}] [{percent}%]")
            .expect("Expected progress style here")
            .progress_chars("=> ");
        bar.set_style(style);

        Self { bar, size }
    }

    /// Sets the position of the progress bar.
    pub fn set_position(&self, position: u64) {
        self.bar.set_position(position);

        // Finish the bar if the bar is full
        if position >= self.size {
            self.bar.finish();
        }
    }
}
