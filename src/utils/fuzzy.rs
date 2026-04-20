use std::cmp;

const FUZZY_THRESHOLD: f64 = 0.25;

/// Does a fuzzy search for a string in a list strings. Strings which are underneath the `FUZZY_THRESHOLD`
/// or have a levenshtein distance lower than 1 are a match.
/// Returns a Vec of tuples which are pairs of words and their distance (distance, word).
pub fn fuzzy_search(words: Vec<&str>, string: &str) -> Vec<(u64, String)> {
    let mut fuzzy_matches = Vec::new();

    for word in words {
        let distance = calculate_distance(word, string);

        // Add word to `fuzzy_matches` if the distance is <= 1 or relative distance < `FUZZY_THRESHOLD`
        let max_length = cmp::max(word.len(), string.len());
        let relative_distance = distance as f64 / max_length as f64;
        if distance <= 1 || relative_distance < FUZZY_THRESHOLD {
            fuzzy_matches.push((distance, word.to_string()));
        }
    }

    fuzzy_matches
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

    distance_matrix[first_word.len()][second_word.len()]
}
