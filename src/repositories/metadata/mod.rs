// SPDX-License-Identifier: GPL-3.0-only
mod filesystem;
mod provider;
mod web;

pub const DEFAULT_METADATA_PROVIDER_ID: &str = web::WEB_METADATA_PROVIDER_ID;

pub use provider::MetadataProvider;

pub use filesystem::FILESYSTEM_METADATA_PROVIDER_ID;
pub use web::WEB_METADATA_PROVIDER_ID;
