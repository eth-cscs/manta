//! Typed schema for `server.toml`.
//!
//! The untyped `config::Config` is loaded by
//! [`manta_shared::common::config::get_server_configuration`]; this
//! module owns the typed deserialisation target.

use std::collections::HashMap;

use crate::server::common::audit::Auditor;
use manta_backend_dispatcher::types::K8sDetails;
use serde::{Deserialize, Serialize};

/// Which backend API this site speaks.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BackendTechnology {
  /// HPE Cray System Management (CSM) backend.
  Csm,
  /// OpenCHAMI backend.
  Ochami,
}

impl BackendTechnology {
  /// Return the lowercase string expected by `StaticBackendDispatcher::new`.
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::Csm => "csm",
      Self::Ochami => "ochami",
    }
  }
}

#[derive(Serialize, Deserialize, Debug)]
/// Connection details for a single ALPS site (CSM or OCHAMI instance).
///
/// The Vault URL used by handlers requiring vault (sat-file, session,
/// console, logs) is derived at startup from
/// `[sites.X.k8s.authentication.vault] base_url`. The vault secret path
/// is derived from a hard-coded prefix and the site name. Neither is
/// configured here.
pub struct Site {
  /// Which backend implementation this site uses (`csm` or `ochami`).
  pub backend: BackendTechnology,
  /// Optional per-site SOCKS5 proxy URL used by every outbound HTTP
  /// request to this site's backend. `None` means direct connection.
  pub socks5_proxy: Option<String>,
  /// Base URL of the backend API (e.g. `https://api.alps.cscs.ch`).
  pub shasta_base_url: String,
  /// Optional Kubernetes connection details, required by handlers
  /// that stream CFS session logs or attach to consoles.
  pub k8s: Option<K8sDetails>,
  /// Path (absolute or relative to the config dir) of the backend's
  /// root CA certificate, used to verify TLS to `shasta_base_url`.
  pub root_ca_cert_file: String,
}

/// Server-only settings — TLS, listen address, console behaviour. Lives
/// under `[server]` in `server.toml`.
#[derive(Serialize, Deserialize, Debug)]
pub struct ServerSettings {
  /// TCP listen address (e.g. "0.0.0.0"). When omitted from config
  /// **and** no `--listen-address` flag is supplied, the server falls
  /// back to `"0.0.0.0"`.
  #[serde(default)]
  pub listen_address: Option<String>,
  /// TCP port. When omitted from config **and** no `--port` flag is
  /// supplied, the effective default depends on whether TLS is
  /// configured: `8443` if both `cert` and `key` are present (HTTPS),
  /// otherwise `8080` (plain HTTP). See
  /// [`ServerSettings::default_port`].
  #[serde(default)]
  pub port: Option<u16>,
  /// Path to the TLS certificate (PEM).
  pub cert: Option<String>,
  /// Path to the TLS private key (PEM).
  pub key: Option<String>,
  /// How long a node-console WebSocket stays open without activity
  /// before the server tears it down.
  pub console_inactivity_timeout_secs: u64,
  /// Per-source-IP rate limit for the `/api/v1/auth/*` endpoints,
  /// in requests per minute. `None` disables in-process rate limiting
  /// (operators are then expected to enforce it at the reverse proxy).
  pub auth_rate_limit_per_minute: Option<u32>,
  /// Global request timeout applied to every HTTP route, in seconds.
  /// When this elapses the server returns `408 REQUEST_TIMEOUT`. All
  /// long-running work (e.g. power transitions) now runs CLI-side,
  /// so no endpoint needs more than the default.
  #[serde(default = "default_request_timeout_secs")]
  pub request_timeout_secs: u64,
  /// Filesystem root that confines `POST /migrate/{backup,restore}`
  /// file access. When set, every `destination` / `bos_file` /
  /// `cfs_file` / `hsm_file` / `ims_file` / `image_dir` path in the
  /// request is canonicalised and rejected unless it resolves under
  /// this directory. When unset (default), the migrate endpoints
  /// return `BadRequest` even for admin callers — the operator must
  /// explicitly opt in to server-side filesystem writes.
  #[serde(default)]
  pub migrate_backup_root: Option<String>,
  /// Opt in to plain-HTTP listen mode. Default `false`: when neither
  /// `cert` nor `key` is configured the server refuses to start, so
  /// bearer tokens can't accidentally land on the wire in cleartext.
  /// Set to `true` only when TLS terminates upstream (reverse proxy
  /// or sidecar); otherwise leave it off and configure both `cert`
  /// and `key`.
  #[serde(default)]
  pub allow_http: bool,
}

