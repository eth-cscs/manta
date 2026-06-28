//! Config-file loaders for `cli.toml` and `server.toml`.
//!
//! This module owns three concerns:
//!
//! 1. **Paths** — XDG-resolved defaults via [`get_default_config_path`],
//!    with per-binary overrides from `MANTA_CLI_CONFIG` /
//!    `MANTA_SERVER_CONFIG`. See [`get_cli_config_file_path`] and
//!    [`get_server_config_file_path`].
//! 2. **Loading** — [`get_cli_configuration`] and
//!    [`get_server_configuration`] parse the TOML file and merge any
//!    `MANTA_*`-prefixed environment variables on top, returning an
//!    untyped `::config::Config`. The typed deserialisation targets
//!    live with each binary (`CliConfiguration` in `manta-cli`,
//!    `ServerConfiguration` in `manta-server`) so this module stays
//!    agnostic of either schema.
//! 3. **In-place editing** — [`read_config_toml`] / [`write_config_toml`]
//!    expose a `toml_edit::DocumentMut` view for `manta config set`
//!    and friends, preserving comments and formatting.
//!
//! Missing-file errors are intentionally rich: the
//! [`MantaError::NotFound`] message includes a minimal example file
//! and, when a legacy `~/.config/manta/config.toml` is detected, a
//! field-by-field migration mapping.
//!
//! [`MantaError::NotFound`]: crate::common::error::MantaError::NotFound

use std::{
  fs::{self, File},
  io::{Read, Write},
  path::PathBuf,
};

use crate::common::error::MantaError as Error;
use config::Config;
use directories::ProjectDirs;
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
///
/// # Errors
///
/// Returns [`MantaError::MissingField`] when the platform cannot resolve
/// a project-dirs triple (typically because `$HOME` is unset).
///
/// [`MantaError::MissingField`]: crate::common::error::MantaError::MissingField
pub fn get_default_config_path() -> Result<PathBuf, Error> {
  Ok(PathBuf::from(get_project_dirs()?.config_dir()))
}

/// Appends `filename` to the default config directory path.
fn default_config_file(filename: &str) -> Result<PathBuf, Error> {
  let mut path = get_default_config_path()?;
  path.push(filename);
  Ok(path)
}

/// Returns the path of the *legacy* unified config file
/// (e.g. `~/.config/manta/config.toml`). Used only by
/// `missing_config_message` to detect when a user is migrating from
/// the pre-split layout — neither binary ever reads from this path
/// at startup.
pub fn get_default_manta_config_file_path() -> Result<PathBuf, Error> {
  default_config_file("config.toml")
}

/// Returns the default CLI config file path
/// (e.g. `~/.config/manta/cli.toml`).
pub fn get_default_manta_cli_config_file_path() -> Result<PathBuf, Error> {
  default_config_file("cli.toml")
}

/// Returns the default server config file path
/// (e.g. `~/.config/manta/server.toml`).
pub fn get_default_manta_server_config_file_path() -> Result<PathBuf, Error> {
  default_config_file("server.toml")
}

/// Returns the default manta cache directory path
/// (e.g. `~/.cache/manta/`).
pub fn get_default_cache_path() -> Result<PathBuf, Error> {
  Ok(PathBuf::from(get_project_dirs()?.cache_dir()))
}

