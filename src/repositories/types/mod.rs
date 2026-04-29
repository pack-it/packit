// SPDX-License-Identifier: GPL-3.0-only
mod checksum;
mod common;
mod index;
mod license;
mod package;
mod package_target;
mod package_version;
mod repository;
mod target_bounds;

pub use self::repository::RepositoryMeta;

pub use self::package::PackageMeta;

pub use self::package_target::PackageTarget;
pub use self::package_version::PackageVersionMeta;

pub use self::checksum::Checksum;
pub use self::common::Script;

pub use self::common::Patch;
pub use self::common::Source;
pub use self::common::Sources;

pub use self::index::IndexMeta;

pub use self::license::Licenses;

pub use self::target_bounds::TargetBounds;
