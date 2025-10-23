use indicatif::ProgressBar;
use std::io::{self, Read};

pub struct ReaderWithProgress<R: Read> {
    reader: R,
    bar: ProgressBar,
    total: u64,
}

impl<R: Read> ReaderWithProgress<R> {
    pub fn new(reader: R, size: u64) -> Self {
        Self {
            reader,
            bar: ProgressBar::new(size),
            total: 0,
        }
    }
}

impl<R: Read> Read for ReaderWithProgress<R> {
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

pub fn display_error(error: String) {}

pub fn succes_message(message: String) {}

// TODO: Add question to user (continue yes/no)
