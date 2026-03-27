use crate::platforms::{OsVersion, TargetArchitecture};

/// Represents a target, which is its architecture and OS version combined.
#[derive(Debug)]
pub struct Target {
    pub architecture: TargetArchitecture,
    pub os: OsVersion,
}

impl Target {
    /// Gets the current target.
    pub fn current() -> Self {
        Self {
            architecture: TargetArchitecture::current(),
            os: OsVersion::current(),
        }
    }
}
