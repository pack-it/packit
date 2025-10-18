use std::path::PathBuf;
use clap::{Parser, Subcommand, Args};

#[derive(Parser, Debug)]
#[command(name = "Packit", version, about = "Universal package manager", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Install(InstallArgs),
    Uninstall(UninstallArgs),
    List(ListArgs),
}

#[derive(Args, Debug)]
struct InstallArgs {
    package_name: String,

    #[arg(short, long)]
    version: Option<String>
}

#[derive(Args, Debug)]
struct UninstallArgs{
    package_name: String,
}

#[derive(Args, Debug)]
struct ListArgs {
    directory: Option<PathBuf>,
}
