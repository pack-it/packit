// SPDX-License-Identifier: GPL-3.0-only
use std::cmp;

use crate::{
    config::Config,
    installer::types::PackageName,
    repositories::{error::RepositoryError, manager::RepositoryManager},
};

const FUZZY_THRESHOLD: f64 = 0.25;

/// Does a fuzzy search against the repository index.toml.
/// Returns a `RepositoryError` for repository related errors
/// or optionally a `PackageName` if a fuzzy match can be found.
pub fn repository_fuzzy_search(
    config: &Config,
    manager: &RepositoryManager,
    package_name: &PackageName,
) -> Result<Option<PackageName>, RepositoryError> {
    let mut best_word: Option<PackageName> = None;
    let mut best_distance: Option<u64> = None;
    for repository_id in &config.repositories_rank {
        let repository_index = manager.read_index_metadata(&repository_id)?;

        let fuzzy_matches = fuzzy_search(repository_index.supported_packages, package_name.as_str());
        for (distance, fuzzy_match) in fuzzy_matches {
            match manager.read_package(&fuzzy_match) {
                Ok(_) => {},
                Err(RepositoryError::PackageNotFoundError { .. }) => continue,
                Err(e) => return Err(e),
            }

            if let Some(current_distance) = best_distance {
                if distance < current_distance {
                    best_distance = Some(distance);
                    best_word = Some(fuzzy_match)
                }
            } else {
                best_distance = Some(distance);
                best_word = Some(fuzzy_match);
            }
        }
    }

    Ok(best_word)
}

/// Wraps around the `fuzzy_search` method and gets the fuzzy match with
/// the lowest distance to the given string.
/// Returns None if there are no fuzzy matches and a `PackageName` if there is at least one fuzzy match.
pub fn min_fuzzy_search<'a, I>(words: I, string: &str) -> Option<PackageName>
where
    I: IntoIterator,
    I::Item: AsRef<PackageName>,
{
    let mut best_word: Option<PackageName> = None;
    let mut best_distance: Option<u64> = None;
    for (distance, fuzzy_match) in fuzzy_search(words, string) {
        if let Some(current_distance) = best_distance {
            if distance < current_distance {
                best_distance = Some(distance);
                best_word = Some(fuzzy_match)
            }
        } else {
            best_distance = Some(distance);
            best_word = Some(fuzzy_match);
        }
    }

    best_word
}

/// Does a fuzzy search for a string in a list strings. Strings which are underneath the `FUZZY_THRESHOLD`
/// or have a levenshtein distance lower than 1 are a match.
/// Returns a Vec of tuples which are pairs of words and their distance (distance, word).
pub fn fuzzy_search<'a, I>(words: I, string: &str) -> Vec<(u64, PackageName)>
where
    I: IntoIterator,
    I::Item: AsRef<PackageName>,
{
    let mut fuzzy_matches = Vec::new();

    for word in words {
        let distance = calculate_distance(word.as_ref().as_str(), string);

        // Add word to `fuzzy_matches` if the distance is <= 1 or relative distance < `FUZZY_THRESHOLD`
        let max_length = cmp::max(word.as_ref().as_str().len(), string.len());
        let relative_distance = distance as f64 / max_length as f64;
        if distance <= 1 || relative_distance < FUZZY_THRESHOLD {
            fuzzy_matches.push((distance, word.as_ref().clone()));
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_distance_calculation() {
        assert_eq!(calculate_distance("same", "same"), 0);
        assert_eq!(calculate_distance("saem", "same"), 1);
        assert_eq!(calculate_distance("sema", "same"), 2);
        assert_eq!(calculate_distance("sitting", "kitten"), 3);
        assert_eq!(calculate_distance("same", "blub"), 4);
    }

    #[test]
    fn test_fuzzy_search() {
        assert_eq!(
            fuzzy_search(vec![&PackageName::from_str("hello").unwrap()], "hello"),
            vec![(0 as u64, PackageName::from_str("hello").unwrap())]
        );

        assert_eq!(
            fuzzy_search(
                vec![
                    &PackageName::from_str("aahello").unwrap(),
                    &PackageName::from_str("aahell").unwrap(),
                    &PackageName::from_str("aahellow").unwrap(),
                    &PackageName::from_str("aahelloxyz").unwrap(),
                    &PackageName::from_str("aahelloxy").unwrap(),
                ],
                "aahello"
            ),
            vec![
                (0 as u64, PackageName::from_str("aahello").unwrap()),
                (1 as u64, PackageName::from_str("aahell").unwrap()),
                (1 as u64, PackageName::from_str("aahellow").unwrap()),
                (2 as u64, PackageName::from_str("aahelloxy").unwrap()),
            ]
        );
    }
}
