use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

use crate::{
    config::Config,
    installed_packages::InstalledPackageStorage,
    installer::{error::InstallerError, installer::Installer},
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
}

#[derive(Args, Debug)]
struct InstallArgs {
    /// The name of the package to install
    package_name: String,

    /// The version of the package to install
    #[arg(short, long)]
    version: Option<String>,
}

#[derive(Args, Debug)]
struct UninstallArgs {
    /// The name of the package to uninstall
    package_name: String,

    /// The version of the package to uninstall
    #[arg(short, long)]
    version: Option<String>,
}

#[derive(Args, Debug)]
struct ListArgs {
    /// Directory to list all packages of (OPTIONAL)
    directory: Option<PathBuf>,

    /// Flag to indicate a full check (actually check packit install directory)
    #[arg(short, long)]
    use_dir: bool,
}

/// Reads and handles the command.
pub fn handle_command(manager: &RepositoryManager, config: &Config) -> Result<(), InstallerError> {
    let info_directory = config.install_directory.to_string() + "/info.toml";
    let mut installed_storage = InstalledPackageStorage::from(&info_directory)?;
    let command = Cli::parse();

    match command.command {
        Commands::Install(args) => {
            // Handle the install command with user specified arguments
            let mut installer = Installer::new(&config, &mut installed_storage);
            installer.install(manager, &args.package_name, args.version)?;
        },
        Commands::Uninstall(args) => {
            // Handle the uninstall command with user specified arguments
            let mut installer = Installer::new(&config, &mut installed_storage);
            installer.uninstall(&args.package_name, args.version)?;
        },
        Commands::List(args) => {
            handle_list(args)?;
        },
    }

    // Save changes
    installed_storage.save_to(&info_directory)?;

    Ok(())
}

/// Handles the list command with user specified arguments.
fn handle_list(args: ListArgs) -> Result<(), InstallerError> {
    todo!()
}
