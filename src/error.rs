use std::fmt;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Config(String),
    Archive(String),
    NotFound(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "io error: {}", e),
            Error::Config(e) => write!(f, "config error: {}", e),
            Error::Archive(e) => write!(f, "archive error: {}", e),
            Error::NotFound(e) => write!(f, "not found: {}", e),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
