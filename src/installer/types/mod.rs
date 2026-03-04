mod dependency;
mod optional_id;
mod package_id;
mod version;
mod version_bounds;

pub use dependency::Dependency;

pub use optional_id::OptionalPackageId;

pub use package_id::PackageId;
pub use package_id::PackageIdError;

pub use version::Version;
pub use version::VersionError;

pub use version_bounds::VersionBounds;

#[cfg(test)]
pub use dependency::tests as dependency_tests;
