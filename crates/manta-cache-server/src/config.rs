//! `cache-server.toml` schema, path resolution, and validation.
//!
//! The file lives in the same platform config directory as the other
//! manta configs (`~/.config/manta/` on Linux), overridable with the
//! `MANTA_CACHE_SERVER_CONFIG` environment variable. The path helper is
//! a deliberate, tiny duplicate of `manta_shared::common::config`'s
//! `ProjectDirs` lookup: depending on `manta-shared` would drag
//! `manta-backend-dispatcher` (+ csm/ochami types) into this otherwise
//! standalone service for one function.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;

/// Sample config embedded in the "config file not found" error so an
/// operator can bootstrap without reading docs first.
const CONFIG_SAMPLE: &str = r#"log = "info"

[server]
# listen_address = "0.0.0.0"
# port = 8444                    # default: 8444 with TLS, 8081 without
# cert = "/path/to/cert.pem"     # omit both + allow_http for plain HTTP
# key  = "/path/to/key.pem"
# allow_http = false
# api_token = "shared-secret"    # require `Authorization: Bearer …` on /api/v1/*
# refresh_interval_secs = 300    # omit = refresh at startup only
# shutdown_grace_period_secs = 30

[sites.alps]
manta_server_url = "https://manta-server.example.com:8443"
# Exactly one of `token` / `token_file`. `token_file` is re-read on
# every refresh, so rotating the secret needs no restart.
token_file = "/run/secrets/manta-cache/alps-token"
"#;

/// Top-level `cache-server.toml` schema.
#[derive(Debug, Deserialize)]
pub struct CacheServerConfiguration {
  /// `tracing` filter string (e.g. `info`, `manta_cache=debug,info`).
  #[serde(default = "default_log")]
  pub log: String,
  /// Listener + refresh settings.
  #[serde(default)]
  pub server: ServerSettings,
  /// One entry per site the cache indexes, keyed by the site name sent
  /// as `X-Manta-Site`.
  #[serde(default)]
  pub sites: HashMap<String, SiteConfig>,
}

/// `[server]` block.
#[derive(Debug, Deserialize)]
pub struct ServerSettings {
  /// Bind address; defaults to [`ServerSettings::DEFAULT_LISTEN_ADDRESS`].
  pub listen_address: Option<String>,
  /// TCP port; defaults per [`ServerSettings::default_port`].
  pub port: Option<u16>,
  /// PEM certificate path. Set together with `key` for TLS.
  pub cert: Option<String>,
  /// PEM private-key path.
  pub key: Option<String>,
  /// Allow plain HTTP when no cert/key is set (TLS terminates
  /// upstream). Defaults to fail-closed, like manta-server.
  #[serde(default)]
  pub allow_http: bool,
  /// Optional shared bearer token required on every `/api/v1/*` call
  /// (`/health` stays open for probes). Absent = unauthenticated
  /// lookups; rely on network controls instead.
  pub api_token: Option<String>,
  /// Re-run the cross-site refresh every N seconds. Absent = refresh
  /// once at startup and serve that index until restart (Stage 4 adds
  /// on-demand refresh endpoints).
  pub refresh_interval_secs: Option<u64>,
  /// Drain window for graceful shutdown on SIGTERM / Ctrl+C.
  #[serde(default = "default_shutdown_grace_period_secs")]
  pub shutdown_grace_period_secs: u64,
}

// Manual impl so an absent `[server]` table gets the same defaults as
// a present-but-sparse one — a derived Default would zero
// `shutdown_grace_period_secs` past the serde field default.
impl Default for ServerSettings {
  fn default() -> Self {
    Self {
      listen_address: None,
      port: None,
      cert: None,
      key: None,
      allow_http: false,
      api_token: None,
      refresh_interval_secs: None,
      shutdown_grace_period_secs: default_shutdown_grace_period_secs(),
    }
  }
}

impl ServerSettings {
  /// Default bind address: all interfaces (the service exists to be
  /// shared).
  pub const DEFAULT_LISTEN_ADDRESS: &'static str = "0.0.0.0";

  /// Default port when neither config nor CLI flag supplies one.
  /// `8444` for HTTPS / `8081` for plain HTTP — one above
  /// manta-server's `8443`/`8080` so a colocated pair never collides.
  pub fn default_port(has_tls: bool) -> u16 {
    if has_tls { 8444 } else { 8081 }
  }
}

