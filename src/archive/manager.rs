//! Reading and extracting archives through the external tools.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use crate::archive::entry::ArchiveEntry;
use crate::archive::kind::ArchiveKind;
use crate::archive::listing::{parse_name_list, parse_sevenzip_slt, parse_tar_verbose};
use crate::error::{Error, Result};
use crate::path::join_within;

/// An archive that has been opened and listed.
#[derive(Debug, Clone)]
pub struct Archive {
    pub path: PathBuf,
    pub kind: ArchiveKind,
    pub entries: Vec<ArchiveEntry>,
    /// Whether at least one entry of the archive is encrypted.
    pub encrypted: bool,
}

impl Archive {
    /// Opens an archive and lists its content.
    ///
    /// When the preferred tool is not installed, the other supported tools are
    /// tried as a fallback.
    pub fn open(path: &Path, password: Option<&str>) -> Result<Self> {
        let metadata =
            std::fs::metadata(path).map_err(|_| Error::NotFound(path.display().to_string()))?;

        if metadata.is_dir() {
            return Err(Error::Unsupported(format!(
                "{} is a directory, not an archive",
                path.display()
            )));
        }

        let kind = ArchiveKind::detect(path);
        let mut last_error: Option<Error> = None;

        for candidate in kind.fallbacks() {
            match Self::list(candidate, path, password) {
                Ok(entries) => {
                    let encrypted = entries.iter().any(|entry| entry.encrypted);
                    return Ok(Self {
                        path: path.to_path_buf(),
                        kind: candidate,
                        entries,
                        encrypted,
                    });
                }
                Err(error @ (Error::PasswordRequired | Error::WrongPassword)) => {
                    return Err(error);
                }
                Err(error) => last_error = Some(error),
            }
        }

        Err(last_error
            .unwrap_or_else(|| Error::Archive(format!("{} could not be read", path.display()))))
    }

    /// Lists the archive with a specific tool.
    fn list(kind: ArchiveKind, path: &Path, password: Option<&str>) -> Result<Vec<ArchiveEntry>> {
        match kind {
            ArchiveKind::SevenZip => {
                let output = run_tool(
                    "7z",
                    &["l", "-slt", "-ba", "--", &path.to_string_lossy()],
                    password,
                )?;
                let entries = parse_sevenzip_slt(&String::from_utf8_lossy(&output.stdout));
                if entries.is_empty() && !output.status.success() {
                    return Err(classify_failure(&output));
                }
                Ok(entries)
            }
            ArchiveKind::Tar => {
                let output = run_tool("tar", &["-tvf", &path.to_string_lossy()], password)?;
                let mut entries = parse_tar_verbose(&String::from_utf8_lossy(&output.stdout));
                if entries.is_empty() {
                    // Some tar implementations do not support the verbose
                    // listing, fall back to the plain one.
                    let output = run_tool("tar", &["-tf", &path.to_string_lossy()], password)?;
                    entries = parse_name_list(&String::from_utf8_lossy(&output.stdout));
                }
                if entries.is_empty() && !output.status.success() {
                    return Err(classify_failure(&output));
                }
                Ok(entries)
            }
            ArchiveKind::Zip => {
                let output = run_tool("unzip", &["-Z1", &path.to_string_lossy()], password)?;
                let entries = parse_name_list(&String::from_utf8_lossy(&output.stdout));
                if entries.is_empty() && !output.status.success() {
                    return Err(classify_failure(&output));
                }
                Ok(entries)
            }
        }
    }

    /// Extracts a single entry into `destination`.
    ///
    /// The returned path is the location of the extracted file.
    pub fn extract_entry(
        &self,
        entry: &ArchiveEntry,
        destination: &Path,
        password: Option<&str>,
    ) -> Result<PathBuf> {
        if self.encrypted && password.is_none() {
            return Err(Error::PasswordRequired);
        }

        let relative = PathBuf::from(entry.components.join("/"));
        let target = join_within(destination, &relative)
            .ok_or_else(|| Error::Archive(format!("unsafe path: {}", entry.full_path())))?;

        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let archive = self.path.to_string_lossy().to_string();
        let output = match self.kind {
            ArchiveKind::SevenZip => {
                let out_dir = format!("-o{}", destination.display());
                run_tool(
                    "7z",
                    &["x", "-y", &out_dir, "--", &archive, &entry.path],
                    password,
                )?
            }
            ArchiveKind::Tar => {
                let dest = destination.to_string_lossy().to_string();
                run_tool(
                    "tar",
                    &[
                        "-xf",
                        &archive,
                        "-C",
                        &dest,
                        "--no-same-owner",
                        "--no-same-permissions",
                        "--",
                        &entry.path,
                    ],
                    password,
                )?
            }
            ArchiveKind::Zip => {
                let dest = destination.to_string_lossy().to_string();
                run_tool(
                    "unzip",
                    &["-o", "-q", &archive, "-d", &dest, &entry.path],
                    password,
                )?
            }
        };

        if !output.status.success() {
            return Err(classify_failure(&output));
        }

        if target.exists() {
            return Ok(target);
        }

        // Some tools normalize the name of the extracted member, look for it
        // before giving up.
        if let Some(found) = find_by_name(destination, entry.name()) {
            return Ok(found);
        }

        Err(Error::Archive(format!(
            "{} could not be extracted",
            entry.full_path()
        )))
    }

