pub mod types;

use std::{
  collections::HashMap,
  fs::{self, File},
  io::{Read, Write},
  path::PathBuf,
};

use anyhow::Context;
use config::Config;
use dialoguer::{Input, Select};
use directories::ProjectDirs;
use manta_backend_dispatcher::types::{K8sAuth, K8sDetails};
use toml_edit::DocumentMut;
use types::{MantaConfiguration, Site};

use crate::common::{
  audit::Auditor,
  check_network_connectivity::check_network_connectivity_to_backend,
  kafka::Kafka,
};

/// Returns the XDG-compliant `ProjectDirs` for manta.
///
/// All path helpers in this module delegate to this function
/// so the qualifier/organization/application triple is defined
/// in exactly one place.
fn get_project_dirs() -> Result<ProjectDirs, anyhow::Error> {
  ProjectDirs::from(
    "local", /*qualifier*/
    "cscs",  /*organization*/
    "manta", /*application*/
  )
  .context(
    "Could not determine project directories \
     (home directory may not be set)",
  )
}

/// Returns the default manta config directory path
/// (e.g. `~/.config/manta/`).
pub(crate) fn get_default_config_path() -> Result<PathBuf, anyhow::Error> {
  Ok(PathBuf::from(get_project_dirs()?.config_dir()))
}

/// Returns the default manta config file path
/// (e.g. `~/.config/manta/config.toml`).
pub(crate) fn get_default_manta_config_file_path()
-> Result<PathBuf, anyhow::Error> {
  let mut path = get_default_config_path()?;
  path.push("config.toml");
  Ok(path)
}

/// Returns the default manta cache directory path
/// (e.g. `~/.cache/manta/`).
pub(crate) fn get_default_cache_path() -> Result<PathBuf, anyhow::Error> {
  Ok(PathBuf::from(get_project_dirs()?.cache_dir()))
}

/// Reads the manta configuration file and parses it as TOML.
///
/// Returns both the file path (for later writing) and the
/// parsed `DocumentMut`.
pub(crate) fn read_config_toml() -> Result<(PathBuf, DocumentMut), anyhow::Error>
{
  let path = get_default_manta_config_file_path()?;

  log::debug!(
    "Reading manta configuration from {}",
    path.to_string_lossy()
  );

  let content =
    fs::read_to_string(&path).context("Error reading configuration file")?;

  let doc = content
    .parse::<DocumentMut>()
    .context("Could not parse configuration file as TOML")?;

  Ok((path, doc))
}

/// Writes a `DocumentMut` back to the manta configuration file.
pub(crate) fn write_config_toml(
  path: &std::path::Path,
  doc: &DocumentMut,
) -> Result<(), anyhow::Error> {
  let mut file = std::fs::OpenOptions::new()
    .write(true)
    .truncate(true)
    .open(path)
    .context("Failed to open configuration file for writing")?;

  file
    .write_all(doc.to_string().as_bytes())
    .context("Failed to write configuration file")?;
  file.flush().context("Failed to flush configuration file")?;

  Ok(())
}

/// Read the root CA certificate from `file_path`, falling
/// back to the default config directory if the path is
/// relative.
pub fn get_csm_root_cert_content(
  file_path: &str,
) -> Result<Vec<u8>, anyhow::Error> {
  let mut buf = Vec::new();
  let root_cert_file_rslt = File::open(file_path);

  let file_rslt = if root_cert_file_rslt.is_err() {
    let mut config_path = get_default_config_path()?;
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
    Err(_) => Err(anyhow::anyhow!("CA public root file could not be found")),
  }
}

fn get_default_manta_audit_file_path() -> Result<PathBuf, anyhow::Error> {
  let mut log_file_path = PathBuf::from(get_project_dirs()?.data_dir());
  log_file_path.push("manta.log");
  Ok(log_file_path)
}

fn get_default_mgmt_plane_ca_cert_file_path() -> Result<PathBuf, anyhow::Error>
{
  let mut ca_cert_file_path = get_default_config_path()?;
  ca_cert_file_path.push("alps_root_cert.pem");
  Ok(ca_cert_file_path)
}

/// Get Manta configuration full path. Configuration may be the default one or specified by user.
/// This function also validates if the config file is TOML format
pub async fn get_config_file_path() -> Result<PathBuf, anyhow::Error> {
  // Get config file path from ENV var
  if let Ok(env_config_file_name) = std::env::var("MANTA_CONFIG") {
    let mut env_config_file = std::path::PathBuf::new();
    env_config_file.push(env_config_file_name);
    Ok(env_config_file)
  } else {
    // Get default config file path ($XDG_CONFIG/manta/config.toml
    get_default_manta_config_file_path()
  }
}

