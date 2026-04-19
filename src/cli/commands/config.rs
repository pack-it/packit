// SPDX-License-Identifier: GPL-3.0-only
use clap::Subcommand;

use crate::{cli::commands::HandleCommand, config::Config, utils::unwrap_or_exit::UnwrapOrExit};

/// Manages the Packit config file.
#[derive(Subcommand, Debug)]
pub enum ConfigArgs {
    /// Shows the current configuration
    Show,
}

impl HandleCommand for ConfigArgs {
    fn handle(&self) {
        let config = Config::from(&Config::get_default_path()).unwrap_or_exit_msg("Cannot load config", 1);

        match self {
            ConfigArgs::Show => self.handle_show(config),
        }
    }
}

impl ConfigArgs {
    fn handle_show(&self, config: Config) {
        println!("Prefix directory: {}", config.prefix_directory.display());

        print!("Multiuser mode: ");
        if config.multiuser {
            println!("on");
        } else {
            println!("off");
        }

        println!("Repositories rank: {}", config.repositories_rank.join(", "));
    }
}
