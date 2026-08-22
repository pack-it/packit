// SPDX-License-Identifier: GPL-3.0-only
#[macro_export]
macro_rules! packit {
    () => {
        assert_cmd::Command::cargo_bin("packit").unwrap()
    };
    ($($args:tt)*) => {
        assert_cmd::Command::cargo_bin("packit").unwrap().args([$($args)*])
    };
}
