//! Theme (colors and bullets) used by the terminal interface.

use ratatui::style::{Color, Modifier, Style};

use crate::config::{ColorValue, FileConfig};

/// Colors and decorations used to draw the interface.
#[derive(Debug, Clone)]
pub struct Theme {
    pub background: Color,
    pub text: Color,
    pub border: Color,
    pub accent: Color,
    pub error: Color,
    pub warning: Color,
    pub success: Color,
    pub dim: Color,

    pub folder_bullet: String,
    pub folder_bullet_color: Color,
    pub file_bullet: String,
    pub file_bullet_color: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: Color::Reset,
            text: Color::Rgb(200, 200, 200),
            border: Color::Rgb(255, 255, 255),
            accent: Color::Rgb(97, 175, 239),
            error: Color::Rgb(255, 85, 85),
            warning: Color::Rgb(229, 192, 123),
            success: Color::Rgb(152, 195, 121),
            dim: Color::Rgb(128, 128, 128),

            folder_bullet: String::from("[+] "),
            folder_bullet_color: Color::Rgb(200, 200, 200),
            file_bullet: String::from("--- "),
            file_bullet_color: Color::Rgb(200, 200, 200),
        }
    }
}

impl Theme {
    /// Builds the theme from the parsed configuration file.
    pub fn from_file(file: &FileConfig) -> Self {
        let mut theme = Theme::default();

        if let Some(value) = &file.background_color {
            if let Some(color) = background_of(value) {
                theme.background = color;
            }
        }

        if let Some(value) = &file.text_color {
            if let Some(color) = foreground_of(value) {
                theme.text = color;
            }
        }

        if let Some(value) = &file.border_color {
            if let Some(color) = foreground_of(value) {
                theme.border = color;
            }
        }

        if let Some(bullet) = &file.folder_bullet {
            theme.folder_bullet = bullet.clone();
        }
        if let Some(value) = &file.folder_bullet_color {
            if let Some(color) = foreground_of(value) {
                theme.folder_bullet_color = color;
            }
        }

        if let Some(bullet) = &file.file_bullet {
            theme.file_bullet = bullet.clone();
        }
        if let Some(value) = &file.file_bullet_color {
            if let Some(color) = foreground_of(value) {
                theme.file_bullet_color = color;
            }
        }

        theme
    }

    /// Style used for the regular content of the interface.
    pub fn base(&self) -> Style {
        Style::default().fg(self.text).bg(self.background)
    }

    /// Style used for the borders of the panels.
    pub fn border(&self) -> Style {
        Style::default().fg(self.border).bg(self.background)
    }

