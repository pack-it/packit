use std::{error::Error, process::exit};

use crate::cli::display::logging::error;

pub trait HandleError<T> {
    fn unwrap_or_exit(self, exit_code: i32) -> T;
}

impl<T, E: Error> HandleError<T> for Result<T, E> {
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
