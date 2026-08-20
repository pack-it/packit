use crate::packit;

#[test]
fn version() {
    packit!().args(["--version"]).assert().success();
}

#[test]
fn info_requirements() {
    // Install simple package for test
    _ = packit!().args(["install", "simple@0.0.1", "--build"]).ok();

    packit!().args(["info", "--tree"]).assert().failure();
    packit!().args(["info", "--active"]).assert().failure();
    packit!().args(["info", "--tree --active"]).assert().failure();
    packit!().args(["info", "simple", "--tree"]).assert().failure();

    packit!().args(["info"]).assert().success();
    packit!().args(["info", "simple"]).assert().success();
    packit!().args(["info", "simple@0.0.1"]).assert().success();
    packit!().args(["info", "simple@0.0.1", "--tree"]).assert().success();
    packit!().args(["info", "simple", "--active"]).assert().success();
    packit!().args(["info", "simple", "--tree", "--active"]).assert().success();

    // Cleanup simple package after test
    _ = packit!().args(["uninstall", "simple"]).ok();
}
