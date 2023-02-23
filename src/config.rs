use config::{Config, File, FileFormat};

/// Reads configuration file with manta parameters
pub fn get(config_path: &str) -> Config {
    Config::builder()
        .add_source(File::new(config_path, FileFormat::Toml))
        .build()
        .unwrap()
}
