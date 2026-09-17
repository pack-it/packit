// SPDX-License-Identifier: GPL-3.0-only
use toml_edit::DocumentMut;

/// Repairs the toml content by removing erroneous lines.
pub fn repair_toml(content: &str) -> String {
    let mut lines: Vec<&str> = content.split('\n').collect();

    // Filter empty and white space lines
    lines.retain(|s| !s.trim().is_empty());

    // Collect errors
    let mut line_errors = collect_error_lines(&lines);

    // Filter the multi-line errors
    filter_multiline_errors(&mut line_errors, &lines);

    if line_errors.is_empty() {
        return content.to_string();
    }

    let repaired_toml: Vec<&str> = lines.iter().enumerate().filter(|(i, _)| line_errors.iter().all(|x| *x != *i)).map(|v| *v.1).collect();
    repaired_toml.join("\n")
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

        // If there is a 'gap' between blocks, meaning a line without an error interrupts the previous error and the current error,
        // then we reset the block. We will start the same process again, but without the first line of the block.
        // Trying all combinations of a block is necessary, because for example if all lines have errors and only line 1 has a real error
        // then the later lines would be wrongly counted as errors.
        if !block.is_empty() && previous_index + 1 != *line_index {
            block = String::new();
            index = block_start + 1;
            block_start = index;
            continue;
        }

        previous_index = *line_index;

        // Push the new line to the block and check if this block works
        block.push_str(line);
        block.push('\n');

        // If the block works we remove it from the errors list. The index is lowered by the block size,
        // because this is removed from the `line_errors`. The block will be reset.
        if block.parse::<DocumentMut>().is_ok() {
            line_errors.drain(block_start..index + 1);
            let block_size = index - block_start;
            index -= block_size;
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

#[cfg(test)]
pub mod tests {

    use super::*;

    #[test]
    fn empty() {
        assert_eq!(repair_toml(""), "");
    }

    #[test]
    fn valid() {
        assert_eq!(repair_toml("key = 1\nother = 2"), "key = 1\nother = 2");
    }

    #[test]
    fn valid_with_empty_line() {
        assert_eq!(repair_toml("key = [1,\n\n2,\n3\n]"), "key = [1,\n\n2,\n3\n]");
    }

    #[test]
    fn invalid_before_valid() {
        assert_eq!(repair_toml("key2 = [\n1,,\n]\nkey = [1,\n2,\n3\n]"), "key = [1,\n2,\n3\n]");
    }

    #[test]
    fn invalid_with_gap() {
        assert_eq!(repair_toml("key = [\nother = 1\n]"), "other = 1");
    }

    #[test]
    fn collect_emtpy() {
        assert!(collect_error_lines(&vec![]).is_empty());
    }

    #[test]
    fn collect_no_errors() {
        assert!(collect_error_lines(&vec!["key = 1", "key = []"]).is_empty());
    }

    #[test]
    fn collect_only_errors() {
        let collected_errors = collect_error_lines(&vec!["key = ", "key = ["]);
        assert!(collected_errors.len() == 2);
        assert_eq!(collected_errors.get(0), Some(&0));
        assert_eq!(collected_errors.get(1), Some(&1));
    }

    #[test]
    fn collect_mixed() {
        let collected_errors = collect_error_lines(&vec!["key = ", "key = 1", "key = ["]);
        assert!(collected_errors.len() == 2);
        assert_eq!(collected_errors.get(0), Some(&0));
        assert_eq!(collected_errors.get(1), Some(&2));
    }

    #[test]
    fn filter_empty() {
        let lines = vec!["key = 1"];
        let mut line_errors = vec![];
        filter_multiline_errors(&mut line_errors, &lines);

        assert_eq!(line_errors, vec![]);

        let mut line_errors = vec![];
        filter_multiline_errors(&mut line_errors, &vec![]);

        assert_eq!(line_errors, vec![]);
    }

    #[test]
    fn filter_no_multiline() {
        let lines = vec!["key = ", "key = 1", "key = ["];
        let mut line_errors = vec![0, 2];
        filter_multiline_errors(&mut line_errors, &lines);

        assert_eq!(line_errors, vec![0, 2]);
    }

    #[test]
    fn filter_all_errors() {
        let lines = vec!["key = [", "1", "]"];
        let mut line_errors = vec![0, 1, 2];
        filter_multiline_errors(&mut line_errors, &lines);

        assert_eq!(line_errors, vec![]);
    }

    #[test]
    fn filter_some_errors() {
        let lines = vec!["key = [", "]", "other = 1", "x"];
        let mut line_errors = vec![0, 1, 3];
        filter_multiline_errors(&mut line_errors, &lines);

        assert_eq!(line_errors, vec![3]);
    }
}
