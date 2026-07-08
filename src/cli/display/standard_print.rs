// SPDX-License-Identifier: GPL-3.0-only

use std::fmt::Display;

use colored::Colorize;

/// Returns the correct string display for an `Option<String>`, dimmed when `None`.
/// TODO: Impl trait
pub fn get_string_option_display(string_option: Option<String>) -> String {
    match string_option {
        Some(string) => string,
        None => "None".dimmed().to_string(),
    }
}

/// Prints a list with help of `print_list`.
/// In case of an empty list "None".dimmed() is printed.
pub fn print_list_or_none<T>(mut items: impl Iterator<Item = T>)
where
    T: Display,
{
    let Some(first_item) = items.next() else {
        println!("{}", "None".dimmed());
        return;
    };

    print!("\n - {}\n", first_item);
    print_list(items);
}

/// Prints the items in a vec as a list, with format:
/// - First item
/// - Second item
///
/// There is always an empty line after a list has been printed.
pub fn print_list<T>(items: impl Iterator<Item = T>)
where
    T: Display,
{
    for item in items {
        print!(" - {}\n", item);
    }

    println!();
}
