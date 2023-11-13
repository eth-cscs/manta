use std::{fs::File, io::Read, path::PathBuf};

use config::Config;
use directories::ProjectDirs;

pub fn get_configuration_file_path() -> PathBuf {
    // XDG Base Directory Specification
    let project_dirs = ProjectDirs::from(
        "local", /*qualifier*/
        "cscs",  /*organization*/
        "manta", /*application*/
    );

    PathBuf::from(project_dirs.unwrap().config_dir())
}

/// Reads configuration parameters related to manta from environment variables or file. If both
/// defiend, then environment variables takes preference
pub fn get_configuration() -> Config {
    let mut config_path = get_configuration_file_path();
    config_path.push("config.toml"); // ~/.config/manta/config is the file

    ::config::Config::builder()
        .add_source(::config::File::from(config_path))
        .add_source(
            ::config::Environment::with_prefix("MANTA")
                .try_parsing(true)
                .prefix_separator("_"),
        )
        .build()
        .unwrap()
}

pub fn get_csm_root_cert_content(site: &str) -> Vec<u8> {
    let mut config_path = get_configuration_file_path();
    config_path.push(site.to_string() + "_root_cert.pem");

    let mut buf = Vec::new();
    let root_cert_file_rslt = File::open(config_path);

    let _ = match root_cert_file_rslt {
        Ok(mut file) => file.read_to_end(&mut buf),
        Err(_) => {
            eprintln!("Root cert file for CSM not found. Exit");
            std::process::exit(1);
        }
    };

    buf
}
