mod defaults;
mod os;
pub mod permissions;
pub mod symlink;
mod target;
mod target_architecture;

pub use defaults::DEFAULT_CONFIG_DIR;
pub use defaults::DEFAULT_PREFIX;

pub use os::Os;
pub use os::OsVersion;
pub use target::Target;
pub use target_architecture::TargetArchitecture;
