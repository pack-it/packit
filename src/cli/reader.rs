use std::io::{self, Read};

use crate::cli::ProgressBar;

/// Custom reader which updates a progress bar.
pub struct ReaderWithProgress<R: Read> {
    reader: R,
    bar: ProgressBar,
    total: u64,
}

impl<R: Read> ReaderWithProgress<R> {
    /// Creates a new reader with a progress bar.
    pub fn new(reader: R, size: u64) -> Self {
        Self {
            reader,
            bar: ProgressBar::new(size),
            total: 0,
        }
    }
}

impl<R: Read> Read for ReaderWithProgress<R> {
    // Implements read, so it updates the progress bar.
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let n_bytes = self.reader.read(buffer)?;

        self.total += n_bytes as u64;
        self.bar.set_position(self.total);

        Ok(n_bytes)
    }
}
