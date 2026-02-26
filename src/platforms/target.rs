use crate::platforms::{OsVersion, TARGET_ARCHITECTURE};

pub struct Target {
    pub architecture: String,
    pub os: OsVersion,
}

impl Target {
    pub fn current() -> Self {
        Self {
            architecture: TARGET_ARCHITECTURE.into(),
            os: OsVersion::current(),
        }
    }
}
