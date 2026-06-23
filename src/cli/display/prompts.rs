// SPDX-License-Identifier: GPL-3.0-only
use std::io::{self, Write};

use crate::{
    cli::display::{
        error::{DisplayError, Result},
        logging::warning,
    },
    utils::ioerror::IOResultExt,
};

/// Represents the response of the prompt.
pub enum QuestionResponse {
    Yes,
    No,
    Invalid,
}

impl QuestionResponse {
    /// Checks if the response is yes.
    pub fn is_yes(&self) -> bool {
        matches!(self, QuestionResponse::Yes)
    }

    /// Checks if the response is no.
    pub fn is_no(&self) -> bool {
        matches!(self, QuestionResponse::No)
    }

    /// Checks if the response is no or invalid.
    pub fn is_no_or_invalid(&self) -> bool {
        self.is_no() || matches!(self, QuestionResponse::Invalid)
    }
}

/// Prompts the user with a yes or no question.
pub fn ask_user(question: &str, default: QuestionResponse) -> Result<QuestionResponse> {
    // Make default bold
    match default {
        QuestionResponse::Yes => print!("{question} [Y/n]: "),
        QuestionResponse::No => print!("{question} [y/N]: "),
        _ => print!("{question} [y/n]: "),
    }

    // Get user input
    let input = read_line()?.to_lowercase();

    // Return default on empty input
    if input.is_empty() {
        return Ok(default);
    }

    if input == "y" || input == "yes" {
        return Ok(QuestionResponse::Yes);
    }

    if input == "n" || input == "no" {
        return Ok(QuestionResponse::No);
    }

    warning!("Invalid input");
    Ok(QuestionResponse::Invalid)
}

/// Prompts the user to give input. The user can skip by pressing enter.
pub fn ask_user_input(question: &str) -> Result<Option<String>> {
    print!("{question} [press enter to skip]: ");

    // Get user input
    let input = read_line()?;
    if input.is_empty() {
        return Ok(None);
    }

    Ok(Some(input.to_string()))
}

/// Reads a line from stdin
fn read_line() -> Result<String> {
    io::stdout().flush().err_operation("flush stdout").map_err(DisplayError::IOError)?;
    let mut input = String::new();
    io::stdin().read_line(&mut input).err_operation("read stdin").map_err(DisplayError::IOError)?;

    Ok(input.trim().to_string())
}
