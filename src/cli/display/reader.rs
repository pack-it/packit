use std::io::{self, Read, Seek};

use crate::cli::display::ProgressBar;

/// Custom reader which updates a progress bar.
pub struct ReaderWithProgress<R: Read + Seek> {
    reader: R,
    bar: ProgressBar,
    total: u64,
}

impl<R: Read + Seek> ReaderWithProgress<R> {
    /// Creates a new reader with a progress bar.
    pub fn new(reader: R, size: u64, bar_prefix: String) -> Self {
        Self {
            reader,
            bar: ProgressBar::new(size, bar_prefix),
            total: 0,
        }
    }
}

// Implements read, to update the progress bar.
impl<R: Read + Seek> Read for ReaderWithProgress<R> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let n_bytes = self.reader.read(buffer)?;

        self.total += n_bytes as u64;
        self.bar.set_position(self.total);

        Ok(n_bytes)
    }
}

impl<R: Read + Seek> Seek for ReaderWithProgress<R> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.reader.seek(pos)
    }
}
