pub fn create_path() -> String {
    let mut parts = vec!["/bin", "/sbin", "/usr/bin", "/usr/sbin"];

    // TODO: add packit paths

    parts.join(":")
}
