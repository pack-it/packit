use std::fs;

use crate::{config::Repository, repositories::error::Result};

pub trait RepositoryProvider {
    fn read_file(&self, path: String) -> Result<String>;
}

pub fn create_provider(repository: &Repository) -> Option<impl RepositoryProvider> {
    match repository.provider.as_str() {
        "fs" => Some(FileSystemProvider {
            path: repository.path.clone(),
        }),
        _ => None
    }
}

pub struct FileSystemProvider {
    path: String,
}

impl RepositoryProvider for FileSystemProvider {
    fn read_file(&self, path: String) -> Result<String> {
        Ok(fs::read_to_string(self.path.clone() + &path)?)
    }
}
