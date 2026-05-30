//! Typed config-file schemas for `cli.toml` and `server.toml`.
//!
//! See [`CliConfiguration`] and [`ServerConfiguration`] for the
//! top-level shapes. The loaders that materialise these from disk
//! live in the parent module ([`super::get_cli_configuration`],
//! [`super::get_server_configuration`]).

use std::collections::HashMap;

use crate::common::audit::Auditor;

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

/// Top-level configuration for the `manta-cli` binary. Persisted as TOML
/// under `~/.config/manta/cli.toml`. Carries only the fields the CLI uses
/// — every backend connection detail (per-site URLs, TLS certs, vault,
/// k8s, per-site SOCKS proxies) lives in `ServerConfiguration`. The CLI
/// only knows about the *one* manta-server it talks to.
#[derive(Serialize, Deserialize, Debug)]
pub struct CliConfiguration {
  /// `EnvFilter` directive string for the tracing subscriber
  /// (e.g. `"info"`, `"manta=debug,hyper=warn"`).
  pub log: String,
  /// Path to the local file the CLI appends audit lines to.
  pub audit_file: String,
  /// Active site name, sent as the `X-Manta-Site` header on every
  /// request to manta-server. Overridable per-invocation with `--site`.
  /// The server validates that the name matches one of its configured
  /// sites; the CLI does no local validation.
  pub site: String,
  /// Default HSM group threaded into commands that accept
  /// `--hsm-group` when none is supplied on the command line.
  pub parent_hsm_group: String,
  /// URL of the manta HTTP server this CLI talks to. Required — the CLI
  /// no longer calls CSM/OCHAMI backends directly; every operation
  /// (including auth) is forwarded through `manta-server`.
  pub manta_server_url: String,
  /// Optional SOCKS5 proxy used to reach `manta_server_url`. Per-site
  /// proxying for backend traffic is the server's concern.
  pub socks5_proxy: Option<String>,
  /// Optional per-request HTTP timeout, in seconds. When set, the
  /// reqwest client used to reach `manta_server_url` is built with
  /// `.timeout(Duration::from_secs(n))`; when `None` (the default),
  /// reqwest applies no per-request timeout — long-running calls
  /// (e.g. `POST /power`) hang until the server responds or the
  /// underlying connection drops. Set this to match the server's
  /// longest legitimate response time when running through a SOCKS5
  /// tunnel or proxy that silently drops idle connections.
  #[serde(default)]
  pub request_timeout_secs: Option<u64>,
  /// Optional Kafka audit forwarder. When `None`, the CLI emits no
  /// audit messages.
  pub auditor: Option<Auditor>,
}

/// Server-only settings — TLS, listen address, console behaviour. Lives
/// under `[server]` in `server.toml`.
#[derive(Serialize, Deserialize, Debug)]
pub struct ServerSettings {
  /// TCP listen address (e.g. "0.0.0.0").
  pub listen_address: String,
  /// TCP port for the TLS server.
  pub port: u16,
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
  /// When this elapses the server returns `408 REQUEST_TIMEOUT`. Long-
  /// running endpoints (e.g. `/power`) override this with their own
  /// per-route timeout.
  #[serde(default = "default_request_timeout_secs")]
  pub request_timeout_secs: u64,
  /// Per-route timeout for `POST /power`, in seconds. Power operations
  /// against large clusters (especially `reset`) often exceed the
  /// global default; this knob keeps the safety net for other
  /// endpoints intact while giving power the headroom it needs.
  #[serde(default = "default_power_timeout_secs")]
  pub power_timeout_secs: u64,
}

/// Default global request timeout — 60s. Matches the historical
/// hardcoded value.
fn default_request_timeout_secs() -> u64 {
  60
}

/// Default per-route power timeout — 600s (10 minutes). Empirically
/// sufficient for cluster-wide `reset` against a few hundred xnames
/// without blowing past it under normal CSM load.
fn default_power_timeout_secs() -> u64 {
  600
}

