# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).


## [Unreleased](https://github.com/pack-it/packit/compare/0.0.2...HEAD)

### Added
- Support for loading non-text external test files.
- The `--active` flag for the list command, to list all active packages.
- Support for context (using our [`contextdiff-parser`](https://github.com/pack-it/contextdiff-parser)) and unified diff formats for patches.

### Changes
- The build system now includes a new `package-build` xtask to create a full Packit prebuild for the release.
- The list command now uses column order grid printing instead of row order.
- The patch apply process now uses a more advanced file path resolver, which ensures better patch compatibility.
- The install tree is now fully expanded before installation, making it clearer what will be installed.
- Trees now perform a check for cycles, throwing an error if they detect one.
- Update all dependencies, remove unnecessary dependency features and ensure all dependencies support MSRV 1.85.

### Fixed
- Fix package not found issue when multiple repositories have the same package but different versions.
- Fix `--updatables` flag listing older installed package versions when a newer up-to-date version is installed.
- Fix checksum check for patch files that are downloaded from the metadata repository.


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
