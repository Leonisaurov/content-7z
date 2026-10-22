use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("config error: {0}")]
    Config(#[from] config::ConfigError),
    #[error("archive error: {0}")]
    Archive(String),
    #[error("not found: {0}")]
    NotFound(String),
}
pub type Result<T> = std::result::Result<T, Error>;
