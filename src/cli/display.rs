use indicatif::{ProgressBar, ProgressStyle};
use std::{
    io::{self, Read},
    time::Duration,
};

pub struct ReaderWithProgress<R: Read> {
    reader: R,
    bar: ProgressBar,
    total: u64,
}

impl<R: Read> ReaderWithProgress<R> {
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

pub struct DisplayLoad {
    progress_bar: ProgressBar,
}

impl DisplayLoad {
    pub fn new() -> Self {
        let progress_bar = ProgressBar::new_spinner();
        progress_bar.set_style(
            ProgressStyle::with_template("{msg} {spinner:.white}")
                .unwrap()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ "),
        );

        Self { progress_bar }
    }

    pub fn show(&self, message: String) {
        self.progress_bar.set_message(message);
        self.progress_bar.enable_steady_tick(Duration::from_millis(100));
    }

    pub fn show_finish(&self, message: String) {
        self.progress_bar.finish_with_message(message);
    }
}

// TODO: Implement
pub fn ask_user(question: &str) -> bool {
    true
}
