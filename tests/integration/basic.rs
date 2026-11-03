// SPDX-License-Identifier: GPL-3.0-only
use crate::{common::environment::Environment, packit};

#[test]
fn version() {
    let env = Environment::init();
    packit!(env, "--version").assert().success();
    env.clean();
}
