use std::process::exit;

use crate::cli::display;
use crate::error_handling::display::display_error;

pub trait HandleError<T> {
    fn unwrap_or_exit(self) -> T;
}

impl<T, E: std::fmt::Display> HandleError<T> for Result<T, E> {
    fn unwrap_or_exit(self) -> T {
        match self {
            Ok(value) => value,
            Err(e) => {
                display_error(format_args!("{}", e));
                exit(1); // TODO: Make status code variable
            },
        }
    }
}