impl ServerSettings {
  /// Effective default listen address when neither config nor CLI flag
  /// supplies one: bind on all interfaces.
  pub const DEFAULT_LISTEN_ADDRESS: &'static str = "0.0.0.0";

  /// Effective default port when neither config nor CLI flag supplies
  /// one. `8443` for the HTTPS path (cert + key both present), `8080`
  /// for plain HTTP — the latter is the typical dev / sidecar setup
  /// where TLS is terminated upstream.
  pub fn default_port(has_tls: bool) -> u16 {
    if has_tls { 8443 } else { 8080 }
  }
}

/// Default global request timeout — 60s. Matches the historical
/// hardcoded value.
fn default_request_timeout_secs() -> u64 {
  60
}

/// Top-level configuration for the `manta-server` binary. Persisted as
/// TOML under `~/.config/manta/server.toml`. Has no notion of an "active"
/// site — the server hosts every configured site simultaneously and
/// clients select per-request via the `X-Manta-Site` header.
#[derive(Serialize, Deserialize, Debug)]
pub struct ServerConfiguration {
  /// `EnvFilter` directive for the tracing subscriber.
  pub log: String,
  /// Network / TLS / console / rate-limit knobs for the HTTPS server.
  pub server: ServerSettings,
  /// Per-site backend connection details, keyed by site name. The
  /// `X-Manta-Site` header on each request picks which one to route to.
  pub sites: HashMap<String, Site>,
  /// Optional Kafka audit forwarder (typically used for `/auth/*`
  /// attempts). When `None`, the server emits no audit messages.
  pub auditor: Option<Auditor>,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn site_deserialize_missing_backend_fails() {
    let bad_toml = r#"
      shasta_base_url = "https://api.example.com"
      root_ca_cert_file = "cert.pem"
      # missing backend
    "#;
    let result = toml::from_str::<Site>(bad_toml);
    assert!(result.is_err());
  }

  #[test]
  fn backend_technology_as_str() {
    assert_eq!(BackendTechnology::Csm.as_str(), "csm");
    assert_eq!(BackendTechnology::Ochami.as_str(), "ochami");
  }

  #[test]
  fn backend_technology_roundtrip_toml() {
    // Verify TOML serializes as lowercase "csm" / "ochami"
    #[derive(Serialize, Deserialize)]
    struct Wrapper {
      backend: BackendTechnology,
    }
    let w = Wrapper {
      backend: BackendTechnology::Csm,
    };
    let s = toml::to_string(&w).unwrap();
    assert!(s.contains("\"csm\"") || s.contains("csm"));
    let parsed: Wrapper = toml::from_str(&s).unwrap();
    assert_eq!(parsed.backend, BackendTechnology::Csm);
  }

  fn make_minimal_site() -> Site {
    Site {
      backend: BackendTechnology::Csm,
      socks5_proxy: None,
      shasta_base_url: "https://api.example.com".to_string(),
      k8s: None,
      root_ca_cert_file: "cert.pem".to_string(),
    }
  }

