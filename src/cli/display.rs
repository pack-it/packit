use colored::Colorize;
use thiserror::Error;

/// The errors that occur during display.
#[derive(Error, Debug)]
pub enum DisplayError {
    #[error("Could not display: {0}")]
    IOError(#[from] std::io::Error),
}

pub fn display_warning(message: &str) {
    println!("{}: {message}", "WARNING".yellow().to_string());
}

// TODO: Do we also want our own error formatting (red colors)?
