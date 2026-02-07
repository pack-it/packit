pub struct InstallerOptions {
    pub build_source: bool,
    pub skip_symlinking: bool,
    pub skip_active: bool,
    pub keep_build: bool,
}

impl Default for InstallerOptions {
    fn default() -> Self {
        Self {
            build_source: false,
            skip_symlinking: false,
            skip_active: false,
            keep_build: false,
        }
    }
}

impl InstallerOptions {
    pub fn build_source(mut self, build_source: bool) -> Self {
        self.build_source = build_source;
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
