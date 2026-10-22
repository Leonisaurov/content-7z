//! Detection and launching of the external editor.

use std::path::Path;
use std::process::{Command, ExitStatus};

use crate::error::{Error, Result};

/// Editors that are looked up in the `PATH` when nothing is configured.
const KNOWN_EDITORS: [&str; 10] = [
    "nvim", "vim", "vi", "emacs", "nano", "micro", "kak", "helix", "code", "gedit",
];

/// An editor command, ready to be executed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Editor {
    pub program: String,
    pub args: Vec<String>,
}

impl Editor {
    /// Builds the command that opens `path` with the editor.
    pub fn command(&self, path: &Path) -> Command {
        let mut command = Command::new(&self.program);
        command.args(&self.args);
        command.arg(path);
        command
    }

    /// Opens `path` with the editor and waits for it to finish.
    pub fn open(&self, path: &Path) -> Result<ExitStatus> {
        self.command(path).status().map_err(Error::Io)
    }
}

/// Finds the editor to use.
///
/// The configured editor wins, then the `EDITOR` environment variable and
/// finally the first known editor available in the `PATH`.
pub fn resolve(configured: Option<&str>) -> Option<Editor> {
    if let Some(configured) = configured {
        let parts = split_command(configured);
        if let Some((program, args)) = parts.split_first() {
            return Some(Editor {
                program: program.clone(),
                args: args.to_vec(),
            });
        }
    }

    if let Ok(editor) = std::env::var("EDITOR") {
        let parts = split_command(&editor);
        if let Some((program, args)) = parts.split_first() {
            return Some(Editor {
                program: program.clone(),
                args: args.to_vec(),
            });
        }
    }

    for candidate in KNOWN_EDITORS {
        if let Ok(path) = which::which(candidate) {
            return Some(Editor {
                program: path.to_string_lossy().to_string(),
                args: Vec::new(),
            });
        }
    }

    None
}

/// Splits a command line into its arguments.
///
/// Supports single quotes, double quotes and backslash escapes, which is
/// enough for the `editor = "code -w"` style of configuration.
pub fn split_command(input: &str) -> Vec<String> {
    let mut parts: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut has_part = false;
    let mut chars = input.chars().peekable();

    while let Some(character) = chars.next() {
        match character {
            ' ' | '\t' | '\n' | '\r' => {
                if has_part {
                    parts.push(std::mem::take(&mut current));
                    has_part = false;
                }
            }
            '\'' => {
                has_part = true;
                for character in chars.by_ref() {
                    if character == '\'' {
                        break;
                    }
                    current.push(character);
                }
            }
            '"' => {
                has_part = true;
                while let Some(character) = chars.next() {
                    if character == '"' {
                        break;
                    }
                    if character == '\\' {
                        if let Some(escaped) = chars.next() {
                            current.push(escaped);
                            continue;
                        }
                    }
                    current.push(character);
                }
            }
            '\\' => {
                has_part = true;
                if let Some(escaped) = chars.next() {
                    current.push(escaped);
                }
            }
            _ => {
                has_part = true;
                current.push(character);
            }
        }
    }

    if has_part {
        parts.push(current);
    }

    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_plain_commands() {
        assert_eq!(split_command("nvim"), vec!["nvim"]);
        assert_eq!(split_command("code -w"), vec!["code", "-w"]);
        assert_eq!(split_command("  code   -w  "), vec!["code", "-w"]);
    }

    #[test]
    fn honours_quotes() {
        assert_eq!(
            split_command("\"/usr/bin/my editor\" -f"),
            vec!["/usr/bin/my editor", "-f"]
        );
        assert_eq!(split_command("'my editor'"), vec!["my editor"]);
    }

    #[test]
    fn honours_escapes() {
        assert_eq!(split_command("my\\ editor"), vec!["my editor"]);
    }

    #[test]
    fn empty_input_yields_no_parts() {
        assert!(split_command("").is_empty());
        assert!(split_command("   ").is_empty());
    }

    #[test]
    fn builds_the_command() {
        let editor = Editor {
            program: "nvim".to_string(),
            args: vec!["-w".to_string()],
        };
        let command = editor.command(Path::new("/tmp/file.txt"));
        let args: Vec<_> = command.get_args().collect();
        assert_eq!(args.len(), 2);
        assert_eq!(args[1], std::ffi::OsStr::new("/tmp/file.txt"));
    }
}
