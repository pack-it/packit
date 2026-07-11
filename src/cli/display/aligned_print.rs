// SPDX-License-Identifier: GPL-3.0-only
use std::fmt::Display;

/// Utility to print key value pairs in an aligned format.
pub struct PairAligner {
    pairs: Vec<(String, String)>,
}

impl PairAligner {
    pub const VERTICAL_LINE_PREFIX: &str = " \u{2502} ";

    /// Creates a new `PairAligner`.
    pub fn new() -> Self {
        Self { pairs: Vec::new() }
    }

    /// Creates a `PairAligner` from a `Vec` of pairs which have `impl Display` for their members.
    #[expect(unused)]
    pub fn from(pairs: Vec<(impl Display, impl Display)>) -> Self {
        let pairs = pairs.into_iter().map(|(key, value)| (key.to_string(), value.to_string())).collect();
        Self { pairs }
    }

    /// Adds a pair which has `impl Display` for it's members to the stored pairs.
    pub fn add(&mut self, key: impl Display, value: impl Display) {
        self.pairs.push((key.to_string(), value.to_string()));
    }

    /// Prints key value pairs in an aligned format. With the given prefix in front.
    pub fn display(&self, prefix: &str) {
        let Some(longest_key) = self.pairs.iter().max_by_key(|(key, _)| key.len()) else {
            return;
        };

        let longest_key_length = longest_key.0.len();

        for (key, value) in &self.pairs {
            let suffix = " ".repeat(longest_key_length - key.len());
            println!("{}{}{}  {}", prefix, key, suffix, value);
        }
    }
}
