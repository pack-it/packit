mod defaults;
mod os;
pub mod symlink;
mod target;
mod target_architecture;

pub use defaults::DEFAULT_CONFIG_DIR;
pub use defaults::DEFAULT_PREFIX;
pub use target_architecture::TARGET_ARCHITECTURE;

pub use target_architecture::get_os_name;
pub use target_architecture::is_unix;

pub use os::OsVersion;
pub use target::Target;
