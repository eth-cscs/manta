use config::{Config, File, FileFormat};

/// Reads configuration file with manta parameters
pub fn get_configuration(config_path: &str) -> Config {
    let config = Config::builder()
        .add_source(File::new(config_path, FileFormat::Toml))
        .build();

    match config {
        Err(_) => {
            eprintln!("Configuration missing or wrong format!. Exit");
            std::process::exit(1);
        }
        Ok(config_content) => config_content,
    }
}
