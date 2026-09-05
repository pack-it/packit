# Tests

Packit has both unit and integration tests. These tests are separated, because the integration tests require setup and clean up functionality.

### Unit tests
Run the following command to run the unit tests:
```
cargo test
```

### Integration tests
Cargo test doesn't have support for a general setup or clean up before and after the integration tests. That's why Packit uses an xtask `xtask-test-runner` which wraps around the `cargo test` command. 
The arguments to `xtest` are passed along to `cargo test`, so test flags can still be used.
Run the following command to run the integration tests:
```
cargo xtest
```

For the setup this wrapper first checks for the `TestConfig.toml`, which is located next to the `Config.toml`. The `TestConfig.toml` is used if it exists, otherwise a default test config is created. The default test config uses `<prefix>/test` as the prefix directory, where `<prefix>` is the 'normal' prefix directory. Lastly the test prefix is created, containing an empty `Register.toml`.

>Note that the setup fails if the test prefix already exists, because this could mean that the 'normal' prefix was used instead.

After the setup is done, the test can be ran. A cleanup function removes everything which didn't exist before the test execution. So if the `TestConfig.toml` already existed it will not be deleted.
