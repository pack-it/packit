// SPDX-License-Identifier: GPL-3.0-only
use std::fmt::Display;

pub const VERTICAL_LINE_PREFIX: &str = " \u{2502} ";

pub struct PairAlginer {
    pairs: Vec<(String, String)>,
}

impl PairAlginer {
    pub fn new() -> Self {
        Self { pairs: Vec::new() }
    }

    #[expect(unused)]
    pub fn from(pairs: Vec<(impl Display, impl Display)>) -> Self {
        let pairs = pairs.into_iter().map(|p| (p.0.to_string(), p.1.to_string())).collect();
        Self { pairs }
    }

    pub fn add(&mut self, key: impl Display, value: impl Display) {
        self.pairs.push((key.to_string(), value.to_string()));
    }

    /// Prints key value pairs in an aligned format.
    pub fn display(&self, prefix: &str) {
        let Some(longest_key) = self.pairs.iter().max_by_key(|s| s.0.len()) else {
            return;
        };

        let longest_key_length = longest_key.0.len();

        for (key, value) in &self.pairs {
            let suffix = " ".repeat(longest_key_length - key.len());
            println!("{}{}{}  {}", prefix, key, suffix, value);
        }
    }
}
