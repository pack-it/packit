# Tests

Packit has both unit and integration tests. These tests are separated, because the integration tests require setup and clean up functionality.

- [Unit tests](#unit-tests)
- [Integration tests](#integration-tests)

## Unit tests
Run the following command to run the unit tests:
```
cargo test
```

## Integration tests
Packits integration tests make use of `integration-tests` feature flag, to manage the integration test environment. The following shortcut can be used:
```
cargo xtest
```

Each integration test creates a new test environment. The (temporary) test directory is passed to packit with the `--config-path` flag. When the test is done the test directory will get out of scope and be dropped, which will trigger the cleanup function.

To use a custom config the `TestConfig.toml` can be used. If this file is present in the prefix directory it will be used for the `url` and `provider` type. Other fields are ignored.

The environment supports parallel testing.