    /// Style used for the selected row.
    pub fn selection(&self) -> Style {
        Style::default()
            .fg(Color::Black)
            .bg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    /// Style used for dimmed elements such as hints and sizes.
    pub fn muted(&self) -> Style {
        Style::default().fg(self.dim).bg(self.background)
    }
}

/// Extracts the foreground color of a configured color.
fn foreground_of(value: &ColorValue) -> Option<Color> {
    match value {
        ColorValue::Rgb(components) if components.len() >= 3 => {
            Some(Color::Rgb(components[0], components[1], components[2]))
        }
        ColorValue::Rgb(_) => None,
        ColorValue::Ansi(spec) => parse_ansi(spec).fg,
    }
}

/// Extracts the background color of a configured color.
fn background_of(value: &ColorValue) -> Option<Color> {
    match value {
        ColorValue::Rgb(components) if components.len() >= 3 => {
            Some(Color::Rgb(components[0], components[1], components[2]))
        }
        ColorValue::Rgb(_) => None,
        ColorValue::Ansi(spec) => parse_ansi(spec).bg,
    }
}

/// Parses the body of an ANSI escape sequence into a [`Style`].
///
/// Supported: the usual attributes (`1` bold, `2` dim, `3` italic,
/// `4` underline, `7` reverse, ...), the standard/bright foreground and
/// background colors (`30`..`37`, `40`..`47`, `90`..`97`) and the extended
/// forms `38;5;n`, `48;5;n`, `38;2;r;g;b` and `48;2;r;g;b`.
pub fn parse_ansi(spec: &str) -> Style {
    let mut style = Style::default();
    let tokens: Vec<u8> = spec
        .split(';')
        .filter_map(|token| token.trim().parse::<u8>().ok())
        .collect();

    let mut index = 0usize;
    while index < tokens.len() {
        let token = tokens[index];
        match token {
            0 => style = Style::default(),
            1 => style = style.add_modifier(Modifier::BOLD),
            2 => style = style.add_modifier(Modifier::DIM),
            3 => style = style.add_modifier(Modifier::ITALIC),
            4 => style = style.add_modifier(Modifier::UNDERLINED),
            5 => style = style.add_modifier(Modifier::SLOW_BLINK),
            7 => style = style.add_modifier(Modifier::REVERSED),
            8 => style = style.add_modifier(Modifier::HIDDEN),
            9 => style = style.add_modifier(Modifier::CROSSED_OUT),
            22 => style = style.remove_modifier(Modifier::BOLD | Modifier::DIM),
            23 => style = style.remove_modifier(Modifier::ITALIC),
            24 => style = style.remove_modifier(Modifier::UNDERLINED),
            25 => style = style.remove_modifier(Modifier::SLOW_BLINK),
            27 => style = style.remove_modifier(Modifier::REVERSED),
            28 => style = style.remove_modifier(Modifier::HIDDEN),
            29 => style = style.remove_modifier(Modifier::CROSSED_OUT),
            30..=37 => style = style.fg(standard_color(token - 30)),
            39 => style = style.fg(Color::Reset),
            40..=47 => style = style.bg(standard_color(token - 40)),
            49 => style = style.bg(Color::Reset),
            90..=97 => style = style.fg(bright_color(token - 90)),
            100..=107 => style = style.bg(bright_color(token - 100)),
            38 | 48 => {
                let (color, consumed) = parse_extended_color(&tokens[index + 1..]);
                match color {
                    Some(color) => {
                        if token == 38 {
                            style = style.fg(color);
                        } else {
                            style = style.bg(color);
                        }
                    }
                    None => break,
                }
                index += consumed;
            }
            _ => {}
        }
        index += 1;
    }

    style
}

/// Parses `5;n` or `2;r;g;b` sequences, returning the color and how many
/// tokens were consumed.
fn parse_extended_color(tokens: &[u8]) -> (Option<Color>, usize) {
    match tokens.first() {
        Some(5) => match tokens.get(1) {
            Some(index) => (Some(Color::Indexed(*index)), 2),
            None => (None, 0),
        },
        Some(2) => {
            if tokens.len() >= 4 {
                (Some(Color::Rgb(tokens[1], tokens[2], tokens[3])), 4)
            } else {
                (None, 0)
            }
        }
        _ => (None, 0),
    }
}

/// Maps an index of the standard palette to a color.
fn standard_color(index: u8) -> Color {
    match index {
        0 => Color::Black,
        1 => Color::Red,
        2 => Color::Green,
        3 => Color::Yellow,
        4 => Color::Blue,
        5 => Color::Magenta,
        6 => Color::Cyan,
        _ => Color::Gray,
    }
}

/// Maps an index of the bright palette to a color.
fn bright_color(index: u8) -> Color {
    match index {
        0 => Color::DarkGray,
        1 => Color::LightRed,
        2 => Color::LightGreen,
        3 => Color::LightYellow,
        4 => Color::LightBlue,
        5 => Color::LightMagenta,
        6 => Color::LightCyan,
        _ => Color::White,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_attributes() {
        let style = parse_ansi("1");
        assert!(style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn parses_standard_colors() {
        assert_eq!(parse_ansi("34").fg, Some(Color::Blue));
        assert_eq!(parse_ansi("44").bg, Some(Color::Blue));
        assert_eq!(parse_ansi("91").fg, Some(Color::LightRed));
    }

    #[test]
    fn parses_extended_colors() {
        assert_eq!(parse_ansi("38;2;10;20;30").fg, Some(Color::Rgb(10, 20, 30)));
        assert_eq!(parse_ansi("48;2;10;20;30").bg, Some(Color::Rgb(10, 20, 30)));
        assert_eq!(parse_ansi("38;5;120").fg, Some(Color::Indexed(120)));
    }

    #[test]
    fn combines_attributes_and_colors() {
        let style = parse_ansi("1;38;2;1;2;3");
        assert!(style.add_modifier.contains(Modifier::BOLD));
        assert_eq!(style.fg, Some(Color::Rgb(1, 2, 3)));
    }

    #[test]
    fn theme_uses_configured_colors() {
        let file: FileConfig = toml::from_str("text-color = [1, 2, 3]\n").unwrap();
        let theme = Theme::from_file(&file);
        assert_eq!(theme.text, Color::Rgb(1, 2, 3));
    }

    #[test]
    fn theme_uses_the_background_color() {
        let file: FileConfig = toml::from_str("background-color = [4, 5, 6]\n").unwrap();
        let theme = Theme::from_file(&file);
        assert_eq!(theme.background, Color::Rgb(4, 5, 6));
    }

    #[test]
    fn an_empty_color_keeps_the_default() {
        let file: FileConfig = toml::from_str("background-color = []\n").unwrap();
        let theme = Theme::from_file(&file);
        assert_eq!(theme.background, Theme::default().background);
    }
}
