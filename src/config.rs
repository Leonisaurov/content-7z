use config::{Config, File, FileFormat};
use dirs::home_dir;

pub fn load() -> Config {
    let mut settings_builder = Config::builder();
    if let Some(home) = home_dir() {
        let mut config_path = home;
        config_path.push(".config/content-7z.toml");
        if config_path.exists() {
            settings_builder = settings_builder.add_source(File::from(config_path).format(FileFormat::Toml));
        }
    }

    settings_builder.build().unwrap_or_default()
}
