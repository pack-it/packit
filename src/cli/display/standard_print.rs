// SPDX-License-Identifier: GPL-3.0-only
use colored::Colorize;
use std::fmt::Display;

pub trait DisplayOption {
    /// Returns the correct string display for an `Option<impl Display>`, dimmed when `None`.
    fn display(&self) -> String;
}

impl<T> DisplayOption for Option<T>
where
    T: Display,
{
    fn display(&self) -> String {
        match self {
            Some(option) => option.to_string(),
            None => "None".dimmed().to_string(),
        }
    }
}

pub trait DisplayJoined {
    /// Joins all items together with a given separator.
    /// Returns a joined `String`.
    fn display(self, separator: &str) -> String;
}

impl<T: Iterator<Item = impl Display>> DisplayJoined for T {
    fn display(self, separator: &str) -> String {
        self.map(|p| p.to_string()).collect::<Vec<String>>().join(separator)
    }
}

/// Prints a list in the format defined in `print_list`.
/// In case of an empty list "None" is printed.
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
        println!(" - {}", item);
    }

    println!();
}
