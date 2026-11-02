// SPDX-License-Identifier: GPL-3.0-only
use crate::{common::environment::Environment, packit};

#[test]
fn version() {
    let env = Environment::init();
    packit!(env, "--version").assert().success();
    env.clean();
}

#[test]
fn info_requirements() {
    let env = Environment::init();

    // Install simple package for test
    _ = packit!(env, "install", "simple@0.0.1", "--build").assert().success();

    packit!(env, "info", "--tree").assert().failure();
    packit!(env, "info", "--active").assert().failure();
    packit!(env, "info", "--tree --active").assert().failure();
    packit!(env, "info", "simple", "--tree").assert().failure();

    packit!(env, "info").assert().success();
    packit!(env, "info", "simple").assert().success();
    packit!(env, "info", "simple@0.0.1").assert().success();
    packit!(env, "info", "simple@0.0.1", "--tree").assert().success();
    packit!(env, "info", "simple", "--active").assert().success();
    packit!(env, "info", "simple", "--tree", "--active").assert().success();

    env.clean();
}
