# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).


## [Unreleased](https://github.com/pack-it/packit/compare/0.0.3...HEAD)


## [v0.0.3](https://github.com/pack-it/packit/compare/0.0.2...0.0.3) - 2026-07-11

### Added
- Support for loading non-text external test files.
- The `--active` flag for the list command, to list all active packages.
- Support for context (using our [`contextdiff-parser`](https://github.com/pack-it/contextdiff-parser)) and unified diff formats for patches.
- The update command now accepts multiple packages as input instead of just one.
- The `--all` flag for the update command flag to update all packages which are not up-to-date.
- The `--exclude` flag for the update command to exclude certain packages when using the `--all` flag.
- The `required_packit_version` field to describe the minimum required Packit version for a package in `repository.toml`, `package.toml` and `targets.toml` metadata files. (BREAKING)
- The `conflicts_with` fields to describe conflicting packages in the `package.toml` metadata file. Two conflicting packages cannot be symlinked at the same time. (BREAKING)
- The `size` field in the source fields in the `targets.toml` metadata file. (BREAKING)
- Deprecation information fields (deprecated_from, disabled_from and reason), to allow specifying deprecation and disabling dates of packages or specific versions. (BREAKING)
- The `--overwrite` flag to the link command to overwrite existing links from another package that are conflicting.
- The install script now asks for administrator privileges on Windows and shows a proper prompt on Unix.
- The Packit install scripts for Unix and Windows now cleanup the created Packit files in case of an error during installation.
- Support for a separate verbose output from scripts, file descriptor 3 can now be used for verbose output. (BREAKING)
- When a build script exits with a non-zero status code, the last 10 lines of the scripts output is now shown.
- The `disable_prebuilds` field to the config, which disables usage of prebuilds when set to true.
- The test script checks in the verifier, which use the metadata test scripts to test if packages work.
- The `--verbose` flag for the `search` command.

### Changes
- The build system now includes a new `package-build` xtask to create a full Packit prebuild for the release.
- The list command now uses column order grid printing instead of row order.
- The patch apply process now uses a more advanced file path resolver, which ensures better patch compatibility.
- The install tree is now fully expanded before installation, making it clearer what will be installed.
- Trees now perform a check for cycles, throwing an error if they detect one.
- Update all dependencies, remove unnecessary dependency features and ensure all dependencies support MSRV 1.88.0.
- When packages are specified for the `check` and `fix` commands, only those are checked when doing a package related check. Initial and general checks are now done as well in the case.
- Improve IOError messages by including information about the operation that failed.
- The `gnubin` directory of a package is now also symlinked into `<prefix>/gnubin`.
- The `check` and `fix` commands now accept `[<PACKAGE-NAME>[@<VERSION>] ...]` instead of just `[<PACKAGE-NAME>@<VERSION> ...]`.
- The `update` command now updates the latest install version, unless otherwise specified.
- The package metadata resolving algorithm is improved and now shows a clear reason when a package cannot be found.
- The `util checksum` command now also shows the size of the file.
- When the `repository.toml` file of a metadata repository cannot be fetched, the repository will not be loaded anymore.
- Improved UI with colors and text styling.
- The `search` command now outputs more and different information based on the given package input.
- The `info` command now shows conflicting packages when verbose is specified.
- Packit now has a MSRV of 1.88.0 (BREAKING)

### Fixed
- Fix package not found issue when multiple repositories have the same package but different versions.
- Fix `--updatables` flag listing older installed package versions when a newer up-to-date version is installed.
- Fix checksum check for patch files that are downloaded from the metadata repository.
- Fix empty directories (`lib`, `share`) in the prefix, by not creating them anymore.
- Fix skip active install option, by only skipping when multiple versions are installed.
- All warnings, errors and debug logs are now outputted to stderr instead of stdout.
- Fix `Permissions` verifier check, by allowing readonly flag on files.
- Fix `InvalidDependencies` verifier check, by also checking for target specific dependencies.


## [v0.0.2](https://github.com/pack-it/packit/compare/0.0.1...0.0.2) - 2026-05-14

### Added
- The config command, to view and change the config through the CLI.
- Support for using external files for test scripts in the package metadata.
- The fuzzy-search feature, to provide a package suggestion in case of a wrong package parameter.
- Support for leading zeros in version numbers.
- The web prebuild provider for web prebuild support.
- The `--updatables` flag on the list command, to list packages which can be updated.
- Support for Regex search in the search command, to search packages based on a given regex pattern.
- Support for applying patches to the source code of packages.
- Verifier and Repairer features
    - The check and fix command now accept multiple package parameters.
    - The inconsistent register fix doesn't require a re-install anymore.
    - Check after fix (re-run check to make sure the fix worked).
    - Checks and fixes for: stray directories, empty directories, package existence, correct permissions, missing Config.toml and missing Register.toml.
    - Checks for all register fields
- Skip uninstall option when update is not possible because of dependents which need the older version.
- Portable repositories, a generated repository containing only specific packages for use on airgapped systems.
- The init command which initializes the Packit environment and files.
- The `--structured` flag to the package command to structure packages into a prebuild directory structure.
- The `--all` flag to the package command to package all installed packages.

### Changes
- The build system now includes a new `build-install` xtask to create a full Packit build in a `bin` directory structure.
- The register file is renamed from `Installed.toml` to `Register.toml`. (BREAKING)
- Renamed the source repository fields in the register: (BREAKING) <br>
`source_repository_url` -> `metadata_repository_url` <br>
`source_repository_provider` -> `metadata_repository_provider` <br>
`source_prebuild_repository_url` -> `prebuilds_repository_url` <br>
`source_prebuild_repository_provider` -> `prebuilds_repository_provider`
- Renamed the `path` repository field in the config to `url`. (BREAKING)
- The list command now has grid printing.

### Removed
- The repositories command, this command is now integrated in the new config command.

### Fixed
- Fix repeated questions for building from source instead of using a prebuild.
- Fix the repairer fix for broken dependency trees. The missing dependency now gets the correct dependents and is set in the dependencies directory.


## [v0.0.1](https://github.com/pack-it/packit/releases/tag/0.0.1) - 2026-04-16

First release of Packit, consisting of the basic implementation of the universal package manager for macOS, Linux and Windows.
