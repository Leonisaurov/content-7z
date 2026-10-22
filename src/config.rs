//! Configuration file handling.
//!
//! The configuration lives in `$XDG_CONFIG_HOME/content-7z.toml` (falling back
//! to `~/.config/content-7z.toml`). Every property is optional, a missing or
//! partially filled file simply falls back to the defaults.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::{Error, Result};
use crate::theme::Theme;

/// Environment variable that overrides the configuration file location.
pub const CONFIG_PATH_ENV: &str = "CONTENT_7Z_CONFIG";

/// Raw contents of the configuration file.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct FileConfig {
    #[serde(rename = "background-color")]
    pub background_color: Option<ColorValue>,
    #[serde(rename = "text-color")]
    pub text_color: Option<ColorValue>,
    #[serde(rename = "border-color")]
    pub border_color: Option<ColorValue>,
    #[serde(rename = "folder-bullet")]
    pub folder_bullet: Option<String>,
    #[serde(rename = "folder-bullet-color")]
    pub folder_bullet_color: Option<ColorValue>,
    #[serde(rename = "file-bullet")]
    pub file_bullet: Option<String>,
    #[serde(rename = "file-bullet-color")]
    pub file_bullet_color: Option<ColorValue>,
    #[serde(rename = "multi-choice-dialog-helper", alias = "multi-choice-helper")]
    pub multi_choice_dialog_helper: Option<String>,
    pub editor: Option<String>,
    #[serde(rename = "always-overwrite")]
    pub always_overwrite: Option<bool>,
    #[serde(rename = "show-size")]
    pub show_size: Option<bool>,
    #[serde(rename = "show-hidden")]
    pub show_hidden: Option<bool>,
}

/// A color as written in the configuration file.
///
/// Either an RGB triplet (`[30, 30, 46]`) or an ANSI escape sequence body
/// (`"1;34"`).
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ColorValue {
    Rgb(Vec<u8>),
    Ansi(String),
}

/// Effective configuration of the application.
#[derive(Debug, Clone)]
pub struct Config {
    pub theme: Theme,
    pub editor: Option<String>,
    pub always_overwrite: bool,
    pub show_size: bool,
    pub show_hidden: bool,
    pub multi_choice_dialog_helper: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            editor: None,
            always_overwrite: false,
            show_size: true,
            show_hidden: true,
            multi_choice_dialog_helper: None,
        }
    }
}

impl Config {
    /// Builds the effective configuration from the parsed file contents.
    pub fn from_file(file: &FileConfig) -> Self {
        Self {
            theme: Theme::from_file(file),
            editor: file
                .editor
                .as_ref()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            always_overwrite: file.always_overwrite.unwrap_or(false),
            show_size: file.show_size.unwrap_or(true),
            show_hidden: file.show_hidden.unwrap_or(true),
            multi_choice_dialog_helper: file.multi_choice_dialog_helper.clone(),
        }
    }

    /// Loads the configuration from disk.
    ///
    /// A missing file is not an error: the defaults are used instead.
    pub fn load() -> Result<Self> {
        let path = config_path();
        let Some(path) = path else {
            return Ok(Config::default());
        };

        if !path.exists() {
            return Ok(Config::default());
        }

        let contents = std::fs::read_to_string(&path)
            .map_err(|e| Error::Config(format!("{}: {e}", path.display())))?;

        let file: FileConfig = toml::from_str(&contents)
            .map_err(|e| Error::Config(format!("{}: {e}", path.display())))?;

        Ok(Config::from_file(&file))
    }
}

/// Returns the path of the configuration file, if it can be determined.
pub fn config_path() -> Option<PathBuf> {
    if let Some(custom) = std::env::var_os(CONFIG_PATH_ENV) {
        if !custom.is_empty() {
            return Some(PathBuf::from(custom));
        }
    }

    let base = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| home_dir().map(|home| home.join(".config")))?;

    Some(base.join("content-7z.toml"))
}

/// Returns the home directory of the current user.
pub fn home_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("USERPROFILE")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
    }

    #[cfg(not(windows))]
    {
        std::env::var_os("HOME")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
    }
}

/// Writes a commented default configuration file to `path`.
pub fn write_default(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| Error::Config(format!("{}: {e}", parent.display())))?;
    }
    std::fs::write(path, DEFAULT_CONFIG)
        .map_err(|e| Error::Config(format!("{}: {e}", path.display())))?;
    Ok(())
}

/// The template used by `write_default` and shipped with the installer.
pub const DEFAULT_CONFIG: &str = r#"# content-7z configuration file.
#
# Colors accept either an RGB triplet, e.g.
#     text-color = [205, 214, 244]
# or the body of an ANSI escape sequence, e.g.
#     text-color = "1;34"

# Catppuccin
# background-color = [30, 30, 46]
# text-color = [205, 214, 244]
# border-color = [69, 71, 90]

# Tokio Dark
# background-color = [17, 18, 29]
# text-color = [160, 168, 205]
# border-color = [74, 80, 87]

# Rose pine
# text-color = [224, 222, 244]
# background-color = [25, 23, 36]
# border-color = [110, 106, 134]

# Github Default
# text-color = [230, 237, 243]
# background-color = [13, 17, 23]
# border-color = [110, 118, 129]

# Lunaperche
# text-color = [198, 198, 198]
# background-color = [0, 0, 0]
# border-color = [88, 88, 88]

# Fancy colored icons
# folder-bullet = " \ue5fe "
# folder-bullet-color = "1;34"

# file-bullet = " \uea7b "
# file-bullet-color = "38;2;88;88;88;1"

# Editor used to open the files, it may contain arguments.
# editor = "nvim"

# Always re-extract files, even when they are already cached.
# always-overwrite = false

# Show the size of the entries in the list.
# show-size = true

# Show hidden entries (those starting with a dot).
# show-hidden = false
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rgb_and_ansi_colors() {
        let file: FileConfig = toml::from_str(
            r#"
            background-color = [30, 30, 46]
            text-color = "1;34"
            "#,
        )
        .unwrap();

        assert!(matches!(file.background_color, Some(ColorValue::Rgb(_))));
        assert!(matches!(file.text_color, Some(ColorValue::Ansi(_))));
    }

    #[test]
    fn unknown_keys_do_not_break_the_file() {
        let file: FileConfig = toml::from_str("unknown-key = 42\n").unwrap();
        assert!(file.editor.is_none());
    }

    #[test]
    fn defaults_are_applied() {
        let config = Config::from_file(&FileConfig::default());
        assert!(config.show_size);
        assert!(!config.always_overwrite);
        assert!(config.editor.is_none());
    }
}
