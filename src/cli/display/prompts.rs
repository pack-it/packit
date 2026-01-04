use std::io::{self, Write};

use crate::cli::display::{display_warning, DisplayError};

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
pub fn ask_user(question: &str, default: QuestionResponse) -> Result<QuestionResponse, DisplayError> {
    // Make default bold
    match default {
        QuestionResponse::Yes => print!("{question} [Y/n]: "),
        QuestionResponse::No => print!("{question} [y/N]: "),
        _ => print!("{question} [y/n]: "),
    }

    // Get user input
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();

    // Return default on empty input
    if input == "" {
        return Ok(default);
    }

    if input == "y" || input == "yes" {
        return Ok(QuestionResponse::Yes);
    }

    if input == "n" || input == "no" {
        return Ok(QuestionResponse::No);
    }

    display_warning!("Invalid input");
    return Ok(QuestionResponse::Invalid);
}
