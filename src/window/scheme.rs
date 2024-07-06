use config::Config;
use std::env;
use which::which;

pub enum ColorType {
    FG,
    BG,
}

pub struct Color {
    pub r: u8,
    pub b: u8,
    pub g: u8,
    pub color_type: ColorType,
    pub repr: String,
}

pub const NOCOLOR: &[u8] = &[27, 91, 48, 109];

impl Color {
    pub fn new(r: u8, g: u8, b: u8, color_type: ColorType) -> Self {
        let mut color = Self {
            r,
            g,
            b,
            color_type,
            repr: String::new()
        };
        color.load();
        color
    }

    pub fn from(colors: Vec<u8>, color_type: ColorType) -> Self {
        let mut color = Self {
            r: if colors.len() > 0 {
                colors[0]
            } else {
                0
            },
            g: if colors.len() > 1 {
                colors[1]
            } else {
                0
            },
            b: if colors.len() > 2 {
                colors[2]
            } else {
                0
            },
            color_type,
            repr: String::new(),
        };
        color.load();
        color
    }

    pub fn load(&mut self) {
        let posfix = match self.color_type {
            ColorType::FG => "38",
            ColorType::BG => "48",
        };

        self.repr = format!("\x1b[{};2;{};{};{}m", posfix,self.r, self.g, self.b)
    }

    pub fn change(&mut self, other: Self) {
        self.r = other.r;
        self.b = other.b;
        self.g = other.g;
        self.load();
    }

    pub fn get(&self) -> String {
        self.repr.clone()
    }    
}

pub struct Scheme {
    pub background_color: Color,
    pub border_color: Color,
    pub text_color: Color,

    pub folder_bullet: String,
    pub folder_bullet_color: Color,

    pub file_bullet: String,
    pub file_bullet_color: Color,

    pub multi_choice_dialog_helper: String,

    pub editor: String,
    pub always_overwrite: bool,
}

impl Scheme {
    pub fn new() -> Self {
        Self {
            background_color: Color::new(0, 0, 0, ColorType::BG),
            border_color: Color::new(255, 255, 255, ColorType::FG),
            text_color: Color::new(200, 200, 200, ColorType::FG),

            folder_bullet: String::from("[+] "),
            folder_bullet_color: Color::new(200, 200, 200, ColorType::FG),

            file_bullet: String::from("--- "),
            file_bullet_color: Color::new(200, 200, 200, ColorType::FG),

            multi_choice_dialog_helper: String::from("\ny(es) / n(o)\n"),

            editor: String::new(),
            always_overwrite: false,
        }
    }

    pub fn from(config: Config) -> Self {
        let mut scheme = Self::new();

        if let Ok(color) = config.get::<Vec<u8>>("background-color") {
            scheme.background_color.change(Color::from(color, ColorType::BG));
        } else if let Ok(color) = config.get_string("background-color") {
            scheme.background_color.repr = format!("\x1b[{}m", color);
        }

        if let Ok(color) = config.get::<Vec<u8>>("border-color") {
            scheme.border_color.change(Color::from(color, ColorType::FG));
        } else if let Ok(color) = config.get_string("border-color") {
            scheme.border_color.repr = format!("\x1b[{}m", color);
        }

        if let Ok(color) = config.get::<Vec<u8>>("text-color") {
            scheme.text_color.change(Color::from(color, ColorType::FG));
        } else if let Ok(color) = config.get_string("text-color") {
            scheme.text_color.repr = format!("\x1b[{}m", color);
        }

        if let Ok(bullet) = config.get_string("folder-bullet") {
            scheme.folder_bullet = bullet;
        }

        if let Ok(color) = config.get::<Vec<u8>>("folder-bullet-color") {
            scheme.folder_bullet_color.change(Color::from(color, ColorType::FG));
        } else if let Ok(color) = config.get_string("folder-bullet-color") {
            scheme.folder_bullet_color.repr = format!("\x1b[{}m", color);
        }
        
        if let Ok(bullet) = config.get_string("file-bullet") {
            scheme.file_bullet = bullet;
        }

        if let Ok(color) = config.get::<Vec<u8>>("file-bullet-color") {
            scheme.file_bullet_color.change(Color::from(color, ColorType::FG));
        } else if let Ok(color) = config.get_string("file-bullet-color") {
            scheme.file_bullet_color.repr = format!("\x1b[{}m", color);
        }

        if let Ok(helper) = config.get_string("multi-choice-dialog-helper") {
            scheme.multi_choice_dialog_helper = helper;
        }

        if let Ok(editor) = config.get_string("editor") {
            scheme.editor = String::from(editor.trim());
        }

        if scheme.editor.is_empty() {
            if let Ok(editor) = env::var("EDITOR") {
                scheme.editor = editor;
            } else {
                let identify_editor = vec!["nvim", "vim", "emacs", "nano", "micro"];
                for editor in identify_editor {
                    match which(editor) {
                        Ok(path) => {
                            scheme.editor = String::from(path.to_str().unwrap());
                            break;
                        },
                        _ => {},
                    }

                }
            }
        }

        if let Ok(state) = config.get_bool("always-overwrite") {
            scheme.always_overwrite = state;
        }

        scheme
    }
}
