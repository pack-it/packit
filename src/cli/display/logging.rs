use std::{error::Error, sync::LazyLock};

use colored::Colorize;

fn log_impl(level: String, message: std::fmt::Arguments) {
    println!("{}: {message}", level.bold());
}

pub fn warning_impl(message: std::fmt::Arguments) {
    log_impl("WARNING".yellow().to_string(), message);
}

/// Macro for displaying warnings.
macro_rules! warning {
    ($message:ident) => {
        $crate::cli::display::logging::warning_impl(format_args!("{}", $message))
    };
    ($($arg:tt)*) => {
        $crate::cli::display::logging::warning_impl(format_args!($($arg)*))
    };
}
pub(crate) use warning;

/// Traces the error and creates a message which includes each cause.
fn trace_error<T: Error>(error: T) -> String {
    let mut message = error.to_string();
    let mut source = error.source();
    while let Some(cause) = source {
        message.push_str(format!("\nCaused by: {cause}").as_str());
        source = cause.source();
    }

    message
}

pub fn error_msg_impl(message: std::fmt::Arguments) {
    log_impl("ERROR".red().to_string(), message);
}

pub fn error_impl<T: Error>(error: T) {
    error_msg_impl(format_args!("{}", trace_error(error)));
}

pub fn error_with_msg_impl<T: Error>(error: T, message: std::fmt::Arguments) {
    error_msg_impl(format_args!("{message}\nCaused by: {}", trace_error(error)));
}

/// Macro for displaying errors. Can take an error type as argument or a `msg` field.
macro_rules! error {
    ($error:expr, $message:ident) => {
        $crate::cli::display::logging::error_with_msg_impl($error, format_args!("{}", $message))
    };
    ($error:expr, $($arg:tt)*) => {
        $crate::cli::display::logging::error_with_msg_impl($error, format_args!($($arg)*))
    };
    (msg: $message:ident) => {
        $crate::cli::display::logging::error_msg_impl(format_args!("{}", $message))
    };
    (msg: $($arg:tt)*) => {
        $crate::cli::display::logging::error_msg_impl(format_args!($($arg)*))
    };
    ($error:expr) => {
        $crate::cli::display::logging::error_impl($error)
    };
}
pub(crate) use error;

pub fn debug_impl(message: std::fmt::Arguments) {
    log_impl("DEBUG".blue().to_string(), message);
}

pub fn debug_error_impl<T: Error>(message: std::fmt::Arguments, error: T) {
    let message = format_args!("{message}\nCaused by: {}", trace_error(error));
    log_impl("DEBUG ERROR".purple().to_string(), message);
}

pub static DEBUG_ENABLED: LazyLock<bool> = LazyLock::new(|| match std::env::var("PACKIT_DEBUG") {
    Ok(value) => value == "true" || value == "1",
    Err(_) => false,
});

/// Macro for displaying debug information. Only shows info when debug is enabled.
macro_rules! debug {
    ($error:expr, $($arg:tt)*) => {
        if *$crate::cli::display::logging::DEBUG_ENABLED {
            $crate::cli::display::logging::debug_error_impl(format_args!($($arg)*), $error);
        }
    };
    ($($arg:tt)*) => {
        if *$crate::cli::display::logging::DEBUG_ENABLED {
            $crate::cli::display::logging::debug_impl(format_args!($($arg)*));
        }
    };
}
pub(crate) use debug;
