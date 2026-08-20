use crate::packit;

#[test]
fn simple_install() {
    packit!().args(["install", "simple", "--build"]).assert().success();

    // TODO:
    // - Check if the file exists in prefix/packages/package/0.0.1/bin
    // - Check if symlink in prefix/bin exists (and points to correct destination)
    // - Check if it exists in the register
    // - Fix exit codes in packit
}
