// SPDX-License-Identifier: GPL-3.0-only
macro_rules! packit_version {
    () => {
        env!("CARGO_PKG_VERSION")
    };
}
pub(crate) use packit_version;

macro_rules! packit_version_name {
    () => {
        "The Big Bang"
    };
}
pub(crate) use packit_version_name;
