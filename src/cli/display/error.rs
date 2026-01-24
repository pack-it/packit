use thiserror::Error;

/// The errors that occur during display.
#[derive(Error, Debug)]
pub enum DisplayError {
    #[error("Could not display\nCaused by: {0}")]
    IOError(#[from] std::io::Error),
}