    /// Extracts the whole archive into `destination`.
    pub fn extract_all(&self, destination: &Path, password: Option<&str>) -> Result<()> {
        if self.encrypted && password.is_none() {
            return Err(Error::PasswordRequired);
        }

        std::fs::create_dir_all(destination)?;
        let archive = self.path.to_string_lossy().to_string();

        let output = match self.kind {
            ArchiveKind::SevenZip => {
                let out_dir = format!("-o{}", destination.display());
                run_tool("7z", &["x", "-y", &out_dir, "--", &archive], password)?
            }
            ArchiveKind::Tar => {
                let dest = destination.to_string_lossy().to_string();
                run_tool(
                    "tar",
                    &[
                        "-xf",
                        &archive,
                        "-C",
                        &dest,
                        "--no-same-owner",
                        "--no-same-permissions",
                    ],
                    password,
                )?
            }
            ArchiveKind::Zip => {
                let dest = destination.to_string_lossy().to_string();
                run_tool("unzip", &["-o", "-q", &archive, "-d", &dest], password)?
            }
        };

        if !output.status.success() {
            return Err(classify_failure(&output));
        }

        Ok(())
    }
}

/// Runs an external tool, optionally feeding a password through its standard
/// input.
///
/// The password is never passed as an argument, so it does not show up in the
/// process list.
fn run_tool(program: &str, args: &[&str], password: Option<&str>) -> Result<Output> {
    let mut command = Command::new(program);
    command
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command.spawn().map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            Error::ToolMissing(program.to_string())
        } else {
            Error::Io(error)
        }
    })?;

    if let Some(password) = password {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = writeln!(stdin, "{password}");
            let _ = stdin.flush();
        }
    }
    // Closing stdin makes the tools stop waiting for a password.
    drop(child.stdin.take());

    child.wait_with_output().map_err(Error::Io)
}

/// Turns the output of a failed command into a meaningful error.
fn classify_failure(output: &Output) -> Error {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}\n{stderr}");
    let lower = combined.to_lowercase();

    if lower.contains("wrong password") || lower.contains("incorrect password") {
        return Error::WrongPassword;
    }
    if lower.contains("enter password") || lower.contains("password is incorrect") {
        return Error::PasswordRequired;
    }

    let message = stderr.trim().to_string();
    let message = if message.is_empty() {
        stdout.trim().to_string()
    } else {
        message
    };

    if message.is_empty() {
        Error::Archive(format!(
            "the command exited with status {}",
            status_code(output)
        ))
    } else {
        Error::Archive(message)
    }
}

/// Returns the exit code of a finished command.
fn status_code(output: &Output) -> i32 {
    output.status.code().unwrap_or(-1)
}

/// Looks for a file called `name` inside `root`.
fn find_by_name(root: &Path, name: &str) -> Option<PathBuf> {
    let entries = std::fs::read_dir(root).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_by_name(&path, name) {
                return Some(found);
            }
        } else if path.file_name().map(|value| value == name).unwrap_or(false) {
            return Some(path);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_tool_is_reported() {
        let error = run_tool("this-tool-does-not-exist", &["--help"], None).unwrap_err();
        assert!(matches!(error, Error::ToolMissing(_)));
    }

    #[test]
    fn password_is_not_part_of_the_arguments() {
        // The tool is called with the password on stdin, never as an argument.
        let output = run_tool("cat", &[], Some("secret")).unwrap();
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "secret");
    }

    #[test]
    fn a_missing_archive_is_reported() {
        let error = Archive::open(Path::new("/does/not/exist.7z"), None).unwrap_err();
        assert!(matches!(error, Error::NotFound(_)));
    }

    #[test]
    fn a_directory_is_not_an_archive() {
        let error = Archive::open(Path::new("/tmp"), None).unwrap_err();
        assert!(matches!(error, Error::Unsupported(_)));
    }
}
