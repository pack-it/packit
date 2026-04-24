// SPDX-License-Identifier: GPL-3.0-only
mod filesystem;
mod web;

pub const DEFAULT_PREBUILD_PROVIDER_ID: &str = filesystem::FILESYSTEM_PREBUILD_PROVIDER_ID;

pub use filesystem::FILESYSTEM_PREBUILD_PROVIDER_ID;
pub use filesystem::FileSystemPrebuildProvider;

pub use web::WEB_PREBUILD_PROVIDER_ID;
pub use web::WebPrebuildProvider;
