// SPDX-License-Identifier: GPL-3.0-only
pub mod error;
mod handler;
mod register_ext;

pub use handler::LocalMetaHandler;
pub use handler::LocalMetadata;
pub use handler::LocalPrebuildMetadata;

pub use register_ext::PackageRegisterExt;
