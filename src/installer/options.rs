// SPDX-License-Identifier: GPL-3.0-only
use crate::installer::install_tree::InstallType;

/// Holds the install options.
pub struct InstallerOptions {
    pub install_type: InstallType,
    pub skip_symlinking: bool,
    pub skip_active: bool,
    pub keep_build: bool,
    pub verbose: bool,
    pub skip_test: bool,
    pub include_build_test: bool,
    pub pause_build: bool,
}

impl Default for InstallerOptions {
    /// Creates a default `InstallerOptions` instance.
    fn default() -> Self {
        Self {
            install_type: InstallType::Prebuild,
            skip_symlinking: false,
            skip_active: false,
            keep_build: false,
            verbose: false,
            skip_test: false,
            include_build_test: false,
            pause_build: false,
        }
    }
}

impl InstallerOptions {
    /// Sets the install type.
    pub fn install_type(mut self, install_type: InstallType) -> Self {
        self.install_type = install_type;
        self
    }

    /// Sets the skip symlinking field.
    pub fn skip_symlinking(mut self, skip_symlinking: bool) -> Self {
        self.skip_symlinking = skip_symlinking;
        self
    }

    /// Sets the skip active field.
    pub fn skip_active(mut self, skip_active: bool) -> Self {
        self.skip_active = skip_active;
        self
    }

    /// Sets the keep build field.
    pub fn keep_build(mut self, keep_build: bool) -> Self {
        self.keep_build = keep_build;
        self
    }

    /// Sets the `verbose` field.
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Sets the `skip_test` field.
    pub fn skip_test(mut self, skip: bool) -> Self {
        self.skip_test = skip;
        self
    }

    /// Sets the `include_build_test` field.
    pub fn include_build_test(mut self, include: bool) -> Self {
        self.include_build_test = include;
        self
    }

    /// Sets the `pause_build` field.
    pub fn pause_build(mut self, pause: bool) -> Self {
        self.pause_build = pause;
        self
    }
}
