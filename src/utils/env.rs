// SPDX-License-Identifier: GPL-3.0-only
use std::collections::{HashMap, HashSet};

/// Represents an environment, containing environment variables and variables to be stripped.
pub struct Environment {
    env_vars: HashMap<String, String>,
    stripped_vars: HashSet<String>,
}

impl Environment {
    /// Creates a new empty environment.
    pub fn new() -> Self {
        Self {
            env_vars: HashMap::new(),
            stripped_vars: HashSet::new(),
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
        self.stripped_vars.insert(key);
    }

    /// Expands the environment with another environment.
    pub fn expand(&mut self, other_environment: Environment) {
        self.insert_vars(other_environment.env_vars);

        for stripped_var in other_environment.stripped_vars {
            self.strip_var(stripped_var);
        }
    }

    /// Gets the env vars.
    pub fn get_env_vars(&self) -> &HashMap<String, String> {
        &self.env_vars
    }

    /// Gets the stripped vars.
    pub fn get_stripped_vars(&self) -> &HashSet<String> {
        &self.stripped_vars
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashSet;

    use super::*;

    #[test]
    fn insert_var() {
        let mut env = Environment::new();
        env.insert_var("test", "some_value");
        env.insert_var("test2", "");
        assert_eq!(
            env.env_vars,
            HashMap::from([
                ("test".to_string(), "some_value".to_string()),
                ("test2".to_string(), "".to_string())
            ])
        );
    }

    #[test]
    fn insert_vars() {
        let mut env = Environment::new();
        let vars = HashMap::from([
            ("test".to_string(), "some_value".to_string()),
            ("test2".to_string(), "".to_string()),
        ]);
        env.insert_vars(vars.clone());
        assert_eq!(env.env_vars, vars);
        assert_eq!(env.stripped_vars, HashSet::new());
    }

    #[test]
    fn insert_vars_empty() {
        let mut env = Environment::new();
        env.insert_vars(HashMap::<String, String>::new());
        assert_eq!(env.env_vars, Environment::new().env_vars);
        assert_eq!(env.stripped_vars, Environment::new().stripped_vars);
    }

    #[test]
    fn strip_var() {
        let mut env = Environment::new();
        env.strip_var("test");
        env.strip_var("test2");
        assert_eq!(env.stripped_vars, HashSet::from(["test".to_string(), "test2".to_string()]));
        assert_eq!(env.env_vars, HashMap::new());
    }

    #[test]
    fn strip_and_insert() {
        let mut env = Environment::new();
        env.insert_var("test", "test_value");
        env.insert_var("test2", "test_value2");
        env.strip_var("test2");
        assert_eq!(env.env_vars, HashMap::from([("test".to_string(), "test_value".to_string())]));
        assert_eq!(env.stripped_vars, HashSet::from(["test2".to_string()]));
        env.strip_var("test3");
        env.insert_var("test3", "test_value3");
        assert_eq!(
            env.env_vars,
            HashMap::from([
                ("test".to_string(), "test_value".to_string()),
                ("test3".to_string(), "test_value3".to_string())
            ])
        );
        assert_eq!(env.stripped_vars, HashSet::from(["test2".to_string()]));
    }
}
