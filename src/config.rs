use config::{Config, File, FileFormat};
use dirs::home_dir;

pub fn load() -> Config {
    let mut config_path = home_dir().unwrap();
    config_path.push(".config/content-7z.toml");

    let mut settings_builder = Config::builder();
    if config_path.exists() {
        settings_builder = settings_builder.add_source(File::from(config_path).format(FileFormat::Toml));
    }

    settings_builder.build().unwrap()
}
