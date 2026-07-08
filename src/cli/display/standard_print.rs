// SPDX-License-Identifier: GPL-3.0-only
use colored::Colorize;

/// Returns the correct string display for an `Option<String>`, dimmed when `None`.
pub fn get_string_option_display(string_option: Option<String>) -> String {
    match string_option {
        Some(string) => string,
        None => "None".dimmed().to_string(),
    }
}
