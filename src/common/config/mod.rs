pub mod types;

use std::{
  collections::HashMap,
  fs::{self, File},
  io::{Read, Write},
  path::PathBuf,
};

use config::Config;
use dialoguer::{Input, Select};
use directories::ProjectDirs;
use manta_backend_dispatcher::{error::Error, types::{K8sAuth, K8sDetails}};
use toml_edit::DocumentMut;
use types::{BackendTechnology, MantaConfiguration, Site};

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
fn get_project_dirs() -> Result<ProjectDirs, Error> {
  ProjectDirs::from(
    "local", /*qualifier*/
    "cscs",  /*organization*/
    "manta", /*application*/
  )
  .ok_or_else(|| {
    Error::Message(
      "Could not determine project directories \
       (home directory may not be set)"
        .to_string(),
    )
  })
}

/// Returns the default manta config directory path
/// (e.g. `~/.config/manta/`).
pub(crate) fn get_default_config_path() -> Result<PathBuf, Error> {
  Ok(PathBuf::from(get_project_dirs()?.config_dir()))
}

/// Returns the default manta config file path
/// (e.g. `~/.config/manta/config.toml`).
pub(crate) fn get_default_manta_config_file_path() -> Result<PathBuf, Error> {
  let mut path = get_default_config_path()?;
  path.push("config.toml");
  Ok(path)
}

/// Returns the default manta cache directory path
/// (e.g. `~/.cache/manta/`).
pub(crate) fn get_default_cache_path() -> Result<PathBuf, Error> {
  Ok(PathBuf::from(get_project_dirs()?.cache_dir()))
}

/// Reads the manta configuration file and parses it as TOML.
///
/// Returns both the file path (for later writing) and the
/// parsed `DocumentMut`.
pub(crate) fn read_config_toml() -> Result<(PathBuf, DocumentMut), Error> {
  let path = get_default_manta_config_file_path()?;

  tracing::debug!(
    "Reading manta configuration from {}",
    path.to_string_lossy()
  );

  let content = fs::read_to_string(&path)?;

  let doc = content.parse::<DocumentMut>()?;

  Ok((path, doc))
}

/// Writes a `DocumentMut` back to the manta configuration file.
pub(crate) fn write_config_toml(
  path: &std::path::Path,
  doc: &DocumentMut,
) -> Result<(), Error> {
  let mut file = std::fs::OpenOptions::new()
    .write(true)
    .truncate(true)
    .open(path)?;

  file.write_all(doc.to_string().as_bytes())?;
  file.flush()?;

  Ok(())
}

/// Read the root CA certificate from `file_path`, falling
/// back to the default config directory if the path is
/// relative.
pub fn get_csm_root_cert_content(
  file_path: &str,
) -> Result<Vec<u8>, Error> {
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
      file.read_to_end(&mut buf)?;
      Ok(buf)
    }
    Err(_) => Err(Error::Message(
      "CA public root file could not be found".to_string(),
    )),
  }
}

fn get_default_manta_audit_file_path() -> Result<PathBuf, Error> {
  let mut log_file_path = PathBuf::from(get_project_dirs()?.data_dir());
  log_file_path.push("manta.log");
  Ok(log_file_path)
}

fn get_default_mgmt_plane_ca_cert_file_path() -> Result<PathBuf, Error> {
  let mut ca_cert_file_path = get_default_config_path()?;
  ca_cert_file_path.push("alps_root_cert.pem");
  Ok(ca_cert_file_path)
}

/// Get Manta configuration full path. Configuration may be the default one or specified by user.
/// This function also validates if the config file is TOML format
pub async fn get_config_file_path() -> Result<PathBuf, Error> {
  if let Ok(env_config_file_name) = std::env::var("MANTA_CONFIG") {
    let mut env_config_file = std::path::PathBuf::new();
    env_config_file.push(env_config_file_name);
    Ok(env_config_file)
  } else {
    get_default_manta_config_file_path()
  }
}

/// Reads configuration parameters related to manta from environment variables or file. If both
/// defiend, then environment variables takes preference
pub async fn get_configuration() -> Result<Config, Error> {
  let config_file_path = get_config_file_path().await?;

  if !config_file_path.exists() {
    tracing::info!(
      "Configuration file '{}' not found. Creating a new one.",
      config_file_path.to_string_lossy()
    );
    create_new_config_file(Some(&config_file_path)).await?;
  };

  let config_file_path_str = config_file_path.to_str().ok_or_else(|| {
    Error::Message(
      "Configuration file path contains invalid UTF-8".to_string(),
    )
  })?;

  let config_file =
    config::File::new(config_file_path_str, config::FileFormat::Toml);

  ::config::Config::builder()
    .add_source(config_file)
    .add_source(
      ::config::Environment::with_prefix("MANTA")
        .try_parsing(true)
        .prefix_separator("_"),
    )
    .build()
    .map_err(Error::ConfigError)
}

