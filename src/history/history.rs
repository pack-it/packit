// SPDX-License-Identifier: GPL-3.0-only
use crate::{
    cli::display::logging::debug,
    installer::types::{PackageId, PackageName, Version},
};
use chrono::Utc;
use std::{fmt::Display, fs::OpenOptions, io::Write, path::PathBuf};

pub struct HistoryManager {
    directory: PathBuf,
    max_file: u64,
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
    Update {
        name: PackageName,
        old_version: Version,
        new_version: Version,
    },
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
    Refresh(PackageId), // TODO: Maybe optional id?
}

impl HistoryManager {
    pub fn new(directory: PathBuf, max_file: u64, command: HistoryCommand, should_store: bool) -> Self {
        Self {
            directory,
            max_file,
            command,
            should_store,
        }
    }

    pub fn store_command(&self) {
        self.write_history(&self.command.to_string());
    }

    pub fn store_action(&mut self, action: Action) {
        // Store the action in the history file
        self.write_history(&action.to_string());

        self.command.actions.push(action);
    }

    // TODO: Implement proper storage
    fn write_history(&self, content: &String) {
        if !self.should_store {
            debug!("Skipping history storage, because it is turned off");
            return;
        }

        let mut file = OpenOptions::new().append(true).create(true).open(&self.directory).unwrap();
        writeln!(file, "{content}").unwrap();
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
            ActionType::Uninstall(package_id) => write!(f, "UNINSTALL {package_id}"),
            ActionType::Update {
                name,
                old_version,
                new_version,
            } => write!(f, "UPDATE {name} {old_version} {new_version}"),
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
