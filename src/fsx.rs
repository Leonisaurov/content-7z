//! Filesystem helpers: temporary directories and safe copies.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::path::is_within;

/// A temporary directory that is removed when it goes out of scope.
#[derive(Debug)]
pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    /// Creates a new private temporary directory.
    pub fn new(prefix: &str) -> io::Result<Self> {
        let base = std::env::temp_dir();
        static COUNTER: AtomicU64 = AtomicU64::new(0);

        for _ in 0..32 {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|value| value.as_nanos())
                .unwrap_or(0);
            let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
            let candidate = base.join(format!(
                "{prefix}{}-{}-{}",
                std::process::id(),
                nanos,
                counter
            ));

            match create_private_dir(&candidate) {
                Ok(()) => return Ok(Self { path: candidate }),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }

        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "could not create a temporary directory",
        ))
    }

    /// Path of the temporary directory.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// Creates a directory that only the current user can access.
#[cfg(unix)]
fn create_private_dir(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::DirBuilderExt;
    fs::DirBuilder::new().mode(0o700).create(path)
}

/// Creates a directory that only the current user can access.
#[cfg(not(unix))]
fn create_private_dir(path: &Path) -> io::Result<()> {
    fs::create_dir(path)
}

/// Copies a directory tree, skipping anything that escapes `root`.
///
/// Symbolic links are only recreated when their target stays inside the tree,
/// which prevents an archive from writing outside of the destination through
/// a link.
pub fn copy_tree_safely(source: &Path, destination: &Path) -> io::Result<u64> {
    let mut copied = 0u64;
    copy_dir(source, destination, source, &mut copied)?;
    Ok(copied)
}

/// Recursively copies the content of `current` into `destination`.
fn copy_dir(current: &Path, destination: &Path, root: &Path, copied: &mut u64) -> io::Result<()> {
    fs::create_dir_all(destination)?;

    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)?;
        let target = destination.join(entry.file_name());

        if metadata.is_dir() {
            copy_dir(&path, &target, root, copied)?;
        } else if metadata.is_symlink() {
            let link = fs::read_link(&path)?;
            let resolved = path.parent().unwrap_or(root).join(&link);
            if is_within(root, &resolved) {
                let _ = fs::remove_file(&target);
                create_symlink(&link, &target).ok();
            }
        } else if metadata.is_file() {
            fs::copy(&path, &target)?;
            *copied += 1;
        }
    }

    Ok(())
}

/// Creates a symbolic link.
#[cfg(unix)]
fn create_symlink(original: &Path, link: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(original, link)
}

/// Creates a symbolic link.
#[cfg(not(unix))]
fn create_symlink(original: &Path, link: &Path) -> io::Result<()> {
    let _ = (original, link);
    Ok(())
}

/// Removes the symbolic links of a tree that point outside of it.
///
/// The extraction tools already refuse to write outside of the destination,
/// this is an extra safety net for the archives that contain links.
pub fn remove_escaping_links(root: &Path) -> io::Result<()> {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => return Ok(()),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)?;

        if metadata.is_dir() {
            remove_escaping_links(&path)?;
        } else if metadata.is_symlink() {
            let link = fs::read_link(&path)?;
            let resolved = path.parent().unwrap_or(root).join(&link);
            if !is_within(root, &resolved) {
                let _ = fs::remove_file(&path);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temporary_directories_are_removed() {
        let path = {
            let temp = TempDir::new("content-7z-test-").unwrap();
            assert!(temp.path().exists());
            temp.path().to_path_buf()
        };
        assert!(!path.exists());
    }

    #[test]
    fn copies_trees() {
        let source = TempDir::new("content-7z-src-").unwrap();
        let destination = TempDir::new("content-7z-dst-").unwrap();

        fs::create_dir_all(source.path().join("dir")).unwrap();
        fs::write(source.path().join("dir/file.txt"), b"hello").unwrap();

        let copied = copy_tree_safely(source.path(), destination.path()).unwrap();
        assert_eq!(copied, 1);
        assert_eq!(
            fs::read_to_string(destination.path().join("dir/file.txt")).unwrap(),
            "hello"
        );
    }

    #[test]
    fn skips_the_links_that_escape_the_tree() {
        let source = TempDir::new("content-7z-src-").unwrap();
        let destination = TempDir::new("content-7z-dst-").unwrap();

        fs::write(source.path().join("file.txt"), b"hello").unwrap();
        std::os::unix::fs::symlink("/etc/passwd", source.path().join("escape")).unwrap();
        std::os::unix::fs::symlink("file.txt", source.path().join("inside")).unwrap();

        copy_tree_safely(source.path(), destination.path()).unwrap();

        assert!(!destination.path().join("escape").exists());
        assert!(destination.path().join("inside").exists());
    }
}
