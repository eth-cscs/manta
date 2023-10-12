use std::path::PathBuf;

use config::Config;
use directories::ProjectDirs;

pub fn get_configuration_file_path() -> PathBuf {
    // XDG Base Directory Specification
    let project_dirs = ProjectDirs::from(
        "local", /*qualifier*/
        "cscs",  /*organization*/
        "manta", /*application*/
    );

    let mut path_to_manta_configuration_file = PathBuf::from(project_dirs.unwrap().config_dir());

    path_to_manta_configuration_file.push("config.toml"); // ~/.config/manta/config is the file

    path_to_manta_configuration_file
}

/// Reads configuration file with manta parameters
pub fn get_configuration() -> Config {
    let path_to_manta_configuration_file = get_configuration_file_path();

    // let settings = config::get_configuration(&path_to_manta_configuration_file.to_string_lossy());
    ::config::Config::builder()
        .add_source(::config::File::from(path_to_manta_configuration_file))
        .add_source(
            ::config::Environment::with_prefix("MANTA")
                .try_parsing(true)
                .prefix_separator("_"),
        )
        .build()
        .unwrap()
}