/// Reads the manta CLI configuration file (`cli.toml`) and parses it as
/// TOML, honoring `MANTA_CLI_CONFIG`.
///
/// Returns both the file path (for later writing via
/// [`write_config_toml`]) and the parsed `DocumentMut`, which
/// preserves comments and formatting for in-place edits.
///
/// # Errors
///
/// - [`MantaError::IoError`] if the file cannot be read.
/// - [`MantaError::TomlEditError`] if the contents are not valid TOML.
///
/// [`MantaError::IoError`]: crate::common::error::MantaError::IoError
/// [`MantaError::TomlEditError`]: crate::common::error::MantaError::TomlEditError
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
///
/// Opens `path` with `write | truncate` and replaces its contents with
/// the document's serialised form. Paired with [`read_config_toml`]
/// for round-tripping `manta config set` edits.
///
/// # Errors
///
/// Returns [`MantaError::IoError`] if the file cannot be opened,
/// written to, or flushed.
///
/// [`MantaError::IoError`]: crate::common::error::MantaError::IoError
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
///
/// # Errors
///
/// - [`MantaError::NotFound`] if neither the literal nor the
///   config-directory-relative path resolves to a readable file.
/// - [`MantaError::IoError`] if a candidate file opens but cannot be
///   read to completion.
///
/// [`MantaError::NotFound`]: crate::common::error::MantaError::NotFound
/// [`MantaError::IoError`]: crate::common::error::MantaError::IoError
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
///
/// When the env var is present its value wins verbatim (no validation,
/// no relative-path resolution); otherwise the XDG default
/// (`~/.config/manta/cli.toml` on Linux) is returned.
///
/// # Errors
///
/// Returns [`MantaError::MissingField`] if `MANTA_CLI_CONFIG` is unset
/// *and* the platform cannot resolve a project-dirs triple. The
/// env-var branch is infallible.
///
/// # Examples
///
/// ```no_run
/// use manta_shared::common::config::get_cli_config_file_path;
///
/// // SAFETY: doc-tests run single-threaded.
/// unsafe { std::env::set_var("MANTA_CLI_CONFIG", "/etc/manta/cli.toml") };
/// let path = get_cli_config_file_path().unwrap();
/// assert_eq!(path, std::path::PathBuf::from("/etc/manta/cli.toml"));
/// ```
///
/// [`MantaError::MissingField`]: crate::common::error::MantaError::MissingField
pub fn get_cli_config_file_path() -> Result<PathBuf, Error> {
  if let Ok(env_path) = std::env::var("MANTA_CLI_CONFIG") {
    Ok(PathBuf::from(env_path))
  } else {
    get_default_manta_cli_config_file_path()
  }
}

/// Returns the server config file path, honoring `MANTA_SERVER_CONFIG` if set.
///
/// Symmetric to [`get_cli_config_file_path`]: env-var wins verbatim,
/// otherwise XDG default (`~/.config/manta/server.toml` on Linux).
///
/// # Errors
///
/// Returns [`MantaError::MissingField`] if `MANTA_SERVER_CONFIG` is
/// unset *and* the platform cannot resolve a project-dirs triple.
///
/// # Examples
///
/// ```no_run
/// use manta_shared::common::config::get_server_config_file_path;
///
/// // SAFETY: doc-tests run single-threaded.
/// unsafe { std::env::set_var("MANTA_SERVER_CONFIG", "/etc/manta/server.toml") };
/// let path = get_server_config_file_path().unwrap();
/// assert_eq!(path, std::path::PathBuf::from("/etc/manta/server.toml"));
/// ```
///
/// [`MantaError::MissingField`]: crate::common::error::MantaError::MissingField
pub fn get_server_config_file_path() -> Result<PathBuf, Error> {
  if let Ok(env_path) = std::env::var("MANTA_SERVER_CONFIG") {
    Ok(PathBuf::from(env_path))
  } else {
    get_default_manta_server_config_file_path()
  }
}

/// Minimal CLI config sample shown in the NotFound error.
const CLI_CONFIG_SAMPLE: &str = r#"log = "info"
# Optional: the active site (`X-Manta-Site` header). Omit it and pass
# `--site <name>` per invocation instead; the server validates the name.
site = "<site_name>"
manta_server_url = "https://manta-server.example.com:8443"

