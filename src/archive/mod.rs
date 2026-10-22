//! Archive inspection and extraction.

pub mod entry;
pub mod kind;
pub mod listing;
pub mod manager;

pub use entry::ArchiveEntry;
pub use kind::ArchiveKind;
pub use manager::Archive;
