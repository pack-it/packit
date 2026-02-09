mod dependency;
mod package_id;
mod version;
mod version_bounds;

pub use dependency::Dependency;
pub use dependency::DependencyParserError;

pub use package_id::PackageId;

pub use version::Version;
pub use version::VersionError;

pub use version_bounds::VersionBounds;

#[cfg(test)]
pub use dependency::tests as dependency_tests;
