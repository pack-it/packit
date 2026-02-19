mod filesystem;
mod web;

pub const DEFAULT_METADATA_PROVIDER_ID: &str = web::WEB_METADATA_PROVIDER_ID;

pub use web::WebMetadataProvider;
pub use web::WEB_METADATA_PROVIDER_ID;

pub use filesystem::FileSystemMetadataProvider;
pub use filesystem::FILESYSTEM_METADATA_PROVIDER_ID;