/// Prompt the user for a string value with a default.
fn prompt_string(prompt: &str, default: &str) -> Result<String, Error> {
  Ok(
    Input::new()
      .with_prompt(prompt)
      .default(default.to_string())
      .show_default(true)
      .interact_text()?,
  )
}

/// Prompt the user for a string value with a default,
/// allowing empty input.
fn prompt_string_allow_empty(
  prompt: &str,
  default: &str,
) -> Result<String, Error> {
  Ok(
    Input::new()
      .with_prompt(prompt)
      .default(default.to_string())
      .show_default(true)
      .allow_empty(true)
      .interact_text()?,
  )
}

async fn create_new_config_file(
  config_file_path_opt: Option<&PathBuf>,
) -> Result<(), Error> {
  eprintln!("Configuration file not found. Please introduce values below:");

  let log_level_values = vec!["error", "info", "warn", "debug", "trace"];

  let log_selection = Select::new()
    .with_prompt("Please select 'log verbosity' level from the list below")
    .items(&log_level_values)
    .default(0)
    .interact()?;

  let log = log_level_values[log_selection].to_string();

  let parent_hsm_group = String::new();

  let audit_file: String = prompt_string_allow_empty(
    "Please type full path for the audit file",
    &get_default_manta_audit_file_path()?
      .to_string_lossy()
      .to_string(),
  )?;

  let site: String = prompt_string("Please type site name", "alps")?;

  let shasta_base_url: String = prompt_string(
    "Please type site management plane URL",
    "https://api.cmn.alps.cscs.ch",
  )?;

  let k8s_api_url: String = prompt_string(
    "Please type kubernetes api URL",
    "https://10.252.1.12:6442",
  )?;

  let vault_base_url: String = prompt_string(
    "Please type Hashicorp Vault URL",
    "https://hashicorp-vault.cscs.ch:8200",
  )?;

  let vault_secret_path: String = prompt_string(
    "Please type Hashicorp Vault secret path",
    "shasta",
  )?;

  let root_ca_cert_file: String = prompt_string(
    "Please type full path for the CA public certificate file",
    &get_default_mgmt_plane_ca_cert_file_path()?
      .to_string_lossy()
      .to_string(),
  )?;

  let backend_options = [BackendTechnology::Csm, BackendTechnology::Ochami];
  let backend_option_labels = ["csm", "ochami"];

  let backend_selection = Select::new()
    .with_prompt("Please select 'backend' technology from the list below")
    .items(&backend_option_labels)
    .default(0)
    .interact()?;

  let backend = backend_options[backend_selection].clone();

  let audit_kafka_brokers: String = prompt_string_allow_empty(
    "Please type kafka broker to send audit logs",
    "kafka.o11y.cscs.ch:9095",
  )
  .unwrap_or_default();

  let audit_kafka_topic: String = if !audit_kafka_brokers.is_empty() {
    prompt_string(
      "Please type kafka topic to send audit logs",
      "test-topic",
    )
    .unwrap_or_default()
  } else {
    String::new()
  };

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
    Some(prompt_string_allow_empty(
      "Please type socks5 proxy URL",
      "socks5h://127.0.0.1:1080",
    )?)
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
    vault_base_url: Some(vault_base_url),
    vault_secret_path: Some(vault_secret_path),
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

  let config_file_content = toml::to_string(&config_toml)?;

  let config_file_path = if let Some(config_file_path) = config_file_path_opt {
    PathBuf::from(config_file_path)
  } else {
    get_default_manta_config_file_path()?
  };

  let parent_dir = config_file_path.parent().ok_or_else(|| {
    Error::Message(
      "Configuration file path has no parent directory".to_string(),
    )
  })?;
  std::fs::create_dir_all(parent_dir).map_err(|e| {
    Error::Message(format!(
      "Failed to create config directory '{}': {}",
      parent_dir.display(),
      e
    ))
  })?;

  let mut config_file =
    File::create(&config_file_path).map_err(|e| {
      Error::Message(format!(
        "Failed to create config file '{}': {}",
        config_file_path.display(),
        e
      ))
    })?;
  config_file
    .write_all(config_file_content.as_bytes())
    .map_err(|e| {
      Error::Message(format!(
        "Failed to write config file '{}': {}",
        config_file_path.display(),
        e
      ))
    })?;

  tracing::info!(
    "Configuration file '{}' created",
    config_file_path.to_string_lossy()
  );

  Ok(())
}
