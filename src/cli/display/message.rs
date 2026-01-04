use colored::Colorize;
use thiserror::Error;

/// The errors that occur during display.
#[derive(Error, Debug)]
pub enum DisplayError {
    #[error("Could not display: {0}")]
    IOError(#[from] std::io::Error),
}

pub fn display_warning_impl(message: std::fmt::Arguments) {
    println!("{}: {message}", "WARNING".yellow().to_string());
}

// Macro for displaying warnings
macro_rules! display_warning {
    ($message:ident) => {
        $crate::cli::display_warning_impl(format_args!("{}", $message))
    };
    ($($arg:tt)*) => {
        $crate::cli::display::display_warning_impl(format_args!($($arg)*))
    };
}
pub(crate) use display_warning;
