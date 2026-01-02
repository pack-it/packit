use serde::{de, Deserialize, Serialize};

#[derive(Debug)]
pub enum VersionLimitType {
    Lower,
    LowerEqual,
    Higher,
    HigherEqual,
    Equal,
}

#[derive(Debug)]
pub struct Dependency {
    name: String,
    version: Option<String>,
    version_limit_type: Option<VersionLimitType>,
}

impl<'de> Deserialize<'de> for Dependency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = de::Deserialize::deserialize(deserializer)?;
        let index = s.chars().position(|c| c == '@');

        let (name, version) = match index {
            Some(index) => s.split_at(index),
            None => {
                return Ok(Self {
                    name: s.to_string(),
                    version: None,
                    version_limit_type: None,
                })
            },
        };

        // Determine the version limit type and strip the version
        let (version, version_limit_type) = if let Some(version) = version.strip_prefix('<') {
            (version, VersionLimitType::Lower)
        } else if let Some(version) = version.strip_prefix("<=") {
            (version, VersionLimitType::LowerEqual)
        } else if let Some(version) = version.strip_prefix('>') {
            (version, VersionLimitType::Higher)
        } else if let Some(version) = version.strip_prefix(">=") {
            (version, VersionLimitType::HigherEqual)
        } else {
            (version, VersionLimitType::Equal)
        };

        Ok(Self {
            name: name.to_string(),
            version: Some(version.to_string()),
            version_limit_type: Some(version_limit_type),
        })
    }
}

impl Serialize for Dependency {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Return only the name if the version isn't specified
        let version = match &self.version {
            Some(version) => version,
            None => return serializer.collect_str(&self.name),
        };

        // Get character representation of version limit type
        let version_limit_type = match self.version_limit_type {
            Some(VersionLimitType::Lower) => "<",
            Some(VersionLimitType::LowerEqual) => "<=",
            Some(VersionLimitType::Higher) => ">",
            Some(VersionLimitType::HigherEqual) => ">=",
            Some(VersionLimitType::Equal) => todo!(),
            None => unreachable!(),
        };

        let dependency_string = self.name.clone() + version_limit_type + &version;
        serializer.collect_str(&dependency_string)
    }
}

impl Dependency {
    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn satisfied(&self) -> bool {
        // TODO
        false
    }
}