# Timeout knobs. Values shown are the built-in defaults — delete a
# line to fall back to the default, or change the value to override.
#
# Per-request HTTP timeout reaching `manta_server_url` (seconds).
# Default 300 caps REST calls. Streams (SSE log tail, WS console)
# are unlimited when this is absent; setting it caps streams too.
request_timeout_secs             = 300
#
# `manta power on/off/reset`: poll interval (seconds) and max
# attempts before giving up. 300 × 3 s = 15 min total wait.
power_poll_interval_secs         = 3
power_max_poll_attempts          = 300
#
# `manta apply sat-file`: poll interval, overall hard cap, and
# cap on consecutive "session not yet visible" responses (seconds).
sat_file_poll_interval_secs      = 10
sat_file_poll_budget_secs        = 14400   # 4 hours
sat_file_not_visible_budget_secs = 300     # 5 minutes

[sites.<site_name>]
backend = "csm"                 # or "ochami"
shasta_base_url = "https://api.example.com"
root_ca_cert_file = "alps_root_cert.pem"
"#;

/// Migration mapping shown when a legacy `config.toml` is detected.
const CLI_CONFIG_MIGRATION: &str = "\
Migration from ~/.config/manta/config.toml:
  copy these fields verbatim:        log, site, auditor, sites
  add CLI-only (now required):       manta_server_url = \"https://...\"
                                     (CLI talks only to the manta server)
  drop (no longer recognised):       sites.<X>.manta_server_url, audit_file
  do not copy (server-only fields):  the [server] section belongs in
                                     server.toml, not cli.toml";

/// Minimal server config sample shown in the NotFound error.
const SERVER_CONFIG_SAMPLE: &str = r#"log = "info"

[server]
listen_address = "0.0.0.0"
port = 8443
cert = "/path/to/server.crt"
key = "/path/to/server.key"
console_inactivity_timeout_secs = 1800
auth_rate_limit_per_minute      = 60     # per source IP for /auth/*; omit to disable
# Values shown for the two timeout knobs are the built-in defaults —
# delete a line to fall back to the default, or change to override.
request_timeout_secs            = 300    # global per-route timeout; returns 408 on expiry
shutdown_grace_period_secs      = 30     # drain window after SIGTERM / Ctrl+C; matches k8s terminationGracePeriodSeconds
# allow_http = false                     # opt in to plain-HTTP listen when no cert/key is set
                                         #   (e.g. TLS terminated upstream). Default fail-closed.
# Filesystem root for POST /migrate/{backup,restore}. Required for those
# endpoints to work — the server will reject migrate requests with 400
# while this is unset. Must be an absolute path to an existing directory.
# migrate_backup_root = "/var/lib/manta/migrate"

# [auditor.kafka]                        # optional: enable Kafka audit emission
# brokers            = ["kafka.example.com:9092"]
# topic              = "manta-audit"
# message_timeout_ms = 5000              # librdkafka per-message delivery deadline; default 5000
# delivery_wait_secs = 0                 # how long produce_message blocks; 0 = fire-and-forget (default)

[sites.<site_name>]
backend = "csm"
shasta_base_url = "https://api.example.com"
root_ca_cert_file = "/path/to/alps_root_cert.pem"
"#;

/// Migration mapping shown when a legacy `config.toml` is detected.
const SERVER_CONFIG_MIGRATION: &str = "\
Migration from ~/.config/manta/config.toml:
  copy these fields verbatim:        log, auditor, sites
  add new [server] section:          listen_address, port, cert, key,
                                     console_inactivity_timeout_secs
  drop (CLI-only):                   site, hsm_group, manta_server_url
  drop (no longer recognised):       sites.<X>.manta_server_url, audit_file";

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

