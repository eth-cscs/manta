use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use config::{Config, FileFormat, FileSourceFile};
use dialoguer::{Input, Select};
use directories::ProjectDirs;
use serde::Serialize;

#[derive(Serialize)]
pub struct Site {
    socks5_proxy: Option<String>,
    shasta_base_url: String,
    k8s_api_url: String,
    vault_base_url: String,
    vault_secret_path: String,
    vault_role_id: String,
    root_ca_cert_file: String,
}

#[derive(Serialize)]
pub struct MantaConfiguration {
    log: String,
    site: String,
    parent_hsm_group: String,
    audit_file: String,
    sites: HashMap<String, Site>,
}

pub fn get_default_config_path() -> PathBuf {
    // XDG Base Directory Specification
    let project_dirs = ProjectDirs::from(
        "local", /*qualifier*/
        "cscs",  /*organization*/
        "manta", /*application*/
    );

    PathBuf::from(project_dirs.unwrap().config_dir())
}

pub fn get_default_manta_config_file_path() -> PathBuf {
    // XDG Base Directory Specification
    let project_dirs = ProjectDirs::from(
        "local", /*qualifier*/
        "cscs",  /*organization*/
        "manta", /*application*/
    );

    let mut config_file_path = PathBuf::from(project_dirs.unwrap().config_dir());
    config_file_path.push("config.toml");
    config_file_path
}

pub fn get_default_manta_audit_file_path() -> PathBuf {
    // XDG Base Directory Specification
    let project_dirs = ProjectDirs::from(
        "local", /*qualifier*/
        "cscs",  /*organization*/
        "manta", /*application*/
    );

    let mut log_file_path = PathBuf::from(project_dirs.unwrap().data_dir());
    log_file_path.push("manta.log");

    log_file_path
}

pub fn get_default_mgmt_plane_ca_cert_file_path() -> PathBuf {
    // XDG Base Directory Specification
    let project_dirs = ProjectDirs::from(
        "local", /*qualifier*/
        "cscs",  /*organization*/
        "manta", /*application*/
    );

    let mut ca_cert_file_path = PathBuf::from(project_dirs.unwrap().config_dir());
    ca_cert_file_path.push("alps_root_cert.pem");

    ca_cert_file_path
}

/// Get Manta configuration full path. Configuration may be the default one or specified by user.
/// This function also validates if the config file is TOML format
pub async fn get_config_file_path() -> config::File<FileSourceFile, FileFormat> {
    // Get config file path from ENV var
    let config_file_path = if let Ok(env_config_file_name) = std::env::var("MANTA_CONFIG") {
        let mut env_config_file = std::path::PathBuf::new();
        env_config_file.push(env_config_file_name);
        env_config_file
    } else {
        // Get default config file path ($XDG_CONFIG/manta/config.toml
        get_default_manta_config_file_path()
    };

    // Validate if config file exists
    if !config_file_path.exists() {
        // Configuration file does not exists --> create a new configuration file
        create_new_config_file(Some(&config_file_path)).await;
    }

    // Validate config file and check format (toml) is correct
    let config_file = config::File::new(
        config_file_path
            .to_str()
            .expect("Configuration file name not defined"),
        config::FileFormat::Toml,
    );

    config_file
}

/// Reads configuration parameters related to manta from environment variables or file. If both
/// defiend, then environment variables takes preference
pub async fn get_configuration() -> Config {
    // Get config file path

    let config_file = get_config_file_path().await;

    // Process config file
    let config_rslt = ::config::Config::builder()
        .add_source(config_file)
        .add_source(
            ::config::Environment::with_prefix("MANTA")
                .try_parsing(true)
                .prefix_separator("_"),
        )
        .build();

    match config_rslt {
        Ok(config) => config,
        Err(error) => {
            eprintln!(
                "Error processing manta configuration file. Reason:\n{}",
                error
            );

            std::process::exit(1);
        }
    }
}

