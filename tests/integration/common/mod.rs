pub mod environment;

// SPDX-License-Identifier: GPL-3.0-only
#[macro_export]
macro_rules! packit {
    ($env:expr) => {
        assert_cmd::Command::cargo_bin("packit").unwrap().arg("--config-path").arg(&$env.test_directory)
    };
    ($env:expr, $($args:tt)*) => {
        assert_cmd::Command::cargo_bin("packit").unwrap().arg("--config-path").arg(&$env.test_directory).args([$($args)*])
    };
}
