// SPDX-License-Identifier: GPL-3.0-only
use terminal_size::{Height, Width, terminal_size};

pub fn print_grid(items: Vec<String>) {
    // Get the width of the terminal for grid calculations
    let terminal_width = match terminal_size() {
        Some((Width(width), Height(_))) => width as usize,
        None => panic!("TODO"),
    };

    // Get the widest string in the given items
    let mut widest = &String::from("");
    for item in &items {
        if item.len() > widest.len() {
            widest = item;
        }
    }

    // Calculate column and row (automatically rounds down to the nearest int), +1 for whitespace
    let column_width = (widest.len() + 1) as usize;
    let column_count = terminal_width / column_width;
    let row_count = items.len().div_ceil(column_count);

    // Print the items in the grid
    for i in 0..row_count {
        for j in 0..column_count {
            if let Some(item) = items.get(i * column_count + j) {
                let current_length = item.len();
                let padding = " ".repeat(column_width - current_length);
                print!("{item}{padding}");
            }
        }

        println!()
    }
}
