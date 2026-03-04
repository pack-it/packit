use crate::platforms::{OsVersion, TargetArchitecture};

#[derive(Debug)]
pub struct Target {
    pub architecture: TargetArchitecture,
    pub os: OsVersion,
}

impl Target {
    pub fn current() -> Self {
        Self {
            architecture: TargetArchitecture::current(),
            os: OsVersion::current(),
        }
    }
}
