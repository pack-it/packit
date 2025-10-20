use std::path::PathBuf;
use clap::{Parser, Subcommand, Args};

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
}

#[derive(Args, Debug)]
struct ListArgs {
    /// Directory to list all packages of (OPTIONAL)
    directory: Option<PathBuf>,
}

// This executes a packit subcommand
pub fn execute() {
    let command = Cli::parse();

    match command.command {
        Commands::Install(args) => {
            install(args);
        }
        Commands::Uninstall(args) => {
            uninstall(args);
        }
        Commands::List(args) => {
            list_(args);
        }
    }
}

fn install(args: InstallArgs) {}

fn uninstall(args: UninstallArgs) {}

fn list_(args: ListArgs) {}

