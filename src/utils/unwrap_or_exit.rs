use std::{error::Error, process::exit};

use crate::cli::display::logging::error;

/// A trait to unwrap or exit the program.
pub trait UnwrapOrExit<T> {
    /// Unwraps and exists with a message and an exit code.
    fn unwrap_or_exit_msg(self, msg: &str, exit_code: i32) -> T;

    /// Unwraps and exists with an exit code.
    fn unwrap_or_exit(self, exit_code: i32) -> T;
}

/// Implementation of UnwrapOrExit for Result.
impl<T, E: Error> UnwrapOrExit<T> for Result<T, E> {
    fn unwrap_or_exit_msg(self, msg: &str, exit_code: i32) -> T {
        match self {
            Ok(value) => value,
            Err(e) => {
                error!(e, msg);
                exit(exit_code);
            },
        }
    }

    fn unwrap_or_exit(self, exit_code: i32) -> T {
        match self {
            Ok(value) => value,
            Err(e) => {
                error!(e);
                exit(exit_code);
            },
        }
    }
}

/// Implementation of UnwrapOrExit for Option.
impl<T> UnwrapOrExit<T> for Option<T> {
    fn unwrap_or_exit_msg(self, msg: &str, exit_code: i32) -> T {
        match self {
            Some(value) => value,
            None => {
                error!(msg: msg);
                exit(exit_code);
            },
        }
    }

    fn unwrap_or_exit(self, exit_code: i32) -> T {
        match self {
            Some(value) => value,
            None => {
                exit(exit_code);
            },
        }
    }
}
