// SPDX-License-Identifier: GPL-3.0-only
mod check;
mod fix;
mod info;
mod install;
mod link;
mod list;
mod package;
mod repositories;
mod search;
mod switch;
mod uninstall;
mod unlink;
mod update;
mod util;

use std::process::exit;

use clap::{Parser, Subcommand, builder::Styles};

#[cfg(target_os = "windows")]
use crate::cli::display::logging::debug;
use crate::cli::{
    commands::{
        check::CheckArgs, fix::FixArgs, info::InfoArgs, install::InstallArgs, link::LinkArgs, list::ListArgs, package::PackageArgs,
        repositories::RepositoryArgs, search::SearchArgs, switch::SwitchArgs, uninstall::UninstallArgs, unlink::UnlinkArgs,
        update::UpdateArgs, util::UtilArgs,
    },
    display::logging::error,
};

/// Represents the CLI interface with clap.
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

    /// Check the installation and only show the issues
    Check(CheckArgs),

    /// Check the installation and fix issues
    Fix(FixArgs),

    /// Package a package version
    Package(PackageArgs),

    /// Get info from a specific package
    Info(InfoArgs),

    /// Update an installed package
    Update(UpdateArgs),

    /// Several utils for advanced users
    #[clap(subcommand)]
    Util(UtilArgs),
}

impl Cli {
    /// Gets an instance of the `Cli`.
    pub fn get_instance() -> Self {
        // Try to enable ANSI support for Windows
        #[cfg(target_os = "windows")]
        {
            match enable_ansi_support::enable_ansi_support() {
                Ok(_) => debug!("Enabled ANSI support for windows!"),
                Err(_) => {
                    debug!("Cannot enable ANSI support for windows, disabling coloring!");
                    colored::control::set_override(false);
                },
            }
        }

        // Parse command
        match Cli::try_parse() {
            Ok(cli) => cli,
            Err(e) => {
                let styles = Styles::default();
                let prefix = format!("{}error:{:#} ", styles.get_error(), styles.get_error());

                let msg = match colored::control::SHOULD_COLORIZE.should_colorize() {
                    true => e.render().ansi().to_string(),
                    false => e.render().to_string(),
                };

                match msg.strip_prefix(&prefix) {
                    Some(msg) => error!(msg: msg),
                    None => println!("{msg}"),
                };

                exit(e.exit_code())
            },
        }
    }

    /// Reads and handles the command.
    pub fn handle_command(&self) {
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
            Commands::Check(args) => args,
            Commands::Fix(args) => args,
            Commands::Package(args) => args,
            Commands::Info(args) => args,
            Commands::Update(args) => args,
            Commands::Util(args) => args,
        };

        args.handle();
    }
}

trait HandleCommand {
    fn handle(&self);
}
