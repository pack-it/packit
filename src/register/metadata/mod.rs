// SPDX-License-Identifier: GPL-3.0-only
pub mod error;
mod handler;

pub use handler::LocalMetaHandler;
pub use handler::LocalMetaPackageHandler;

pub use handler::LocalMetadata;

#[expect(unused_imports)]
pub use handler::LocalPrebuildMetadata;
