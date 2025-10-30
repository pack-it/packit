use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    os,
    path::Path,
};

use thiserror::Error;

const MAX_FILE_SIZE: usize = 1000;

pub struct HistoryManager {
    latest_file: String,
    history_directory: String,
}

pub struct HistoryFile {
    file: String,
    pub elements: Vec<HistoryElement>,
}

#[derive(Clone)]
pub struct HistoryElement {
    pub timestamp: String,
    pub data: HistoryData,
}

#[derive(Clone)]
pub enum HistoryData {
    Previous(String),
    Next(String),
    Command(String),
    Install(String),
    Uninstall(String),
}

// Errors that occur when reading or writing the history file.
#[derive(Error, Debug)]
pub enum HistoryError {
    #[error("Cannot read or write history: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Cannot parse file index to int: {0}")]
    IntParseError(#[from] std::num::ParseIntError),

    #[error("Cannot parse history file, invalid element type '{element_type}'")]
    InvalidElementType {
        element_type: String,
    },

    #[error("Invalid file name")]
    InvalidFileName,
}

impl HistoryManager {
    pub fn new(config_dir: &str) -> Self {
        Self {
            latest_file: format!("{config_dir}/history.pit"),
            history_directory: format!("{config_dir}/history"),
        }
    }

    pub fn add_elements(&mut self, elements: Vec<HistoryElement>) -> Result<(), HistoryError> {
        let mut file = match fs::exists(&self.latest_file)? {
            true => HistoryFile::read_from(&self.latest_file)?,
            false => self.create_new_file()?,
        };

        // Create new file if the file exceeds the max file size
        if file.elements.len() >= MAX_FILE_SIZE {
            file = self.create_new_file()?;
        }

        file.add_elements(elements)?;

        Ok(())
    }

    fn create_new_file(&self) -> Result<HistoryFile, HistoryError> {
        let first_file = !fs::exists(&self.latest_file)?;

        let new_index = match first_file {
            true => 0,
            false => self.get_current_file_index()? + 1,
        };

        let new_file_name = format!("history.{new_index}.pit");
        let new_file_path = format!("{}/{new_file_name}", self.history_directory);
        let mut new_file = HistoryFile::new(&new_file_path);

        if !first_file {
            // Load previous file
            let previous_index = self.get_current_file_index()?;
            let previous_file_name = format!("history.{previous_index}.pit");
            let mut previous_file = HistoryFile::read_from(&format!("{}/{previous_file_name}", self.history_directory))?;

            // Add previous history file element to new file
            let previous_element = HistoryElement {
                timestamp: "TODO".into(),
                data: HistoryData::Previous(previous_file_name),
            };
            new_file.add_elements(vec![previous_element])?;

            // Add next history file element to previous file
            let next_element = HistoryElement {
                timestamp: "TODO".into(),
                data: HistoryData::Next(new_file_name.clone()),
            };
            previous_file.add_elements(vec![next_element])?;
        }

        // Create new symlink
        if fs::exists(&self.latest_file)? {
            fs::remove_file(&self.latest_file)?;
        }

        let original_path = format!("./history/{new_file_name}");

        #[cfg(target_family = "unix")]
        os::unix::fs::symlink(original_path, self.latest_file.clone())?;

        #[cfg(target_family = "windows")]
        os::windows::fs::symlink_file(original_path, self.latest_file.clone())?;

        Ok(new_file)
    }

    fn get_current_file_index(&self) -> Result<u64, HistoryError> {
        let latest_file = fs::read_link(&self.latest_file)?;
        let file_name = latest_file.file_name().ok_or(HistoryError::InvalidFileName)?.to_str().ok_or(HistoryError::InvalidFileName)?;

        let index = file_name.split('.').collect::<Vec<_>>()[1];

        Ok(index.parse()?)
    }

    pub fn get_latest_file(&self) -> Result<HistoryFile, HistoryError> {
        if !fs::exists(&self.latest_file)? {
            return Ok(self.create_new_file()?);
        }

        let current_index = self.get_current_file_index()?;
        let file_name = format!("{}/history.{current_index}.pit", self.history_directory);

        Ok(HistoryFile::read_from(&file_name)?)
    }
}

impl HistoryFile {
    fn new(file: &str) -> Self {
        Self {
            file: file.into(),
            elements: Vec::new(),
        }
    }

    pub fn read_from(file: &str) -> Result<Self, HistoryError> {
        // If the file does not exist, we return an empty storage
        if !fs::exists(file)? {
            return Ok(HistoryFile::new(file));
        }

        // Read data from file
        let file_content = fs::read_to_string(file)?;

        // If the file is empty, return an empty storage
        if file_content.trim().is_empty() {
            return Ok(HistoryFile::new(file));
        }

        let mut elements = Vec::new();

        for line in file_content.lines() {
            elements.push(line.try_into()?);
        }

        Ok(HistoryFile {
            file: file.into(),
            elements,
        })
    }

    fn add_elements(&mut self, elements: Vec<HistoryElement>) -> Result<(), HistoryError> {
        if !fs::exists(&self.file)? {
            // Create file parents
            if let Some(parent) = Path::new(&self.file).parent() {
                fs::create_dir_all(parent)?;
            }

            File::create(&self.file)?;
        }

        let mut file = OpenOptions::new().write(true).append(true).open(&self.file)?;

        for element in elements {
            writeln!(file, "{}", String::from(element.clone()))?;
            self.elements.push(element);
        }

        Ok(())
    }
}

impl TryFrom<&str> for HistoryElement {
    type Error = HistoryError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = value.split(" ").collect();
        let element_type = parts[0];
        let timestamp = parts[1];
        let element_data = parts[1..].join(" ");

        Ok(Self {
            timestamp: timestamp.into(),
            data: HistoryData::from_data(element_type, &element_data)?,
        })
    }
}

impl From<HistoryElement> for String {
    fn from(value: HistoryElement) -> Self {
        format!("{} {} {}", value.data.get_type(), value.timestamp, value.data.serialize_data())
    }
}

impl HistoryData {
    fn from_data(data_type: &str, data: &str) -> Result<Self, HistoryError> {
        Ok(match data_type {
            "PREVIOUS" => HistoryData::Previous(data.into()),
            "NEXT" => HistoryData::Next(data.into()),
            "COMMAND" => HistoryData::Command(data.into()),
            "INSTALL" => HistoryData::Install(data.into()),
            "UNINSTALL" => HistoryData::Uninstall(data.into()),
            _ => {
                return Err(HistoryError::InvalidElementType {
                    element_type: data_type.into(),
                })
            },
        })
    }

    fn get_type(&self) -> &str {
        match self {
            HistoryData::Previous(_) => "PREVIOUS",
            HistoryData::Next(_) => "NEXT",
            HistoryData::Command(_) => "COMMAND",
            HistoryData::Install(_) => "INSTALL",
            HistoryData::Uninstall(_) => "UNINSTALL",
        }
    }

    fn serialize_data(&self) -> &str {
        match self {
            HistoryData::Previous(data) => data,
            HistoryData::Next(data) => data,
            HistoryData::Command(data) => data,
            HistoryData::Install(data) => data,
            HistoryData::Uninstall(data) => data,
        }
    }
}
