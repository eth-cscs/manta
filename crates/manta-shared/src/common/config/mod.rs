pub mod types;

use std::{
  fs::{self, File},
  io::{Read, Write},
  path::PathBuf,
};

use config::Config;
use directories::ProjectDirs;
use manta_backend_dispatcher::error::Error;
use toml_edit::DocumentMut;

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
    Error::MissingField(
      "Could not determine project directories \
       (home directory may not be set)"
        .to_string(),
    )
  })
}

/// Returns the default manta config directory path
/// (e.g. `~/.config/manta/`).
pub fn get_default_config_path() -> Result<PathBuf, Error> {
  Ok(PathBuf::from(get_project_dirs()?.config_dir()))
}

/// Returns the path of the *legacy* unified config file
/// (e.g. `~/.config/manta/config.toml`). Used only by
/// `missing_config_message` to detect when a user is migrating from
/// the pre-split layout — neither binary ever reads from this path
/// at startup.
pub fn get_default_manta_config_file_path() -> Result<PathBuf, Error> {
  let mut path = get_default_config_path()?;
  path.push("config.toml");
  Ok(path)
}

/// Returns the default CLI config file path
/// (e.g. `~/.config/manta/cli.toml`).
pub fn get_default_manta_cli_config_file_path() -> Result<PathBuf, Error> {
  let mut path = get_default_config_path()?;
  path.push("cli.toml");
  Ok(path)
}

/// Returns the default server config file path
/// (e.g. `~/.config/manta/server.toml`).
pub fn get_default_manta_server_config_file_path() -> Result<PathBuf, Error> {
  let mut path = get_default_config_path()?;
  path.push("server.toml");
  Ok(path)
}

/// Returns the default manta cache directory path
/// (e.g. `~/.cache/manta/`).
pub fn get_default_cache_path() -> Result<PathBuf, Error> {
  Ok(PathBuf::from(get_project_dirs()?.cache_dir()))
}

/// Reads the manta CLI configuration file (`cli.toml`) and parses it as
/// TOML, honoring `MANTA_CLI_CONFIG`.
///
/// Returns both the file path (for later writing) and the
/// parsed `DocumentMut`.
pub fn read_config_toml() -> Result<(PathBuf, DocumentMut), Error> {
  let path = get_cli_config_file_path()?;

  tracing::debug!(
    "Reading manta CLI configuration from {}",
    path.to_string_lossy()
  );

  let content = fs::read_to_string(&path)?;

  let doc = content.parse::<DocumentMut>()?;

  Ok((path, doc))
}

/// Writes a `DocumentMut` back to the manta configuration file.
pub fn write_config_toml(
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
pub fn get_csm_root_cert_content(file_path: &str) -> Result<Vec<u8>, Error> {
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
    Err(_) => Err(Error::NotFound(
      "CA public root file could not be found".to_string(),
    )),
  }
}

/// Returns the CLI config file path, honoring `MANTA_CLI_CONFIG` if set.
pub fn get_cli_config_file_path() -> Result<PathBuf, Error> {
  if let Ok(env_path) = std::env::var("MANTA_CLI_CONFIG") {
    Ok(PathBuf::from(env_path))
  } else {
    get_default_manta_cli_config_file_path()
  }
}

/// Returns the server config file path, honoring `MANTA_SERVER_CONFIG` if set.
pub fn get_server_config_file_path() -> Result<PathBuf, Error> {
  if let Ok(env_path) = std::env::var("MANTA_SERVER_CONFIG") {
    Ok(PathBuf::from(env_path))
  } else {
    get_default_manta_server_config_file_path()
  }
}

/// Minimal CLI config sample shown in the NotFound error.
const CLI_CONFIG_SAMPLE: &str = r#"log = "info"
audit_file = "/tmp/manta-cli-audit.log"
site = "<site_name>"
parent_hsm_group = ""
manta_server_url = "https://manta-server.example.com:8443"

[sites.<site_name>]
backend = "csm"                 # or "ochami"
shasta_base_url = "https://api.example.com"
root_ca_cert_file = "alps_root_cert.pem"
"#;

/// Migration mapping shown when a legacy `config.toml` is detected.
const CLI_CONFIG_MIGRATION: &str = "\
Migration from ~/.config/manta/config.toml:
  copy these fields verbatim:        log, audit_file, site, parent_hsm_group,
                                     auditor, sites
  add CLI-only (now required):       manta_server_url = \"https://...\"
                                     (CLI talks only to the manta server)
  drop (no longer recognised):       sites.<X>.manta_server_url
  do not copy (server-only fields):  the [server] section belongs in
                                     server.toml, not cli.toml";