  #[test]
  fn server_configuration_roundtrip_toml_minimal() {
    let mut sites = HashMap::new();
    sites.insert("alps".to_string(), make_minimal_site());
    let cfg = ServerConfiguration {
      log: "info".to_string(),
      server: ServerSettings {
        listen_address: Some("0.0.0.0".to_string()),
        port: Some(8443),
        cert: Some("/etc/manta/tls/server.crt".to_string()),
        key: Some("/etc/manta/tls/server.key".to_string()),
        console_inactivity_timeout_secs: 1800,
        auth_rate_limit_per_minute: Some(60),
        request_timeout_secs: 60,
        migrate_backup_root: None,
        allow_http: false,
      },
      sites,
      auditor: None,
    };
    let toml_str = toml::to_string(&cfg).unwrap();
    let parsed: ServerConfiguration = toml::from_str(&toml_str).unwrap();
    assert_eq!(parsed.server.port, Some(8443));
    assert_eq!(parsed.server.listen_address.as_deref(), Some("0.0.0.0"));
    assert_eq!(parsed.server.console_inactivity_timeout_secs, 1800);
    assert_eq!(parsed.server.request_timeout_secs, 60);
    assert_eq!(
      parsed.server.cert.as_deref(),
      Some("/etc/manta/tls/server.crt")
    );
  }

  /// Default port helper: 8443 when TLS is configured, 8080
  /// otherwise. Used by `manta-server::main` when no `port` is set
  /// in config or on the CLI.
  #[test]
  fn server_settings_default_port_depends_on_tls() {
    assert_eq!(ServerSettings::default_port(true), 8443);
    assert_eq!(ServerSettings::default_port(false), 8080);
  }

  /// power_timeout_secs is gone — confirm the surrounding
  /// timeout-related fields still default correctly when the only
  /// remaining knob is absent.
  #[test]
  fn server_settings_request_timeout_secs_defaults_to_60() {
    let toml_str = r#"
      listen_address = "0.0.0.0"
      port = 8443
      console_inactivity_timeout_secs = 1800
    "#;
    let parsed: ServerSettings = toml::from_str(toml_str).unwrap();
    assert_eq!(parsed.request_timeout_secs, 60);
  }

  /// `[server]` block with neither `listen_address` nor `port`
  /// supplied — both fields deserialise as `None`, leaving the
  /// effective values to be filled in at startup time. Confirms the
  /// schema-level back-compat for the new defaults.
  #[test]
  fn server_settings_listen_address_and_port_default_to_none() {
    let toml_str = r#"
      console_inactivity_timeout_secs = 1800
    "#;
    let parsed: ServerSettings = toml::from_str(toml_str).unwrap();
    assert!(parsed.listen_address.is_none());
    assert!(parsed.port.is_none());
  }

  /// Existing server.toml files that pre-date the request_timeout
  /// field must keep working — the field falls back to its default.
  #[test]
  fn server_settings_request_timeout_field_defaults_when_omitted() {
    let toml_str = r#"
      listen_address = "0.0.0.0"
      port = 8443
      console_inactivity_timeout_secs = 1800
    "#;
    let parsed: ServerSettings = toml::from_str(toml_str).unwrap();
    assert_eq!(parsed.request_timeout_secs, 60);
  }

  #[test]
  fn server_configuration_deserialize_missing_server_section_fails() {
    let bad_toml = r#"
      log = "info"
      [sites]
    "#;
    let result = toml::from_str::<ServerConfiguration>(bad_toml);
    assert!(result.is_err());
  }

  #[test]
  fn server_settings_optional_tls_paths() {
    // TLS cert/key are optional in the schema — flags can supply them
    // at runtime when the config omits them.
    let toml_str = r#"
      listen_address = "0.0.0.0"
      port = 8443
      console_inactivity_timeout_secs = 1800
    "#;
    let parsed: ServerSettings = toml::from_str(toml_str).unwrap();
    assert!(parsed.cert.is_none());
    assert!(parsed.key.is_none());
  }
}
