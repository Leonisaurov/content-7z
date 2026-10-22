//! Safe handling of paths that come from untrusted archives.
//!
//! Archive entries are attacker controlled data: they may contain absolute
//! paths, `..` components, windows separators or drive letters. Every path
//! that is going to be used to touch the filesystem goes through this module
//! first.

use std::path::{Component, Path, PathBuf};

/// Normalizes an archive entry path into a safe relative path.
///
/// * Windows separators are converted to `/`.
/// * Empty and `.` components are dropped.
/// * A leading `/` (or a windows drive prefix) is stripped, matching what
///   `tar`, `unzip` and `7z` do when they extract.
/// * `..` components make the whole entry unusable and yield `None`.
/// * Entries that are empty after normalization yield `None`.
pub fn normalize_archive_path(raw: &str) -> Option<PathBuf> {
    let mut unified = raw.replace('\\', "/");

    // Drop a windows drive prefix such as `C:`.
    let bytes = unified.as_bytes();
    if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        unified = unified[2..].to_string();
    }

    let mut out = PathBuf::new();
    for part in unified.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." {
            return None;
        }
        if part.contains('\0') {
            return None;
        }
        out.push(part);
    }

    if out.as_os_str().is_empty() {
        None
    } else {
        Some(out)
    }
}

/// Joins `relative` onto `base`, guaranteeing the result stays inside `base`.
///
/// Returns `None` when the relative path tries to escape the base directory.
pub fn join_within(base: &Path, relative: &Path) -> Option<PathBuf> {
    let mut out = base.to_path_buf();
    for component in relative.components() {
        match component {
            Component::Normal(part) => out.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    Some(out)
}

/// Returns `true` when `candidate` is `base` itself or lives inside of it.
///
/// Both paths are compared lexically, after being normalized.
pub fn is_within(base: &Path, candidate: &Path) -> bool {
    let base = normalize_for_compare(base);
    let candidate = normalize_for_compare(candidate);
    candidate.starts_with(&base)
}

/// Normalizes a path lexically (without touching the filesystem).
fn normalize_for_compare(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_regular_paths() {
        assert_eq!(
            normalize_archive_path("folder/file.txt"),
            Some(PathBuf::from("folder/file.txt"))
        );
    }

    #[test]
    fn strips_leading_slash_and_dot_segments() {
        assert_eq!(
            normalize_archive_path("/etc/passwd"),
            Some(PathBuf::from("etc/passwd"))
        );
        assert_eq!(
            normalize_archive_path("./a//b/./c"),
            Some(PathBuf::from("a/b/c"))
        );
    }

    #[test]
    fn converts_windows_separators() {
        assert_eq!(
            normalize_archive_path("dir\\file.txt"),
            Some(PathBuf::from("dir/file.txt"))
        );
    }

    #[test]
    fn strips_windows_drive_prefix() {
        assert_eq!(
            normalize_archive_path("C:/Windows/file.txt"),
            Some(PathBuf::from("Windows/file.txt"))
        );
    }

    #[test]
    fn rejects_traversal() {
        assert_eq!(normalize_archive_path("../etc/passwd"), None);
        assert_eq!(normalize_archive_path("a/../../b"), None);
        assert_eq!(normalize_archive_path(".."), None);
    }

    #[test]
    fn rejects_empty_entries() {
        assert_eq!(normalize_archive_path(""), None);
        assert_eq!(normalize_archive_path("/"), None);
        assert_eq!(normalize_archive_path("./"), None);
    }

    #[test]
    fn join_within_blocks_escapes() {
        let base = Path::new("/tmp/base");
        assert!(join_within(base, Path::new("a/b")).is_some());
        assert!(join_within(base, Path::new("../b")).is_none());
        assert!(join_within(base, Path::new("/etc/passwd")).is_none());
    }

    #[test]
    fn is_within_detects_escapes() {
        assert!(is_within(
            Path::new("/tmp/base"),
            Path::new("/tmp/base/file")
        ));
        assert!(!is_within(
            Path::new("/tmp/base"),
            Path::new("/tmp/other/file")
        ));
    }
}
