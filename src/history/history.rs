// SPDX-License-Identifier: GPL-3.0-only
use crate::{
    cli::display::logging::{debug, warning},
    installer::types::{PackageId, PackageName, Version},
    utils::{
        constants::HISTORY_DIRECTORY_NAME,
        ioerror::{self, IOResultExt},
    },
};
use chrono::Utc;
use std::{
    fmt::Display,
    fs::{self, File, Metadata, OpenOptions},
    io::Write,
    os::unix::fs::MetadataExt,
    path::PathBuf,
};
use thiserror::Error;

const MAX_FILE_SIZE: u64 = 100 * 1024;

/// The errors that occur during history.
#[derive(Error, Debug)]
pub enum HistoryError {
    #[error("Error while interacting with filesystem")]
    IOError(#[from] ioerror::IOError),
}

pub(super) type Result<T> = std::result::Result<T, HistoryError>;

pub struct HistoryManager {
    directory: PathBuf,
    file: Option<File>,
    command: HistoryCommand,
    should_store: bool,
}

pub struct HistoryCommand {
    timestamp: i64,
    command: String,
    actions: Vec<Action>,
}

pub struct Action {
    timestamp: i64,
    action: ActionType,
}

// TODO: ConfigChange, Util commands: Portable repo and package commands
pub enum ActionType {
    Install(PackageId),
    Uninstall(PackageId),
    Link(PackageName),
    Unlink(PackageName),
    Switch {
        name: PackageName,
        old_version: Version,
        new_version: Version,
    },
    SwitchDependency {
        dependent: PackageId,
        dependency: PackageName,
        old_version: Version,
        new_version: Version,
    },
    Refresh(PackageId),
}

impl HistoryManager {
    pub fn start_command(directory: PathBuf, should_store: bool) -> Result<Self> {
        // Get the command as a string
        let args: Vec<String> = std::env::args().collect();
        let command_string = args.join(" ");
        let command = HistoryCommand::new(command_string);

        let mut manager = Self {
            directory,
            file: None,
            command,
            should_store,
        };

        manager.store_command()?;
        Ok(manager)
    }

    fn find_current_file(&self) -> Result<Option<(Metadata, u32)>> {
        let mut highest_metadata: Option<(Metadata, u32)> = None;

        if !self.directory.exists() {
            return Ok(highest_metadata);
        }

        for entry in fs::read_dir(&self.directory).err_with_path("read", &self.directory)? {
            let entry = entry.err_with_path("iterate", &self.directory)?;
            let metadata = entry.metadata().err_with_path("read metadata of", entry.path())?;

            if !metadata.is_file() {
                warning!("Non-file found in Packit history directory");
                continue;
            }

            // Get the file name
            let file_name = entry.file_name().to_ascii_lowercase();
            let Some(file_name) = file_name.to_str() else { continue };

            let Some((prefix, number)) = file_name.split_once('.') else {
                warning!("File with incorrect name '{file_name}' in Packit history directory");
                continue;
            };

            // Check the history file prefix
            if prefix != HISTORY_DIRECTORY_NAME {
                warning!("File with incorrect name '{file_name}' in Packit history directory");
                continue;
            }

            let Ok(number) = str::parse::<u32>(number) else {
                warning!("File with incorrect name '{file_name}' in Packit history directory");
                continue;
            };

            // Keep track of the highest number
            match &highest_metadata {
                Some(highest) if highest.1 > number => {},
                Some(_) | None => highest_metadata = Some((metadata, number)),
            }
        }

        Ok(highest_metadata)
    }

    fn create_history_file(&self, number: u32) -> Result<PathBuf> {
        // Make sure the history directory exists
        if !self.directory.exists() {
            fs::create_dir_all(&self.directory).err_with_path("create dirs", &self.directory)?;
        }

        let path = self.directory.join(format!("history.{number}"));
        _ = File::create(&path).err_with_path("create file", &path)?;
        Ok(path)
    }

    pub fn store_command(&mut self) -> Result<()> {
        self.write_history(&self.command.to_string())
    }

    pub fn end_command(&mut self) -> Result<()> {
        self.write_history(&format!("COMMAND END"))
    }

    pub fn store_action(&mut self, action: Action) -> Result<()> {
        // Store the action in the history file
        self.write_history(&action.to_string())?;

        self.command.actions.push(action);
        Ok(())
    }

    /// Writes the given content to the history file.
    /// Returns early if `should_store` is false.
    /// The history file is only created after this check and if it was not created already.
    fn write_history(&mut self, content: &String) -> Result<()> {
        if !self.should_store {
            debug!("Skipping history storage, because it is turned off");
            return Ok(());
        }

        // Only set the history file when necessary
        if self.file.is_none() {
            self.set_history_file()?;
        }

        if let Some(file) = &mut self.file {
            writeln!(file, "{content}").err_operation("write")?;
        }

        Ok(())
    }

    fn set_history_file(&mut self) -> Result<()> {
        // Get the current history file and check if it's under the size limit
        let path = match self.find_current_file()? {
            Some((metadata, number)) if metadata.size() > MAX_FILE_SIZE => self.create_history_file(number + 1)?,
            Some((_, number)) => self.directory.join(format!("history.{number}")),
            None => self.create_history_file(0)?,
        };

        let file = OpenOptions::new().append(true).open(&path).err_with_path("open", &path)?;
        self.file = Some(file);

        Ok(())
    }
}

impl HistoryCommand {
    pub fn new(command: String) -> Self {
        Self {
            timestamp: Utc::now().timestamp(),
            command,
            actions: Vec::new(),
        }
    }
}

impl Action {
    pub fn new(action: ActionType) -> Self {
        Self {
            timestamp: Utc::now().timestamp(),
            action,
        }
    }
}

impl Display for HistoryCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "COMMAND {}\n", self.timestamp)?;
        write!(f, "{}", self.command)
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ACTION {}\n{}", self.timestamp, self.action)
    }
}

impl Display for ActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            ActionType::Install(package_id) => write!(f, "INSTALL {package_id}"),
            ActionType::Uninstall(optional_id) => write!(f, "UNINSTALL {optional_id}"),
            ActionType::Link(package_name) => write!(f, "LINK {package_name}"),
            ActionType::Unlink(package_name) => write!(f, "UNLINK {package_name}"),
            ActionType::Switch {
                name,
                old_version,
                new_version,
            } => write!(f, "SWITCH {name} {old_version} {new_version}"),
            ActionType::SwitchDependency {
                dependent,
                dependency,
                old_version,
                new_version,
            } => write!(f, "SWITCH-DEPENDENCY {dependent} {dependency} {old_version} {new_version}"),
            ActionType::Refresh(package_id) => write!(f, "REFRESH {package_id}"),
        }
    }
}
