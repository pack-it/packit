use std::sync::LazyLock;

use colored::Colorize;

fn log_impl(level: String, message: std::fmt::Arguments) {
    println!("{}: {message}", level.bold());
}

pub fn warning_impl(message: std::fmt::Arguments) {
    log_impl("WARNING".yellow().to_string(), message);
}

// Macro for displaying warnings
macro_rules! warning {
    ($message:ident) => {
        $crate::cli::display::logging::warning_impl(format_args!("{}", $message))
    };
    ($($arg:tt)*) => {
        $crate::cli::display::logging::warning_impl(format_args!($($arg)*))
    };
}
pub(crate) use warning;

pub fn error_impl(message: std::fmt::Arguments) {
    log_impl("ERROR".red().to_string(), message);
}

// Macro for displaying errors
macro_rules! error {
    ($message:ident) => {
        $crate::cli::display::logging::error_impl(format_args!("{}", $message))
    };
    ($($arg:tt)*) => {
        $crate::cli::display::logging::error_impl(format_args!($($arg)*))
    };
}
pub(crate) use error;

pub fn debug_impl(message: std::fmt::Arguments) {
    log_impl("DEBUG".blue().to_string(), message);
}

pub static DEBUG_ENABLED: LazyLock<bool> = LazyLock::new(|| match std::env::var("PACKIT_DEBUG") {
    Ok(value) => value == "true" || value == "1",
    Err(_) => false,
});

macro_rules! debug {
    ($($arg:tt)*) => {
        if *$crate::cli::display::logging::DEBUG_ENABLED {
            $crate::cli::display::logging::debug_impl(format_args!($($arg)*));
        }
    };
}
pub(crate) use debug;
