//! Error types shared by every module of the application.

use std::fmt;
use std::io;

/// Convenient result alias used across the crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Everything that can go wrong while inspecting or extracting an archive.
#[derive(Debug)]
pub enum Error {
    /// An I/O operation failed.
    Io(io::Error),
    /// The configuration file could not be read or is invalid.
    Config(String),
    /// The archive could not be read by the backing tool.
    Archive(String),
    /// The tool required to handle this archive is not installed.
    ToolMissing(String),
    /// The archive is encrypted and no password was supplied.
    PasswordRequired,
    /// The supplied password did not unlock the archive.
    WrongPassword,
    /// The requested file or directory does not exist.
    NotFound(String),
    /// The operation is not supported for this kind of archive.
    Unsupported(String),
    /// The user aborted the operation.
    Cancelled,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "i/o error: {e}"),
            Error::Config(e) => write!(f, "configuration error: {e}"),
            Error::Archive(e) => write!(f, "archive error: {e}"),
            Error::ToolMissing(tool) => {
                write!(f, "the '{tool}' command is required but was not found")
            }
            Error::PasswordRequired => {
                write!(f, "the archive is encrypted, a password is required")
            }
            Error::WrongPassword => write!(f, "the password provided is not the correct one"),
            Error::NotFound(e) => write!(f, "not found: {e}"),
            Error::Unsupported(e) => write!(f, "unsupported operation: {e}"),
            Error::Cancelled => write!(f, "operation cancelled"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::Io(value)
    }
}
