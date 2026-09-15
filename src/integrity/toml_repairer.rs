// SPDX-License-Identifier: GPL-3.0-only
use toml_edit::DocumentMut;

fn repair_toml(content: &str) -> Option<String> {
    let mut lines: Vec<&str> = content.split('\n').collect();

    // Filter empty and white space lines
    lines.retain(|s| !s.trim().is_empty());

    // Collect errors
    let mut line_errors = collect_error_lines(&lines);

    // Filter the multi-line errors
    filter_multiline_errors(&mut line_errors, &lines);

    if line_errors.is_empty() {
        return Some(content.to_string());
    }

    let repaired_toml: Vec<&str> = lines.iter().enumerate().filter(|(i, _)| line_errors.iter().all(|x| *x != *i)).map(|v| *v.1).collect();
    let repaired_toml = repaired_toml.join("\n");

    Some(repaired_toml)
}

/// Collects all lines which have errors.
/// Note that this includes toml content which spans multiple lines.
fn collect_error_lines(lines: &Vec<&str>) -> Vec<usize> {
    let mut error_line_indices = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        if line.parse::<DocumentMut>().is_err() {
            error_line_indices.push(index);
        }
    }

    error_line_indices
}

/// Filters the errors arising from multiline toml.
/// If you take a single line of toml which spans multiple lines it will give an error. This function filters those errors.
/// We iterate through all lines with errors, we append them until they are valid toml.
/// If the first line cannot be valid when combined with other lines this line contains an actual error.
/// We then continue this process with the next line. To make this more efficient we also check if there is a 'line-gap' between
/// the current and previous error line. If this is the case we can reset early.
fn filter_multiline_errors(line_errors: &mut Vec<usize>, lines: &Vec<&str>) {
    let mut block = String::new();
    let mut block_start = 0;
    let mut previous_index = 0;
    let mut index = 0;
    while let Some(line_index) = line_errors.get(index) {
        let Some(line) = lines.get(*line_index) else {
            continue;
        };

        if !block.is_empty() && previous_index + 1 != *line_index {
            block = String::new();
            index = block_start + 1;
            block_start = index;
            continue;
        }

        previous_index = *line_index;

        block.push_str(line);
        block.push('\n');
        if block.parse::<DocumentMut>().is_ok() {
            line_errors.drain(block_start..index + 1);
            index -= index - block_start;
            block = String::new();
            block_start = index;
        }

        index += 1;

        // Reset the block and index if we are out of lines
        if index == line_errors.len() && !block.is_empty() {
            block = String::new();
            index = block_start + 1;
            block_start = index;
        }
    }
}