/// Minimal server config sample shown in the NotFound error.
const SERVER_CONFIG_SAMPLE: &str = r#"log = "info"
audit_file = "/var/log/manta/server-audit.log"

[server]
listen_address = "0.0.0.0"
port = 8443
cert = "/path/to/server.crt"
key = "/path/to/server.key"
console_inactivity_timeout_secs = 1800

[sites.<site_name>]
backend = "csm"
shasta_base_url = "https://api.example.com"
root_ca_cert_file = "/path/to/alps_root_cert.pem"
"#;

/// Migration mapping shown when a legacy `config.toml` is detected.
const SERVER_CONFIG_MIGRATION: &str = "\
Migration from ~/.config/manta/config.toml:
  copy these fields verbatim:        log, audit_file, auditor, sites
  add new [server] section:          listen_address, port, cert, key,
                                     console_inactivity_timeout_secs
  drop (CLI-only):                   site, parent_hsm_group, hsm_group,
                                     manta_server_url
  drop (no longer recognised):       sites.<X>.manta_server_url";

fn missing_config_message(
  binary: &str,
  expected_path: &std::path::Path,
  sample: &str,
  migration: &str,
) -> String {
  let legacy_exists = get_default_manta_config_file_path()
    .map(|p| p.exists())
    .unwrap_or(false);
  let mut msg = format!(
    "{binary} configuration file '{}' not found.\n\nMinimal example:\n\n{sample}",
    expected_path.to_string_lossy()
  );
  if legacy_exists {
    msg.push('\n');
    msg.push_str(migration);
  }
  msg
}

/// Load `cli.toml`. Fails loudly if the file is missing; the error
/// message includes a minimal example and (when a legacy config.toml is
/// detected) a field-by-field migration mapping.
pub fn get_cli_configuration() -> Result<Config, Error> {
  let path = get_cli_config_file_path()?;
  if !path.exists() {
    return Err(Error::NotFound(missing_config_message(
      "CLI",
      &path,
      CLI_CONFIG_SAMPLE,
      CLI_CONFIG_MIGRATION,
    )));
  }
  let path_str = path.to_str().ok_or_else(|| {
    Error::MissingField(
      "CLI configuration file path contains invalid UTF-8".to_string(),
    )
  })?;
  ::config::Config::builder()
    .add_source(::config::File::new(path_str, ::config::FileFormat::Toml))
    .add_source(
      ::config::Environment::with_prefix("MANTA")
        .try_parsing(true)
        .prefix_separator("_"),
    )
    .build()
    .map_err(Error::ConfigError)
}

/// Load `server.toml`. Fails loudly if the file is missing; the error
/// message includes a minimal example and (when a legacy config.toml is
/// detected) a field-by-field migration mapping.
pub fn get_server_configuration() -> Result<Config, Error> {
  let path = get_server_config_file_path()?;
  if !path.exists() {
    return Err(Error::NotFound(missing_config_message(
      "Server",
      &path,
      SERVER_CONFIG_SAMPLE,
      SERVER_CONFIG_MIGRATION,
    )));
  }
  let path_str = path.to_str().ok_or_else(|| {
    Error::MissingField(
      "Server configuration file path contains invalid UTF-8".to_string(),
    )
  })?;
  ::config::Config::builder()
    .add_source(::config::File::new(path_str, ::config::FileFormat::Toml))
    .add_source(
      ::config::Environment::with_prefix("MANTA")
        .try_parsing(true)
        .prefix_separator("_"),
    )
    .build()
    .map_err(Error::ConfigError)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn default_cli_config_path_ends_with_cli_toml() {
    let path = get_default_manta_cli_config_file_path().unwrap();
    assert_eq!(path.file_name().unwrap(), "cli.toml");
  }

  #[test]
  fn default_server_config_path_ends_with_server_toml() {
    let path = get_default_manta_server_config_file_path().unwrap();
    assert_eq!(path.file_name().unwrap(), "server.toml");
  }

  #[test]
  fn default_legacy_config_path_ends_with_config_toml() {
    let path = get_default_manta_config_file_path().unwrap();
    assert_eq!(path.file_name().unwrap(), "config.toml");
  }

  #[test]
  fn cli_and_server_default_paths_share_parent() {
    let cli = get_default_manta_cli_config_file_path().unwrap();
    let server = get_default_manta_server_config_file_path().unwrap();
    assert_eq!(cli.parent(), server.parent());
  }
}
