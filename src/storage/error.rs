use thiserror::Error;

/// The errors that occur when reading or saving the register file.
#[derive(Error, Debug)]
pub enum InstalledPackagesError {
    #[error("Cannot read or write installed packages file")]
    IOError(#[from] std::io::Error),

    #[error("Cannot parse installed packages file")]
    ParseError(#[from] toml::de::Error),

    #[error("Cannot serialize installed packages")]
    SerializeError(#[from] toml::ser::Error),
}

// TODO: Use Result<T> here as well