/// Shared TOML + env-var loading logic.
///
/// Checks that `path` exists, converts it to a UTF-8 string, then
/// runs the standard `Config::builder` chain (file + `MANTA_*` env
/// vars). `label` is used only in error messages ("CLI" or "Server").
fn load_config(
  label: &str,
  path: PathBuf,
  sample: &str,
  migration: &str,
) -> Result<Config, Error> {
  if !path.exists() {
    return Err(Error::NotFound(missing_config_message(
      label, &path, sample, migration,
    )));
  }
  let path_str = path.to_str().ok_or_else(|| {
    Error::MissingField(format!(
      "{label} configuration file path contains invalid UTF-8"
    ))
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

/// Load `cli.toml`. Fails loudly if the file is missing; the error
/// message includes a minimal example and (when a legacy config.toml is
/// detected) a field-by-field migration mapping.
///
/// Reads the file at [`get_cli_config_file_path`] and layers
/// `MANTA_*`-prefixed environment variables on top (env wins).
///
/// # Errors
///
/// - [`MantaError::NotFound`] when the resolved config file does not
///   exist on disk; the message embeds [`CLI_CONFIG_SAMPLE`-equivalent]
///   guidance.
/// - [`MantaError::MissingField`] if the resolved path is not valid
///   UTF-8 (required by the underlying `config` crate).
/// - [`MantaError::ConfigError`] on TOML parse, env-var type-coercion,
///   or builder failures.
///
/// [`CLI_CONFIG_SAMPLE`-equivalent]: self
/// [`MantaError::NotFound`]: crate::common::error::MantaError::NotFound
/// [`MantaError::MissingField`]: crate::common::error::MantaError::MissingField
/// [`MantaError::ConfigError`]: crate::common::error::MantaError::ConfigError
pub fn get_cli_configuration() -> Result<Config, Error> {
  load_config(
    "CLI",
    get_cli_config_file_path()?,
    CLI_CONFIG_SAMPLE,
    CLI_CONFIG_MIGRATION,
  )
}

/// Load `server.toml`. Fails loudly if the file is missing; the error
/// message includes a minimal example and (when a legacy config.toml is
/// detected) a field-by-field migration mapping.
///
/// Reads the file at [`get_server_config_file_path`] and layers
/// `MANTA_*`-prefixed environment variables on top (env wins).
///
/// # Errors
///
/// - [`MantaError::NotFound`] when the resolved config file does not
///   exist on disk; the message embeds a minimal `server.toml`
///   example and (if applicable) a migration mapping.
/// - [`MantaError::MissingField`] if the resolved path is not valid
///   UTF-8.
/// - [`MantaError::ConfigError`] on TOML parse, env-var type-coercion,
///   or builder failures.
///
/// [`MantaError::NotFound`]: crate::common::error::MantaError::NotFound
/// [`MantaError::MissingField`]: crate::common::error::MantaError::MissingField
/// [`MantaError::ConfigError`]: crate::common::error::MantaError::ConfigError
pub fn get_server_configuration() -> Result<Config, Error> {
  load_config(
    "Server",
    get_server_config_file_path()?,
    SERVER_CONFIG_SAMPLE,
    SERVER_CONFIG_MIGRATION,
  )
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::io::Write;
  use std::sync::Mutex;
  use tempfile::NamedTempFile;

  // The MANTA_* env vars are process-global; tests that mutate them
  // must serialise on this lock or they'll race each other under
  // cargo's default parallel test runner.
  static ENV_LOCK: Mutex<()> = Mutex::new(());

  /// Guard that sets the named env var on construction and clears it on
  /// drop. Use inside a test holding `ENV_LOCK` so concurrent tests
  /// don't see the half-installed value.
  struct EnvGuard(&'static str);
  impl EnvGuard {
    fn set(key: &'static str, value: &str) -> Self {
      // SAFETY: serialised by `ENV_LOCK` above.
      unsafe { std::env::set_var(key, value) };
      Self(key)
    }
  }
  impl Drop for EnvGuard {
    fn drop(&mut self) {
      unsafe { std::env::remove_var(self.0) };
    }
  }

  fn write_tmp_toml(content: &str) -> NamedTempFile {
    let mut f = NamedTempFile::new().expect("tempfile");
    f.write_all(content.as_bytes()).expect("write tempfile");
    f
  }

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

  #[test]
  fn cli_config_file_path_honors_env_var() {
    let _g = ENV_LOCK.lock().unwrap();
    let _e = EnvGuard::set("MANTA_CLI_CONFIG", "/tmp/custom-cli.toml");
    let path = get_cli_config_file_path().unwrap();
    assert_eq!(path, PathBuf::from("/tmp/custom-cli.toml"));
  }

  #[test]
  fn server_config_file_path_honors_env_var() {
    let _g = ENV_LOCK.lock().unwrap();
    let _e = EnvGuard::set("MANTA_SERVER_CONFIG", "/tmp/custom-server.toml");
    let path = get_server_config_file_path().unwrap();
    assert_eq!(path, PathBuf::from("/tmp/custom-server.toml"));
  }

  #[test]
  fn cli_configuration_with_missing_file_returns_notfound() {
    let _g = ENV_LOCK.lock().unwrap();
    let _e = EnvGuard::set(
      "MANTA_CLI_CONFIG",
      "/nonexistent-dir/definitely-not-here.toml",
    );
    let err = get_cli_configuration().unwrap_err();
    match err {
      Error::NotFound(msg) => {
        assert!(
          msg.contains("CLI configuration file"),
          "expected helpful NotFound message, got: {msg}"
        );
        assert!(
          msg.contains("Minimal example"),
          "expected sample TOML in message"
        );
      }
      other => panic!("expected NotFound, got {other:?}"),
    }
  }

  #[test]
  fn cli_configuration_with_malformed_toml_returns_config_error() {
    let _g = ENV_LOCK.lock().unwrap();
    let bad = write_tmp_toml("this is = not [valid toml");
    let _e = EnvGuard::set("MANTA_CLI_CONFIG", bad.path().to_str().unwrap());
    let err = get_cli_configuration().unwrap_err();
    assert!(
      matches!(err, Error::ConfigError(_)),
      "expected ConfigError variant, got {err:?}"
    );
  }

  #[test]
  fn cli_configuration_loads_valid_toml_and_env_var_overrides_file() {
    let _g = ENV_LOCK.lock().unwrap();
    let good = write_tmp_toml(
      r#"log = "info"
site = "alps"
manta_server_url = "https://example:8443"
"#,
    );
    let _path =
      EnvGuard::set("MANTA_CLI_CONFIG", good.path().to_str().unwrap());

    let cfg = get_cli_configuration().unwrap();
    assert_eq!(cfg.get_string("log").unwrap(), "info");
    assert_eq!(cfg.get_string("site").unwrap(), "alps");
    drop(cfg);

    // Set a MANTA_*-prefixed env var; the `Environment` source should
    // merge over the file value.
    let _override = EnvGuard::set("MANTA_LOG", "trace");
    let cfg = get_cli_configuration().unwrap();
    assert_eq!(
      cfg.get_string("log").unwrap(),
      "trace",
      "env var should override file value"
    );
  }

  #[test]
  fn server_configuration_with_missing_file_returns_notfound() {
    let _g = ENV_LOCK.lock().unwrap();
    let _e = EnvGuard::set(
      "MANTA_SERVER_CONFIG",
      "/nonexistent-dir/missing-server.toml",
    );
    let err = get_server_configuration().unwrap_err();
    match err {
      Error::NotFound(msg) => {
        assert!(
          msg.contains("Server configuration file"),
          "expected helpful NotFound message, got: {msg}"
        );
      }
      other => panic!("expected NotFound, got {other:?}"),
    }
  }

  #[test]
  fn server_configuration_loads_valid_toml() {
    let _g = ENV_LOCK.lock().unwrap();
    let good = write_tmp_toml(
      r#"log = "info"

[server]
listen_address = "0.0.0.0"
port = 8443
cert = "/etc/manta/cert.pem"
key = "/etc/manta/key.pem"
"#,
    );
    let _e =
      EnvGuard::set("MANTA_SERVER_CONFIG", good.path().to_str().unwrap());
    let cfg = get_server_configuration().unwrap();
    assert_eq!(cfg.get_string("server.listen_address").unwrap(), "0.0.0.0");
    assert_eq!(cfg.get_int("server.port").unwrap(), 8443);
  }
}
