use colored::Colorize;

pub fn warning_impl(message: std::fmt::Arguments) {
    println!("{}: {message}", "WARNING".yellow().to_string());
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
    println!("{}: {message}", "ERROR".red().to_string());
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
