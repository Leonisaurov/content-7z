use config::Config;

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

        self.repr = format!("\x1b[{};2;{};{};{}m", posfix,self.r, self.b, self.g)
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
    pub background: Color,
    pub borders: Color,
    pub text: Color,
    pub type_flag: Color,
}

impl Scheme {
    pub fn new() -> Self {
        Self {
            background: Color::new(0, 0, 0, ColorType::BG),
            borders: Color::new(255, 255, 255, ColorType::FG),
            text: Color::new(200, 200, 200, ColorType::FG),
            type_flag: Color::new(200, 200, 200, ColorType::FG),
        }
    }

    pub fn from(config: Config) -> Self {
        let mut scheme = Self::new();

        if let Ok(color) = config.get::<Vec<u8>>("background-color") {
            scheme.background.change(Color::from(color, ColorType::BG));
        }

        if let Ok(color) = config.get::<Vec<u8>>("border-color") {
            scheme.borders.change(Color::from(color, ColorType::FG));
        }

        if let Ok(color) = config.get::<Vec<u8>>("text-color") {
            scheme.text.change(Color::from(color, ColorType::FG));
        }

        scheme
    }
}
