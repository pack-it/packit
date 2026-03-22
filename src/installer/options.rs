use crate::installer::install_tree::InstallTypes;

pub struct InstallerOptions {
    pub install_type: InstallTypes,
    pub skip_symlinking: bool,
    pub skip_active: bool,
    pub keep_build: bool,
}

impl Default for InstallerOptions {
    fn default() -> Self {
        Self {
            install_type: InstallTypes::Prebuild { is_dependency: false },
            skip_symlinking: false,
            skip_active: false,
            keep_build: false,
        }
    }
}

impl InstallerOptions {
    pub fn install_type(mut self, install_type: InstallTypes) -> Self {
        self.install_type = install_type;
        self
    }

    pub fn skip_symlinking(mut self, skip_symlinking: bool) -> Self {
        self.skip_symlinking = skip_symlinking;
        self
    }

    pub fn skip_active(mut self, skip_active: bool) -> Self {
        self.skip_active = skip_active;
        self
    }

    pub fn keep_build(mut self, keep_build: bool) -> Self {
        self.keep_build = keep_build;
        self
    }
}
