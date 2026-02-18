mod filesystem;
mod web;

pub const DEFAULT_PROVIDER_ID: &str = web::WEB_PROVIDER_ID;

pub use web::WebProvider;
pub use web::WEB_PROVIDER_ID;

pub use filesystem::FileSystemProvider;
pub use filesystem::FILESYSTEM_PROVIDER_ID;
