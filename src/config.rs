use config::{Config, File, FileFormat};
use dirs::home_dir;

pub fn load() -> Config {
    let mut config_path = home_dir().unwrap();
    config_path.push(".config/content-7z.toml");

    let mut settings = Config::default();
    if !config_path.exists() {
        return settings
    }

    settings.merge(File::from(config_path).format(FileFormat::Toml)).unwrap();

    settings
}
