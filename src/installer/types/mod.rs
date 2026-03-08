mod dependency;
mod optional_id;
mod package_id;
mod package_name;
mod version;
mod version_bounds;
mod version_intervals;

pub use dependency::Dependency;

pub use optional_id::OptionalPackageId;

pub use package_id::PackageId;
pub use package_id::PackageIdError;

pub use package_name::PackageName;

pub use version::Version;
pub use version::VersionError;

pub use version_bounds::VersionBounds;

pub use version_intervals::VersionIntervals;

#[cfg(test)]
pub use dependency::tests as dependency_tests;
