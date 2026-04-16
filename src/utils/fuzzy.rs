use std::cmp;

const FUZZY_THRESHOLD: f64 = 0.25;

/// Does a fuzzy search for a string in a list strings. It will search for the string with the lowest distance
/// and then calculate a relative distance to make sure string length is taken into account.
/// Returns the most simmilar word or None if the word was above the `FUZZY_THRESHOLD`.
pub fn fuzzy_search(words: Vec<&str>, string: &str) -> Option<String> {
    let mut best_distance = None;
    let mut current_best = None;

    for word in words {
        let distance = calculate_distance(word, string);

        if let Some(current_distance) = best_distance {
            if distance < current_distance {
                best_distance = Some(distance);
                current_best = Some(word.to_string())
            }
        } else {
            best_distance = Some(distance);
            current_best = Some(word.to_string());
        }
    }

    // Make sure that the best word has a relative distance below the `FUZZY_THRESHOLD` or
    // that it has a distance of 1 or lower.
    if let Some(best_distance) = best_distance
        && let Some(current_best) = current_best.as_ref()
    {
        let max_length = cmp::max(current_best.len(), string.len());
        let relative_distance: f64 = best_distance as f64 / max_length as f64;
        if best_distance > 1 && relative_distance > FUZZY_THRESHOLD {
            return None;
        }
    }

    current_best
}

/// Calculates the distance between two strings with a levenshtein like algorithm which allows
/// for transpositions for characters which are next to each other (ideal for typo's).
/// Returns the distance between two strings.
fn calculate_distance(first_word: &str, second_word: &str) -> u64 {
    let first_word_char = first_word.as_bytes();
    let second_word_char = second_word.as_bytes();
    let mut distance_matrix = vec![vec![0 as u64; second_word.len() + 1]; first_word.len() + 1];

    for i in 0..first_word.len() + 1 {
        distance_matrix[i][0] = i as u64;
    }

    for j in 0..second_word.len() + 1 {
        distance_matrix[0][j] = j as u64;
    }

    for i in 1..first_word.len() + 1 {
        for j in 1..second_word.len() + 1 {
            let cost = match first_word_char[i - 1] == second_word_char[j - 1] {
                true => 0,
                false => 1,
            };

            // Deletion and insertion
            distance_matrix[i][j] = cmp::min(distance_matrix[i - 1][j] + 1, distance_matrix[i][j - 1] + 1);

            // Substitution
            distance_matrix[i][j] = cmp::min(distance_matrix[i][j], distance_matrix[i - 1][j - 1] + cost);

            // Transposition
            if i > 1 && j > 1 && first_word_char[i - 1] == second_word_char[j - 2] && first_word_char[i - 2] == second_word_char[j - 1] {
                distance_matrix[i][j] = cmp::min(distance_matrix[i][j], distance_matrix[i - 2][j - 2] + cost);
            }
        }
    }

    for i in &distance_matrix {
        for j in i {
            print!("{j} ")
        }
        println!();
    }

    distance_matrix[first_word.len()][second_word.len()]
}
