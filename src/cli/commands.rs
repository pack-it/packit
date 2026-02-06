mod fix;
mod install;
mod link;
mod list;
mod repositories;
mod search;
mod switch;
mod uninstall;
mod unlink;

use std::process::exit;

use clap::{builder::Styles, Parser, Subcommand};

use crate::{
    cli::{
        commands::{
            fix::FixArgs, install::InstallArgs, link::LinkArgs, list::ListArgs, repositories::RepositoryArgs, search::SearchArgs,
            switch::SwitchArgs, uninstall::UninstallArgs, unlink::UnlinkArgs,
        },
        display::logging::error,
    },
    config::Config,
    repositories::manager::RepositoryManager,
};

#[derive(Parser, Debug)]
#[command(name = "Packit", version, about)]
#[command(long_about = "The universal package manager, designed to streamline the experience of installing packages on your system.")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Install a package on your system
    Install(InstallArgs),

    /// Uninstall a package from your system
    Uninstall(UninstallArgs),

    /// List all installed packages
    List(ListArgs),

    /// List all configured repositories
    Repositories(RepositoryArgs),

    /// Search a certain package
    Search(SearchArgs),

    /// Switch the active version of a package
    Switch(SwitchArgs),

    /// Create symlinks for a certain package
    Link(LinkArgs),

    /// Remove symlinks for a certain package
    Unlink(UnlinkArgs),

    /// Check the installation and fix problems
    Fix(FixArgs),
}

impl Cli {
    pub fn get_instance() -> Self {
        match Cli::try_parse() {
            Ok(cli) => cli,
            Err(e) => {
                let styles = Styles::default();
                let prefix = format!("{}error:{:#} ", styles.get_error(), styles.get_error());

                let msg = e.render().ansi().to_string();
                match msg.strip_prefix(&prefix) {
                    Some(msg) => error!(msg: msg),
                    None => println!("{msg}"),
                };

                exit(e.exit_code())
            },
        }
    }

    /// Reads and handles the command.
    pub fn handle_command(&self, manager: &RepositoryManager, config: &Config) {
        // Handle commands with user specified arguments
        let args: &dyn HandleCommand = match &self.command {
            Commands::Install(args) => args,
            Commands::Uninstall(args) => args,
            Commands::List(args) => args,
            Commands::Repositories(args) => args,
            Commands::Search(args) => args,
            Commands::Switch(args) => args,
            Commands::Link(args) => args,
            Commands::Unlink(args) => args,
            Commands::Fix(args) => args,
        };

        args.handle(config, manager);
    }
}

trait HandleCommand {
    fn handle(&self, config: &Config, manager: &RepositoryManager);
}
