//! Archive entries as reported by the listing tools.

use crate::path::normalize_archive_path;

/// A single entry of an archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveEntry {
    /// Path exactly as reported by the listing tool.
    pub path: String,
    /// Sanitized path components, relative to the root of the archive.
    pub components: Vec<String>,
    /// Whether the entry is a directory.
    pub is_dir: bool,
    /// Uncompressed size in bytes (`0` when unknown).
    pub size: u64,
    /// Whether the content of the entry is encrypted.
    pub encrypted: bool,
}

impl ArchiveEntry {
    /// Builds an entry, dropping the ones that cannot be used safely.
    ///
    /// Entries with a path that escapes the archive root (`..` components) or
    /// that are empty after normalization are rejected.
    pub fn new(path: &str, is_dir: bool, size: u64, encrypted: bool) -> Option<Self> {
        let normalized = normalize_archive_path(path)?;
        let components: Vec<String> = normalized
            .components()
            .filter_map(|component| component.as_os_str().to_str().map(str::to_string))
            .collect();

        if components.is_empty() {
            return None;
        }

        Some(Self {
            path: path.to_string(),
            components,
            is_dir,
            size,
            encrypted,
        })
    }

    /// Full path of the entry inside the archive.
    pub fn full_path(&self) -> String {
        self.components.join("/")
    }

    /// Name of the entry, without its parent directories.
    pub fn name(&self) -> &str {
        self.components
            .last()
            .map(String::as_str)
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_entries() {
        let entry = ArchiveEntry::new("dir/file.txt", false, 12, false).unwrap();
        assert_eq!(entry.components, vec!["dir", "file.txt"]);
        assert_eq!(entry.name(), "file.txt");
        assert_eq!(entry.full_path(), "dir/file.txt");
    }

    #[test]
    fn rejects_traversal_entries() {
        assert!(ArchiveEntry::new("../evil", false, 0, false).is_none());
    }

    #[test]
    fn rejects_empty_entries() {
        assert!(ArchiveEntry::new("/", true, 0, false).is_none());
    }
}
