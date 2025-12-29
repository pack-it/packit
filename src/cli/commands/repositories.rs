use crate::{
    cli::{self, commands::HandleCommand, error::CommandError},
    config::Config,
    repositories::manager::RepositoryManager,
};
use clap::Args;
use colored::Colorize;

#[derive(Args, Debug)]
pub struct RepositoryArgs {}

impl HandleCommand for RepositoryArgs {
    /// Handles the repositories command, listing all configured repositories.
    fn handle(&self, config: &Config, manager: &RepositoryManager) -> Result<(), CommandError> {
        for (index, (repository_id, repository)) in config.repositories.iter().enumerate() {
            if index != 0 {
                println!();
            }

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

        Ok(())
    }
}
