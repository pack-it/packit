// SPDX-License-Identifier: GPL-3.0-only
pub mod error;
mod handler;

pub use handler::LocalMetaHandler;

#[expect(unused_imports)]
pub use handler::LocalMetadata;

#[expect(unused_imports)]
pub use handler::LocalPrebuildMetadata;
