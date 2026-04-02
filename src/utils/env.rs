// SPDX-License-Identifier: GPL-3.0-only
use std::collections::HashMap;

/// Represents an environment, containing environment variables and variables to be stripped.
pub struct Environment {
    pub env_vars: HashMap<String, String>,
    pub stripped_vars: Vec<String>,
}

impl Environment {
    /// Creates a new empty environment.
    pub fn new() -> Self {
        Environment {
            env_vars: HashMap::new(),
            stripped_vars: Vec::new(),
        }
    }

    /// Inserts multiple new variables into the environment.
    /// Note that if a variable was stripped before, it will now be inserted again.
    pub fn insert_vars<K: Into<String>, V: Into<String>>(&mut self, vars: HashMap<K, V>) {
        let vars: HashMap<String, String> = vars.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
        self.stripped_vars.retain(|x| !vars.contains_key(x));
        self.env_vars.extend(vars);
    }

    /// Inserts a new variable into the environment.
    /// Note that if a variable was stripped before, it will now be inserted again.
    pub fn insert_var<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) {
        let key = key.into();
        let value = value.into();
        self.stripped_vars.retain(|x| *x != key);
        self.env_vars.insert(key, value);
    }

    /// Removes a variables from the environment.
    /// Note that if the variable was inserted before, it will now be removed again.
    pub fn strip_var<K: Into<String>>(&mut self, key: K) {
        let key = key.into();
        self.env_vars.remove(&key);
        self.stripped_vars.push(key);
    }
}
