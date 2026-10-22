//! Command line interface.

use std::path::PathBuf;

/// Environment variable that can hold the archive password.
pub const PASSWORD_ENV: &str = "CONTENT_7Z_PASSWORD";

/// The name of the program.
pub const PROGRAM: &str = "content-7z";

/// The version of the program.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// What the program has to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Browse the archive in the interactive interface.
    Browse(Options),
    /// Print the content of the archive and exit.
    List(Options),
    /// Extract the whole archive and exit.
    Extract {
        options: Options,
        destination: PathBuf,
    },
    /// Print the help message.
    Help,
    /// Print the version.
    Version,
}

/// The options shared by every command.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Options {
    /// Path of the archive.
    pub archive: PathBuf,
    /// Password of the archive, when it is known.
    pub password: Option<String>,
}

/// Parses the arguments of the program.
pub fn parse<S: AsRef<str>>(args: &[S]) -> Result<Command, String> {
    let mut args = args.iter().map(|value| value.as_ref().to_string());

    let program = args.next().unwrap_or_else(|| PROGRAM.to_string());

    let mut archive: Option<String> = None;
    let mut password: Option<String> = None;
    let mut list = false;
    let mut extract: Option<PathBuf> = None;
    let mut help = false;
    let mut version = false;

    while let Some(argument) = args.next() {
        match argument.as_str() {
            "-h" | "--help" => help = true,
            "-V" | "--version" => version = true,
            "-l" | "--list" => list = true,
            "-p" | "--password" => {
                password = Some(
                    args.next()
                        .ok_or_else(|| format!("{program}: --password needs a value"))?,
                );
            }
            "-x" | "--extract" => {
                let value = args
                    .next()
                    .ok_or_else(|| format!("{program}: --extract needs a directory"))?;
                extract = Some(PathBuf::from(value));
            }
            other if other.starts_with("--password=") => {
                password = Some(other["--password=".len()..].to_string());
            }
            other if other.starts_with("--extract=") => {
                extract = Some(PathBuf::from(&other["--extract=".len()..]));
            }
            // Kept for compatibility with the previous versions, which took
            // the password as `-p<password>`.
            other if other.starts_with("-p") && other.len() > 2 => {
                password = Some(other[2..].to_string());
            }
            other if other.starts_with('-') && other.len() > 1 => {
                return Err(format!("{program}: unknown option '{other}'"));
            }
            other => {
                if archive.is_some() {
                    return Err(format!("{program}: unexpected argument '{other}'"));
                }
                archive = Some(other.to_string());
            }
        }
    }

    if help {
        return Ok(Command::Help);
    }
    if version {
        return Ok(Command::Version);
    }

    let Some(archive) = archive else {
        return Err(format!(
            "{program}: missing archive\nTry '{program} --help' for more information."
        ));
    };

    if password.is_none() {
        password = std::env::var(PASSWORD_ENV)
            .ok()
            .filter(|value| !value.is_empty());
    }

    let options = Options {
        archive: PathBuf::from(archive),
        password,
    };

    Ok(match (list, extract) {
        (true, _) => Command::List(options),
        (false, Some(destination)) => Command::Extract {
            options,
            destination,
        },
        (false, None) => Command::Browse(options),
    })
}

/// The help message of the program.
pub fn help() -> String {
    format!(
        "\
{PROGRAM} {VERSION}
Browse the content of 7z, zip and tar archives from the terminal.

USAGE:
    {PROGRAM} [OPTIONS] <ARCHIVE>

OPTIONS:
    -p, --password <PASSWORD>  Password of an encrypted archive.
                               It can also be given with the {PASSWORD_ENV}
                               environment variable.
    -l, --list                 Print the content of the archive and exit.
    -x, --extract <DIRECTORY>  Extract the whole archive and exit.
    -h, --help                 Print this help.
    -V, --version              Print the version.

KEYS (interactive mode):
    up/down      move the selection
    enter/right  open the selected folder
    left/-/back  go back to the parent folder
    o            open the selected file with the editor
    x            extract the whole archive
    /            filter the entries by name
    r            reload the archive
    ?            show the help
    q            quit
"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_simple_invocation() {
        let command = parse(&["content-7z", "archive.7z"]).unwrap();
        assert_eq!(
            command,
            Command::Browse(Options {
                archive: PathBuf::from("archive.7z"),
                password: None,
            })
        );
    }

    #[test]
    fn parses_the_password_option() {
        let command = parse(&["content-7z", "archive.7z", "--password", "secret"]).unwrap();
        match command {
            Command::Browse(options) => assert_eq!(options.password.as_deref(), Some("secret")),
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parses_the_attached_password_option() {
        let command = parse(&["content-7z", "archive.7z", "-psecret"]).unwrap();
        match command {
            Command::Browse(options) => assert_eq!(options.password.as_deref(), Some("secret")),
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parses_the_list_option() {
        assert!(matches!(
            parse(&["content-7z", "archive.7z", "--list"]).unwrap(),
            Command::List(_)
        ));
    }

    #[test]
    fn parses_the_extract_option() {
        assert!(matches!(
            parse(&["content-7z", "archive.7z", "--extract", "out"]).unwrap(),
            Command::Extract { .. }
        ));
    }

    #[test]
    fn rejects_unknown_options() {
        assert!(parse(&["content-7z", "--nope"]).is_err());
    }

    #[test]
    fn rejects_a_missing_archive() {
        assert!(parse(&["content-7z"]).is_err());
    }

    #[test]
    fn rejects_extra_arguments() {
        assert!(parse(&["content-7z", "a.7z", "b.7z"]).is_err());
    }
}