pub async fn create_new_config_file(config_file_path_opt: Option<&PathBuf>) {
    eprintln!("Confguration file not found. Please introduce values below:");
    /* let log: String = Input::new()
    .with_prompt("Please enter a value for param 'log'")
    .with_initial_text("error")
    .show_default(true)
    .interact_text()
    .unwrap(); */

    let log_level_values = vec!["error", "info", "warning", "debug", "trace"];

    let log_selection = Select::new()
        .with_prompt("Please select 'log verbosity' level from the list below")
        .items(&log_level_values)
        .default(0)
        .interact()
        .unwrap();

    let log = log_level_values[log_selection].to_string();

    let parent_hsm_group = "".to_string();

    /* let mut parent_hsm_group = String::new();
    print!("Please enter a value for param 'parent_hsm_group': ");
    let _ = stdout().flush();
    stdin().read_line(&mut parent_hsm_group).unwrap(); */

    let audit_file: String = Input::new()
        .with_prompt("Please type full path for the audit file")
        .default(
            get_default_manta_audit_file_path()
                .to_string_lossy()
                .to_string(),
        )
        .show_default(true)
        .allow_empty(true)
        .interact_text()
        .unwrap();

    let site: String = Input::new()
        .with_prompt("Please type site name")
        .default("alps".to_string())
        .show_default(true)
        .interact_text()
        .unwrap();

    let shasta_base_url: String = Input::new()
        .with_prompt("Please type site management plane URL")
        .default("https://api.cmn.alps.cscs.ch".to_string())
        .show_default(true)
        .interact_text()
        .unwrap();

    let k8s_api_url: String = Input::new()
        .with_prompt("Please type kubernetes api URL")
        .default("https://10.252.1.12:6442".to_string())
        .show_default(true)
        .interact_text()
        .unwrap();

    let vault_base_url: String = Input::new()
        .with_prompt("Please type Hashicorp Vault URL")
        .default("https://hashicorp-vault.cscs.ch:8200".to_string())
        .show_default(true)
        .interact_text()
        .unwrap();

    let vault_secret_path: String = Input::new()
        .with_prompt("Please type Hashicorp Vault secret path")
        .default("shasta".to_string())
        .show_default(true)
        .interact_text()
        .unwrap();

    let vault_role_id: String = Input::new()
        .with_prompt("Please type Hashicorp Vault role id")
        .default("b15517de-cabb-06ba-af98-633d216c6d99".to_string())
        .show_default(true)
        .interact_text()
        .unwrap();

    let root_ca_cert_file: String = Input::new()
        .with_prompt("Please type full path for the CA public certificate file")
        .default(
            get_default_mgmt_plane_ca_cert_file_path()
                .to_string_lossy()
                .to_string(),
        )
        .show_default(true)
        .interact_text()
        .unwrap();

    println!("Testing connectivity to CSM backend, please wait ...");

    let test_backend_api =
        mesa::common::authentication::test_connectivity_to_backend(&shasta_base_url).await;

    let socks5_proxy = if test_backend_api {
        println!("This machine can access CSM API, no need to setup SOCKS5 proxy");
        None
    } else {
        println!("This machine cannot access CSM API, configuring SOCKS5 proxy");

        // Get the right socks5 proxy value based on if client can reach backend api or not
        Some(
            Input::new()
                .with_prompt("Please type socks5 proxy URL")
                .default("socks5h://127.0.0.1:1080".to_string())
                .allow_empty(true)
                .interact_text()
                .unwrap(),
        )
    };

    let site_details = Site {
        socks5_proxy,
        shasta_base_url,
        k8s_api_url,
        vault_base_url,
        vault_secret_path,
        vault_role_id,
        root_ca_cert_file,
    };

    let mut site_hashmap = HashMap::new();
    site_hashmap.insert(site.clone(), site_details);

    let config_toml = MantaConfiguration {
        log,
        site,
        parent_hsm_group,
        audit_file,
        sites: site_hashmap,
    };

    let config_file_content = toml::to_string(&config_toml).unwrap();

    // Create configuration file on user's location, otherwise use default path
    let config_file_path = if let Some(config_file_path) = config_file_path_opt {
        PathBuf::from(config_file_path)
    } else {
        get_default_manta_config_file_path()
    };

    let mut config_file = File::create(config_file_path.clone()).unwrap();
    config_file
        .write_all(config_file_content.as_bytes())
        .unwrap();

    eprintln!(
        "Configuration file '{}' created",
        config_file_path.to_string_lossy()
    )
}

pub fn get_csm_root_cert_content(file_path: &str) -> Vec<u8> {
    let mut buf = Vec::new();
    let root_cert_file_rslt = File::open(file_path);

    let file_rslt = if root_cert_file_rslt.is_err() {
        let mut config_path = get_default_config_path();
        config_path.push(file_path);
        File::open(config_path)
    } else {
        root_cert_file_rslt
    };

    let mut file = file_rslt.expect("CA public root file cound not be found.");

    let _ = file.read_to_end(&mut buf);

    buf
}
