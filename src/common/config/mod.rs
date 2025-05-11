pub mod types;

use std::{
  collections::HashMap,
  fs::File,
  io::{Read, Write},
  path::PathBuf,
};

use config::Config;
use dialoguer::{Input, Select};
use directories::ProjectDirs;
use manta_backend_dispatcher::{
  error::Error,
  types::{K8sAuth, K8sDetails},
};
use types::{MantaConfiguration, Site};

use crate::common::{
  audit::Auditor,
  check_network_connectivity::check_network_connectivity_to_backend,
  kafka::Kafka,
};

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

pub fn get_csm_root_cert_content(file_path: &str) -> Result<Vec<u8>, Error> {
  let mut buf = Vec::new();
  let root_cert_file_rslt = File::open(file_path);

  let file_rslt = if root_cert_file_rslt.is_err() {
    let mut config_path = get_default_config_path();
    config_path.push(file_path);
    File::open(config_path)
  } else {
    root_cert_file_rslt
  };

  match file_rslt {
    Ok(mut file) => {
      let _ = file.read_to_end(&mut buf);

      Ok(buf)
    }
    Err(_) => Err(Error::Message(
      "CA public root file cound not be found.".to_string(),
    )),
  }
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
pub async fn get_config_file_path() -> PathBuf {
  // Get config file path from ENV var
  if let Ok(env_config_file_name) = std::env::var("MANTA_CONFIG") {
    let mut env_config_file = std::path::PathBuf::new();
    env_config_file.push(env_config_file_name);
    env_config_file
  } else {
    // Get default config file path ($XDG_CONFIG/manta/config.toml
    get_default_manta_config_file_path()
  }
}

/// Reads configuration parameters related to manta from environment variables or file. If both
/// defiend, then environment variables takes preference
pub async fn get_configuration() -> Config {
  // Get config file path
  let config_file_path = get_config_file_path().await;

  // If config file does not exists, then use config file generator to create a default config
  // file
  if !config_file_path.exists() {
    // Configuration file does not exists --> create a new configuration file
    eprintln!(
      "Configuration file '{}' not found. Creating a new one.",
      config_file_path.to_string_lossy()
    );
    create_new_config_file(Some(&config_file_path)).await;
  };

  // Process config file and check format (toml) is correct
  let config_file = config::File::new(
    config_file_path
      .to_str()
      .expect("Configuration file name not defined"),
    config::FileFormat::Toml,
  );

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

  let log_level_values = vec!["error", "info", "warning", "debug", "trace"];

  let log_selection = Select::new()
    .with_prompt("Please select 'log verbosity' level from the list below")
    .items(&log_level_values)
    .default(0)
    .interact()
    .unwrap();

  let log = log_level_values[log_selection].to_string();

  let parent_hsm_group = "".to_string();

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

  /* let vault_role_id: String = Input::new()
  .with_prompt("Please type Hashicorp Vault role id")
  .default("b15517de-cabb-06ba-af98-633d216c6d99".to_string())
  .show_default(true)
  .interact_text()
  .unwrap(); */

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

  let backend_options = vec!["csm", "ochami"];

  let backend_selection = Select::new()
    .with_prompt("Please select 'backend' technology from the list below")
    .items(&backend_options)
    .default(0)
    .interact()
    .unwrap();

  let backend = backend_options[backend_selection].to_string();

  // Broker is optional value
  let audit_kafka_brokers: String = Input::new()
    .with_prompt("Please type kafka broker to send audit logs")
    .default("kafka.o11y.cscs.ch:9095".to_string())
    .show_default(true)
    .interact_text()
    .unwrap_or_default();

  let audit_kafka_topic: String = if !audit_kafka_brokers.is_empty() {
    Input::new()
      .with_prompt("Please type kafka topic to send audit logs")
      .default("test-topic".to_string())
      .show_default(true)
      .interact_text()
      .unwrap_or_default()
  } else {
    "".to_string()
  };

  // If both kafka broker and topic are empty, then auditor is None
  let auditor =
    if audit_kafka_brokers.is_empty() && audit_kafka_topic.is_empty() {
      let kafka = Kafka {
        brokers: vec![audit_kafka_brokers],
        topic: audit_kafka_topic,
      };

      Some(Auditor { kafka })
    } else {
      None
    };

  println!("Testing connectivity to CSM backend, please wait ...");

  let test_backend_api =
    check_network_connectivity_to_backend(&shasta_base_url).await;

  let socks5_proxy = if test_backend_api.is_ok() {
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

  let k8s_auth = K8sAuth::Native {
    certificate_authority_data: "".to_string(),
    client_certificate_data: "".to_string(),
    client_key_data: "".to_string(),
  };

  let k8s_details = K8sDetails {
    api_url: k8s_api_url.clone(),
    authentication: k8s_auth,
  };

  let site_details = Site {
    socks5_proxy,
    shasta_base_url,
    // k8s_api_url: Some(k8s_api_url),
    vault_base_url: Some(vault_base_url),
    vault_secret_path: Some(vault_secret_path),
    // vault_role_id: Some(vault_role_id),
    root_ca_cert_file,
    k8s: Some(k8s_details),
    backend,
  };

  let mut site_hashmap = HashMap::new();
  site_hashmap.insert(site.clone(), site_details);

  let config_toml = MantaConfiguration {
    log,
    site,
    parent_hsm_group,
    audit_file,
    sites: site_hashmap,
    auditor,
  };

  let config_file_content = toml::to_string(&config_toml).unwrap();

  // Create configuration file on user's location, otherwise use default path
  //
  // Create PathBuf to store the manta config file specified by user or get default one
  let config_file_path = if let Some(config_file_path) = config_file_path_opt {
    PathBuf::from(config_file_path)
  } else {
    get_default_manta_config_file_path()
  };

  // Create directories if needed
  let _ = std::fs::create_dir_all(config_file_path.parent().unwrap());

  // Create manta config file
  let mut config_file = File::create(config_file_path.clone()).unwrap();
  config_file
    .write_all(config_file_content.as_bytes())
    .unwrap();

  eprintln!(
    "Configuration file '{}' created",
    config_file_path.to_string_lossy()
  )
}
