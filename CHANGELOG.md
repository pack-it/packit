# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).


## [Unreleased](https://github.com/pack-it/packit/compare/0.0.1...HEAD)

### Added
- The config command, to view and change the config through the CLI.
- Support for using external files for test scripts in the package metadata.
- The fuzzy-search feature, to provide a package suggestion in case of a wrong package parameter.
- Add support for leading zeros in version numbers.
- Add web prebuild provider for web prebuild support.
- The `--updatables` flag on the list command, to list packages which can be updated.
- The list command now has grid printing.
- The regex search, to search packages based on a given regex pattern.
- Add support for applying patches to the source code of packages.
- Verifier and Repairer features
    - The check and fix command now accept multiple package parameters.
    - The inconsistent register fix doesn't require a re-install anymore.
    - Add check after fix (re-run check to make sure the fix worked).
    - Add checks and fixes for: stray directories, empty directories, package existence, correct permissions, missing Config.toml and missing Installed.toml.
- Skip uninstall option when update is not possible because of dependents which need the older version.

### Changes
- The build system now includes a new `build-install` xtask to create a full Packit build in a `bin` directory structure.

### Removed
- The repositories command, this command is now integrated in the new config command.

### Fixed
- Fix repeated questions for building from source instead of using a prebuild.


## [v0.0.1](https://github.com/pack-it/packit/releases/tag/0.0.1) - 2026-04-16

First release of Packit, consisting of the basic implementation of the universal package manager for macOS, Linux and Windows.
