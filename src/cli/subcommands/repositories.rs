use crate::{cli, config::Config, repositories::manager::RepositoryManager};
use colored::Colorize;

/// Handles the repositories command, listing all configured repositories.
pub fn handle_repositories(config: &Config, manager: &RepositoryManager) {
    let mut first = true;

    for (repository_id, repository) in &config.repositories {
        if !first {
            println!();
        }
        first = false;

        // Read metadata of repository
        let metadata = match manager.read_repository_metadata(&repository_id) {
            Ok(metadata) => metadata,
            Err(e) => {
                // Display the error and continue
                cli::display_warning(&format!("Cannot read repository metadata of repository '{repository_id}'"));
                cli::display_warning(&format!("{e}"));
                continue;
            },
        };

        // Print repository information
        println!("{} ({repository_id})", metadata.name.bold().blue());
        println!("{}", metadata.description.green());
        println!("Maintainers: {}", metadata.maintainers.join(", "));
        println!("Repository provider: {}, path: {}", repository.provider, repository.path);
    }
}
