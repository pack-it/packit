#[macro_export]
macro_rules! packit {
    () => {
        assert_cmd::Command::cargo_bin("packit").unwrap()
    };
}
