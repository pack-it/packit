// SPDX-License-Identifier: GPL-3.0-only
use crate::{common::environment::Environment, packit};

#[test]
fn version() {
    let environment = Environment::init();
    packit!("--version").assert().success();
    environment.clean();
}

#[test]
fn info_requirements() {
    let environment = Environment::init();

    // Install simple package for test
    _ = packit!("install", "simple@0.0.1", "--build").assert().success();

    packit!("info", "--tree").assert().failure();
    packit!("info", "--active").assert().failure();
    packit!("info", "--tree --active").assert().failure();
    packit!("info", "simple", "--tree").assert().failure();

    packit!("info").assert().success();
    packit!("info", "simple").assert().success();
    packit!("info", "simple@0.0.1").assert().success();
    packit!("info", "simple@0.0.1", "--tree").assert().success();
    packit!("info", "simple", "--active").assert().success();
    packit!("info", "simple", "--tree", "--active").assert().success();

    environment.clean();
}
