use crate::packit;

#[test]
fn version() {
    packit!("--version").assert().success();
}

#[test]
fn info_requirements() {
    // Install simple package for test
    _ = packit!("install", "simple@0.0.1", "--build").ok();

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

    // Cleanup simple package after test
    _ = packit!("uninstall", "simple").ok();
}