/// Reads configuration parameters related to manta from environment variables or file. If both
/// defiend, then environment variables takes preference
pub async fn get_configuration() -> Result<Config, anyhow::Error> {
  // Get config file path
  let config_file_path = get_config_file_path().await?;

  // If config file does not exists, then use config file generator to create a default config
  // file
  if !config_file_path.exists() {
    // Configuration file does not exists --> create a new configuration file
    log::info!(
      "Configuration file '{}' not found. Creating a new one.",
      config_file_path.to_string_lossy()
    );
    create_new_config_file(Some(&config_file_path)).await?;
  };

  // Process config file and check format (toml) is correct
  let config_file_path_str = config_file_path.to_str().ok_or_else(|| {
    anyhow::anyhow!("Configuration file path contains invalid UTF-8")
  })?;

  let config_file =
    config::File::new(config_file_path_str, config::FileFormat::Toml);

  // Process config file
  ::config::Config::builder()
    .add_source(config_file)
    .add_source(
      ::config::Environment::with_prefix("MANTA")
        .try_parsing(true)
        .prefix_separator("_"),
    )
    .build()
    .context("Could not process manta configuration file")
}

async fn create_new_config_file(
  config_file_path_opt: Option<&PathBuf>,
) -> Result<(), anyhow::Error> {
  eprintln!("Configuration file not found. Please introduce values below:");

  let log_level_values = vec!["error", "info", "warning", "debug", "trace"];

  let log_selection = Select::new()
    .with_prompt("Please select 'log verbosity' level from the list below")
    .items(&log_level_values)
    .default(0)
    .interact()
    .context("Failed to read log level selection")?;

  let log = log_level_values[log_selection].to_string();

  let parent_hsm_group = String::new();

  let audit_file: String = Input::new()
    .with_prompt("Please type full path for the audit file")
    .default(
      get_default_manta_audit_file_path()?
        .to_string_lossy()
        .to_string(),
    )
    .show_default(true)
    .allow_empty(true)
    .interact_text()
    .context("Failed to read audit file path")?;

  let site: String = Input::new()
    .with_prompt("Please type site name")
    .default("alps".to_string())
    .show_default(true)
    .interact_text()
    .context("Failed to read site name")?;

  let shasta_base_url: String = Input::new()
    .with_prompt("Please type site management plane URL")
    .default("https://api.cmn.alps.cscs.ch".to_string())
    .show_default(true)
    .interact_text()
    .context("Failed to read management plane URL")?;

  let k8s_api_url: String = Input::new()
    .with_prompt("Please type kubernetes api URL")
    .default("https://10.252.1.12:6442".to_string())
    .show_default(true)
    .interact_text()
    .context("Failed to read kubernetes API URL")?;

  let vault_base_url: String = Input::new()
    .with_prompt("Please type Hashicorp Vault URL")
    .default("https://hashicorp-vault.cscs.ch:8200".to_string())
    .show_default(true)
    .interact_text()
    .context("Failed to read Vault URL")?;

  let vault_secret_path: String = Input::new()
    .with_prompt("Please type Hashicorp Vault secret path")
    .default("shasta".to_string())
    .show_default(true)
    .interact_text()
    .context("Failed to read Vault secret path")?;

  let root_ca_cert_file: String = Input::new()
    .with_prompt("Please type full path for the CA public certificate file")
    .default(
      get_default_mgmt_plane_ca_cert_file_path()?
        .to_string_lossy()
        .to_string(),
    )
    .show_default(true)
    .interact_text()
    .context("Failed to read CA certificate file path")?;

  let backend_options = vec!["csm", "ochami"];

  let backend_selection = Select::new()
    .with_prompt("Please select 'backend' technology from the list below")
    .items(&backend_options)
    .default(0)
    .interact()
    .context("Failed to read backend selection")?;

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
    String::new()
  };

  // Create auditor only when both broker and topic are provided
  let auditor =
    if !audit_kafka_brokers.is_empty() && !audit_kafka_topic.is_empty() {
      let kafka = Kafka::new(vec![audit_kafka_brokers], audit_kafka_topic);

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
        .context("Failed to read SOCKS5 proxy URL")?,
    )
  };

  let k8s_auth = K8sAuth::Native {
    certificate_authority_data: String::new(),
    client_certificate_data: String::new(),
    client_key_data: String::new(),
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

  let config_file_content = toml::to_string(&config_toml)
    .context("Failed to serialize configuration to TOML")?;

  // Create configuration file on user's location, otherwise use default path
  //
  // Create PathBuf to store the manta config file specified by user or get default one
  let config_file_path = if let Some(config_file_path) = config_file_path_opt {
    PathBuf::from(config_file_path)
  } else {
    get_default_manta_config_file_path()?
  };

  // Create directories if needed
  let parent_dir = config_file_path
    .parent()
    .context("Configuration file path has no parent directory")?;
  std::fs::create_dir_all(parent_dir).with_context(|| {
    format!(
      "Failed to create config directory '{}'",
      parent_dir.display()
    )
  })?;

  // Create manta config file
  let mut config_file = File::create(&config_file_path).with_context(|| {
    format!(
      "Failed to create config file '{}'",
      config_file_path.display()
    )
  })?;
  config_file
    .write_all(config_file_content.as_bytes())
    .with_context(|| {
      format!(
        "Failed to write config file '{}'",
        config_file_path.display()
      )
    })?;

  log::info!(
    "Configuration file '{}' created",
    config_file_path.to_string_lossy()
  );

  Ok(())
}
