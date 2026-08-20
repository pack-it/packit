use crate::packit;

#[test]
fn version() {
    packit!().args(["--version"]).assert().success();
}

#[test]
fn info_requirements() {
    packit!().args(["info", "--tree"]).assert().failure();
    packit!().args(["info", "--active"]).assert().failure();
    packit!().args(["info", "--tree --active"]).assert().failure();

    // TODO: Install simple package first
    // packit!().args(["info", "simple", "--tree"]).assert().success();
    // packit!().args(["info", "simple", "--active"]).assert().success();
    // packit!().args(["info", "simple", "--tree --active"]).assert().success();
}
