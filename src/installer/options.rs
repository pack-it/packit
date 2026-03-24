use crate::installer::install_tree::InstallType;

/// Holds the install options.
pub struct InstallerOptions {
    pub install_type: InstallType,
    pub skip_symlinking: bool,
    pub skip_active: bool,
    pub keep_build: bool,
}

impl Default for InstallerOptions {
    /// Creates and returns `Self` with default settings.
    fn default() -> Self {
        Self {
            install_type: InstallType::Prebuild,
            skip_symlinking: false,
            skip_active: false,
            keep_build: false,
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
}