/// Top-level configuration for the `manta-server` binary. Persisted as
/// TOML under `~/.config/manta/server.toml`. Has no notion of an "active"
/// site — the server hosts every configured site simultaneously and
/// clients select per-request via the `X-Manta-Site` header.
#[derive(Serialize, Deserialize, Debug)]
pub struct ServerConfiguration {
  /// `EnvFilter` directive for the tracing subscriber.
  pub log: String,
  /// Path to the local file the server appends audit lines to.
  pub audit_file: String,
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
  fn cli_configuration_roundtrip_toml_minimal() {
    let cfg = CliConfiguration {
      log: "info".to_string(),
      audit_file: "/tmp/cli-audit.log".to_string(),
      site: "alps".to_string(),
      parent_hsm_group: "nodes_free".to_string(),
      manta_server_url: "https://manta-server.cscs.ch:8443".to_string(),
      socks5_proxy: Some("socks5h://127.0.0.1:1080".to_string()),
      request_timeout_secs: None,
      auditor: None,
    };
    let toml_str = toml::to_string(&cfg).unwrap();
    let parsed: CliConfiguration = toml::from_str(&toml_str).unwrap();
    assert_eq!(parsed.site, "alps");
    assert_eq!(parsed.parent_hsm_group, "nodes_free");
    assert_eq!(parsed.manta_server_url, "https://manta-server.cscs.ch:8443");
    assert_eq!(
      parsed.socks5_proxy.as_deref(),
      Some("socks5h://127.0.0.1:1080")
    );
  }

  #[test]
  fn cli_configuration_socks5_proxy_optional() {
    let toml_str = r#"
      log = "info"
      audit_file = "/tmp/cli-audit.log"
      site = "alps"
      parent_hsm_group = ""
      manta_server_url = "https://manta-server.cscs.ch:8443"
    "#;
    let parsed: CliConfiguration = toml::from_str(toml_str).unwrap();
    assert!(parsed.socks5_proxy.is_none());
  }

  #[test]
  fn cli_configuration_missing_manta_server_url_fails() {
    let bad_toml = r#"
      log = "info"
      audit_file = "/tmp/cli-audit.log"
      site = "alps"
      parent_hsm_group = ""
      # missing manta_server_url
    "#;
    let result = toml::from_str::<CliConfiguration>(bad_toml);
    assert!(result.is_err());
  }

  #[test]
  fn server_configuration_roundtrip_toml_minimal() {
    let mut sites = HashMap::new();
    sites.insert("alps".to_string(), make_minimal_site());
    let cfg = ServerConfiguration {
      log: "info".to_string(),
      audit_file: "/var/log/manta/server-audit.log".to_string(),
      server: ServerSettings {
        listen_address: "0.0.0.0".to_string(),
        port: 8443,
        cert: Some("/etc/manta/tls/server.crt".to_string()),
        key: Some("/etc/manta/tls/server.key".to_string()),
        console_inactivity_timeout_secs: 1800,
        auth_rate_limit_per_minute: Some(60),
        request_timeout_secs: 60,
        power_timeout_secs: 600,
      },
      sites,
      auditor: None,
    };
    let toml_str = toml::to_string(&cfg).unwrap();
    let parsed: ServerConfiguration = toml::from_str(&toml_str).unwrap();
    assert_eq!(parsed.server.port, 8443);
    assert_eq!(parsed.server.listen_address, "0.0.0.0");
    assert_eq!(parsed.server.console_inactivity_timeout_secs, 1800);
    assert_eq!(parsed.server.request_timeout_secs, 60);
    assert_eq!(parsed.server.power_timeout_secs, 600);
    assert_eq!(
      parsed.server.cert.as_deref(),
      Some("/etc/manta/tls/server.crt")
    );
  }

  /// Existing server.toml files that pre-date the timeout fields must
  /// keep working — both fields fall back to their defaults.
  #[test]
  fn server_settings_timeout_fields_default_when_omitted() {
    let toml_str = r#"
      listen_address = "0.0.0.0"
      port = 8443
      console_inactivity_timeout_secs = 1800
    "#;
    let parsed: ServerSettings = toml::from_str(toml_str).unwrap();
    assert_eq!(parsed.request_timeout_secs, 60);
    assert_eq!(parsed.power_timeout_secs, 600);
  }

  #[test]
  fn server_configuration_deserialize_missing_server_section_fails() {
    let bad_toml = r#"
      log = "info"
      audit_file = "/tmp/server.log"
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
