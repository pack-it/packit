use crate::{
    cli::{commands::HandleCommand, display::logging::warning},
    config::Config,
    repositories::manager::RepositoryManager,
    utils::unwrap_or_exit::UnwrapOrExit,
};
use clap::Args;
use colored::Colorize;

/// Lists all configured repositories.
#[derive(Args, Debug)]
pub struct RepositoryArgs;

impl HandleCommand for RepositoryArgs {
    /// Handles the repositories command, listing all configured repositories.
    fn handle(&self) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);
        let manager = RepositoryManager::new(&config);
        for (index, (repository_id, repository)) in config.repositories.iter().enumerate() {
            if index != 0 {
                println!();
            }

            // Read metadata of repository
            let metadata = match manager.read_repository_metadata(&repository_id) {
                Ok(metadata) => metadata,
                Err(e) => {
                    // Display the error and continue
                    warning!("Cannot read repository metadata of repository '{repository_id}'");
                    warning!("{e}");
                    continue;
                },
            };

            // Print repository information
            println!("{} ({repository_id})", metadata.name.bold().blue());
            println!("{}", metadata.description.green());
            println!("License: {}", metadata.license);
            println!("Maintainers: {}", metadata.maintainers.join(", "));
            println!("Repository provider: {}, path: {}", repository.provider, repository.path);
        }
    }
}
