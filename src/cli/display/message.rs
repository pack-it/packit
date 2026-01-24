use colored::Colorize;
use thiserror::Error;

/// The errors that occur during display.
#[derive(Error, Debug)]
pub enum DisplayError {
    #[error("Could not display\nCaused by: {0}")]
    IOError(#[from] std::io::Error),
}

pub fn display_warning(message: std::fmt::Arguments) {
    println!("{}: {message}", "WARNING".yellow().to_string());
}

// Macro for displaying warnings
macro_rules! warning {
    ($message:ident) => {
        $crate::cli::display_warning(format_args!("{}", $message))
    };
    ($($arg:tt)*) => {
        $crate::cli::display::display_warning(format_args!($($arg)*))
    };
}
pub(crate) use warning;

pub fn display_error(message: std::fmt::Arguments) {
    println!("{}: {message}", "ERROR".red().to_string());
}

// Macro for displaying errors
macro_rules! error {
    ($message:ident) => {
        $crate::cli::display_error(format_args!("{}", $message))
    };
    ($($arg:tt)*) => {
        $crate::cli::display::display_error(format_args!($($arg)*))
    };
}
pub(crate) use error;
