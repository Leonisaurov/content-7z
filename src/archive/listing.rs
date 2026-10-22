//! Parsers for the output of the listing tools.
//!
//! Every parser is a pure function over the textual output of a tool, which
//! makes them easy to test without touching the filesystem.

use crate::archive::entry::ArchiveEntry;

/// Parses the technical listing of `7z` (`7z l -slt -ba`).
///
/// The output is a sequence of blocks of `key = value` lines separated by
/// empty lines. Unknown keys and stray lines (such as the password prompt of
/// `7z`) are ignored.
pub fn parse_sevenzip_slt(output: &str) -> Vec<ArchiveEntry> {
    let mut entries = Vec::new();

    for block in blocks(output) {
        let mut path: Option<&str> = None;
        let mut size: u64 = 0;
        let mut is_dir = false;
        let mut encrypted = false;

        for line in block {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let value = value.trim();

            match key.trim() {
                "Path" => path = Some(value),
                "Size" => size = value.parse::<u64>().unwrap_or(0),
                "Folder" => is_dir = value == "+",
                "Encrypted" => encrypted = value == "+",
                "Attributes" if value.starts_with('D') => {
                    is_dir = true;
                }
                _ => {}
            }
        }

        let Some(path) = path else {
            continue;
        };

        if path.ends_with('/') {
            is_dir = true;
        }

        if let Some(entry) = ArchiveEntry::new(path, is_dir, size, encrypted) {
            entries.push(entry);
        }
    }

    entries
}

/// Parses the verbose listing of `tar` (`tar -tvf`).
///
/// Both the GNU and the BSD layouts are supported; the size is left at zero
/// when it cannot be determined.
pub fn parse_tar_verbose(output: &str) -> Vec<ArchiveEntry> {
    let mut entries = Vec::new();

    for line in output.lines() {
        let line = line.trim_end();
        if line.trim().is_empty() {
            continue;
        }

        let Some((mode, size, name)) = split_tar_line(line) else {
            continue;
        };

        let is_dir = mode.starts_with('d') || name.ends_with('/');
        let name = name.trim_end_matches('/');

        if let Some(entry) = ArchiveEntry::new(name, is_dir, size, false) {
            entries.push(entry);
        }
    }

    entries
}

/// Parses a plain list of names (`tar -tf`, `unzip -Z1`).
///
/// Directories are detected through their trailing slash.
pub fn parse_name_list(output: &str) -> Vec<ArchiveEntry> {
    let mut entries = Vec::new();

    for line in output.lines() {
        let name = line.trim_end_matches('\r');
        if name.trim().is_empty() {
            continue;
        }

        let is_dir = name.ends_with('/');
        let name = name.trim_end_matches('/');

        if let Some(entry) = ArchiveEntry::new(name, is_dir, 0, false) {
            entries.push(entry);
        }
    }

    entries
}

/// Splits the output into blocks of non empty lines.
fn blocks(output: &str) -> Vec<Vec<&str>> {
    let mut result: Vec<Vec<&str>> = Vec::new();
    let mut current: Vec<&str> = Vec::new();

    for line in output.lines() {
        let line = line.trim_end();
        if line.trim().is_empty() {
            if !current.is_empty() {
                result.push(std::mem::take(&mut current));
            }
        } else {
            current.push(line);
        }
    }

    if !current.is_empty() {
        result.push(current);
    }

    result
}

/// Extracts the mode, the size and the name of a `tar -tvf` line.
fn split_tar_line(line: &str) -> Option<(&str, u64, &str)> {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    if tokens.len() < 3 {
        return None;
    }

    let mode = tokens[0];

    // The time is the only token containing a colon.
    let time_index = tokens.iter().position(|token| token.contains(':'))?;
    if time_index < 2 {
        return None;
    }

    // GNU layout: `mode owner size date time name`.
    // BSD layout: `mode links owner group size month day time name`.
    let size_index = if tokens[time_index - 1].contains('-') {
        time_index - 2
    } else {
        time_index.checked_sub(3)?
    };

    let size = tokens.get(size_index)?.parse::<u64>().unwrap_or(0);

    // The name is everything that follows the time token.
    let offset = line.find(tokens[time_index])? + tokens[time_index].len();
    let name = line[offset..].trim_start();
    // Symbolic links are listed as `name -> target`.
    let name = name.split(" -> ").next().unwrap_or(name);

    Some((mode, size, name))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SLT: &str = "Path = sub\n\
                        Size = 0\n\
                        Attributes = D drwxr-xr-x\n\
                        Encrypted = -\n\
                        \n\
                        Path = a.txt\n\
                        Size = 6\n\
                        Attributes = A -rw-r--r--\n\
                        Encrypted = +\n";

    #[test]
    fn parses_technical_listing() {
        let entries = parse_sevenzip_slt(SLT);
        assert_eq!(entries.len(), 2);

        assert_eq!(entries[0].components, vec!["sub"]);
        assert!(entries[0].is_dir);

        assert_eq!(entries[1].components, vec!["a.txt"]);
        assert!(!entries[1].is_dir);
        assert_eq!(entries[1].size, 6);
        assert!(entries[1].encrypted);
    }

    #[test]
    fn ignores_stray_lines() {
        let output = "Enter password (will not be echoed):\n\nPath = a.txt\nSize = 1\n";
        let entries = parse_sevenzip_slt(output);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name(), "a.txt");
    }

    #[test]
    fn parses_gnu_tar_listing() {
        let output = "-rw-r--r-- root/root         6 2026-10-22 02:44 a.txt\n\
                      drwxr-xr-x root/root         0 2026-10-22 02:44 sub/\n\
                      -rw-r--r-- root/root        11 2026-10-22 02:44 sub/b.txt\n";
        let entries = parse_tar_verbose(output);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].size, 6);
        assert!(!entries[0].is_dir);
        assert!(entries[1].is_dir);
        assert_eq!(entries[2].components, vec!["sub", "b.txt"]);
        assert_eq!(entries[2].size, 11);
    }

    #[test]
    fn parses_bsd_tar_listing() {
        let output = "-rw-r--r--  0 root   root       6 Jan  1 00:00 a.txt\n\
                      drwxr-xr-x  0 root   root       0 Jan  1 00:00 sub\n";
        let entries = parse_tar_verbose(output);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].size, 6);
        assert!(entries[1].is_dir);
    }

    #[test]
    fn parses_name_list() {
        let entries = parse_name_list("a.txt\nsub/\nsub/b.txt\n");
        assert_eq!(entries.len(), 3);
        assert!(entries[1].is_dir);
        assert_eq!(entries[2].components, vec!["sub", "b.txt"]);
    }

    #[test]
    fn skips_unsafe_entries() {
        let entries = parse_name_list("a.txt\n../evil\n");
        assert_eq!(entries.len(), 1);
    }
}
