// SPDX-License-Identifier: GPL-3.0-only
use crate::{common::environment::Environment, packit, util};
use std::fs;

#[test]
fn basic_install() {
    let env = Environment::init();

    // Install simple package for test
    _ = packit!(env, "install", "simple@0.0.1", "--build").assert().success();

    assert!(util::check_package_directory(&env.test_directory, "simple", &["0.0.1"]));
    assert!(util::check_local_metadata(&env.test_directory, "simple", &["0.0.1"]));
    assert!(util::check_active_package(&env.test_directory, "simple", "0.0.1"));

    // Check symlink existence and destination
    let Ok(destination) = fs::read_link(&env.test_directory.join("bin").join("simple")) else {
        panic!("Symlink doesn't exist or is not a symlink");
    };

    assert_eq!(
        destination,
        env.test_directory.join("packages").join("simple").join("0.0.1").join("bin").join("simple")
    );

    // Check that there is no dependency directory
    assert!(!env.test_directory.join("dependencies").join("simple@0.0.1").exists());

    env.clean();
}