/// One `[sites.<name>]` block: where that site's manta-server lives and
/// the service-account credential used to refresh from it.
#[derive(Debug, Deserialize)]
pub struct SiteConfig {
  /// Base URL of the manta-server hosting the site (scheme + host +
  /// port; `/api/v1` is appended by the cache).
  pub manta_server_url: String,
  /// Inline service-account bearer token. Mutually exclusive with
  /// `token_file`.
  pub token: Option<String>,
  /// Path to a file holding the token. Re-read on every refresh, so a
  /// secret manager can rotate it without a restart. Mutually
  /// exclusive with `token`.
  pub token_file: Option<PathBuf>,
}

fn default_log() -> String {
  "info".to_string()
}

fn default_shutdown_grace_period_secs() -> u64 {
  30
}

/// Resolve the config file path: `MANTA_CACHE_SERVER_CONFIG` env var,
/// else `<platform config dir>/manta/cache-server.toml` (the same
/// directory as `server.toml` / `cli.toml`).
pub fn config_file_path() -> Result<PathBuf, String> {
  if let Ok(env_path) = std::env::var("MANTA_CACHE_SERVER_CONFIG") {
    return Ok(PathBuf::from(env_path));
  }
  // Same qualifier/organization/application triple as manta-shared's
  // `get_project_dirs`, so all manta configs share one directory.
  let dirs = directories::ProjectDirs::from("local", "cscs", "manta")
    .ok_or_else(|| {
      "Could not determine the config directory (home directory may \
       not be set); set MANTA_CACHE_SERVER_CONFIG instead."
        .to_string()
    })?;
  Ok(dirs.config_dir().join("cache-server.toml"))
}

/// Read and parse the config file, returning it with the path it was
/// loaded from (for the startup summary).
pub fn load() -> Result<(CacheServerConfiguration, PathBuf), String> {
  let path = config_file_path()?;
  let raw = std::fs::read_to_string(&path).map_err(|e| {
    format!(
      "Could not read config file '{}': {e}\n\n\
       Create it with content like:\n\n{CONFIG_SAMPLE}",
      path.display()
    )
  })?;
  let config: CacheServerConfiguration = toml::from_str(&raw).map_err(|e| {
    format!("Config file '{}' is not valid: {e}", path.display())
  })?;
  config.validate()?;
  Ok((config, path))
}

impl CacheServerConfiguration {
  /// Structural checks that TOML typing cannot express.
  pub fn validate(&self) -> Result<(), String> {
    if self.sites.is_empty() {
      return Err(
        "No [sites.<name>] entries configured — the cache would have \
         nothing to index. Add at least one site."
          .to_string(),
      );
    }
    for (name, site) in &self.sites {
      match (&site.token, &site.token_file) {
        (Some(_), Some(_)) => {
          return Err(format!(
            "[sites.{name}] sets both `token` and `token_file`; \
             configure exactly one."
          ));
        }
        (None, None) => {
          return Err(format!(
            "[sites.{name}] sets neither `token` nor `token_file`; \
             the refresh needs a service-account credential."
          ));
        }
        _ => {}
      }
      if site.manta_server_url.trim().is_empty() {
        return Err(format!("[sites.{name}] `manta_server_url` is empty."));
      }
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn parse(raw: &str) -> CacheServerConfiguration {
    toml::from_str(raw).expect("toml parses")
  }

  #[test]
  fn minimal_config_parses_and_validates() {
    let config = parse(
      r#"
      [sites.alps]
      manta_server_url = "https://manta:8443"
      token = "ey"
      "#,
    );
    config.validate().expect("valid");
    assert_eq!(config.log, "info");
    assert_eq!(config.server.shutdown_grace_period_secs, 30);
    assert!(config.server.refresh_interval_secs.is_none());
  }

  #[test]
  fn both_token_forms_is_rejected() {
    let config = parse(
      r#"
      [sites.alps]
      manta_server_url = "https://manta:8443"
      token = "ey"
      token_file = "/run/secret"
      "#,
    );
    let err = config.validate().unwrap_err();
    assert!(err.contains("exactly one"), "{err}");
  }

  #[test]
  fn missing_credential_is_rejected() {
    let config = parse(
      r#"
      [sites.alps]
      manta_server_url = "https://manta:8443"
      "#,
    );
    let err = config.validate().unwrap_err();
    assert!(err.contains("neither"), "{err}");
  }

  #[test]
  fn no_sites_is_rejected() {
    let config = parse("log = \"info\"");
    let err = config.validate().unwrap_err();
    assert!(err.contains("No [sites"), "{err}");
  }

  #[test]
  fn default_ports_sit_next_to_manta_server() {
    assert_eq!(ServerSettings::default_port(true), 8444);
    assert_eq!(ServerSettings::default_port(false), 8081);
  }
}
