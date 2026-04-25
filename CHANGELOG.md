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

### Removed
- The repositories command, this command is now integrated in the new config command.

### Fixed
- Fix repeated questions for building from source instead of using a prebuild.


## [v0.0.1](https://github.com/pack-it/packit/releases/tag/0.0.1) - 2026-04-16

First release of Packit, consisting of the basic implementation of the universal package manager for macOS, Linux and Windows.
