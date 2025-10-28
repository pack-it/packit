use std::io::{self, Read};

use indicatif::{ProgressBar, ProgressStyle};

/// Custom reader which updates a progress bar.
pub struct ReaderWithProgress<R: Read> {
    reader: R,
    bar: ProgressBar,
    total: u64,
}

impl<R: Read> ReaderWithProgress<R> {
    /// Creates a new reader with a progress bar.
    pub fn new(reader: R, size: u64) -> Self {
        let bar = ProgressBar::new(size);

        // Set the style of the progress bar
        let style = ProgressStyle::with_template("[{wide_bar:.white}] [{percent}%]")
            .expect("Expected progress style here")
            .progress_chars("=> ");
        bar.set_style(style);

        Self { reader, bar, total: 0 }
    }
}

impl<R: Read> Read for ReaderWithProgress<R> {
    // Implements read, so it updates the progress bar.
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
