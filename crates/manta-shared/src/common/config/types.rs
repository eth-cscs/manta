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
pub struct Site {
  pub backend: BackendTechnology,
  pub socks5_proxy: Option<String>,
  pub shasta_base_url: String,
  pub k8s: Option<K8sDetails>,
  pub vault_base_url: Option<String>,
  pub vault_secret_path: Option<String>,
  pub root_ca_cert_file: String,
}

/// Top-level configuration for the `manta-cli` binary. Persisted as TOML
/// under `~/.config/manta/cli.toml`. Carries only the fields the CLI uses
/// — server-only knobs (TLS, listen address) live in `ServerConfiguration`.
#[derive(Serialize, Deserialize, Debug)]
pub struct CliConfiguration {
  pub log: String,
  pub audit_file: String,
  /// Active site for this CLI process. Overridable per-invocation with
  /// `--site <name>`.
  pub site: String,
  pub parent_hsm_group: String,
  /// URL of the manta HTTP server this CLI talks to. Required — the CLI
  /// no longer calls CSM/OCHAMI backends directly; every operation
  /// (including auth) is forwarded through `manta-server`.
  pub manta_server_url: String,
  pub sites: HashMap<String, Site>,
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
}

/// Top-level configuration for the `manta-server` binary. Persisted as
/// TOML under `~/.config/manta/server.toml`. Has no notion of an "active"
/// site — the server hosts every configured site simultaneously and
/// clients select per-request via the `X-Manta-Site` header.
#[derive(Serialize, Deserialize, Debug)]
pub struct ServerConfiguration {
  pub log: String,
  pub audit_file: String,
  pub server: ServerSettings,
  pub sites: HashMap<String, Site>,
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
      vault_base_url: None,
      vault_secret_path: None,
      root_ca_cert_file: "cert.pem".to_string(),
    }
  }

  #[test]
  fn cli_configuration_roundtrip_toml_minimal() {
    let mut sites = HashMap::new();
    sites.insert("alps".to_string(), make_minimal_site());
    let cfg = CliConfiguration {
      log: "info".to_string(),
      audit_file: "/tmp/cli-audit.log".to_string(),
      site: "alps".to_string(),
      parent_hsm_group: "nodes_free".to_string(),
      manta_server_url: "https://manta-server.cscs.ch:8443".to_string(),
      sites,
      auditor: None,
    };
    let toml_str = toml::to_string(&cfg).unwrap();
    let parsed: CliConfiguration = toml::from_str(&toml_str).unwrap();
    assert_eq!(parsed.site, "alps");
    assert_eq!(parsed.parent_hsm_group, "nodes_free");
    assert_eq!(parsed.manta_server_url, "https://manta-server.cscs.ch:8443");
    assert!(parsed.sites.contains_key("alps"));
  }

  #[test]
  fn cli_configuration_missing_manta_server_url_fails() {
    let bad_toml = r#"
      log = "info"
      audit_file = "/tmp/cli-audit.log"
      site = "alps"
      parent_hsm_group = ""

      [sites.alps]
      backend = "csm"
      shasta_base_url = "https://api.example.com"
      root_ca_cert_file = "cert.pem"
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
      },
      sites,
      auditor: None,
    };
    let toml_str = toml::to_string(&cfg).unwrap();
    let parsed: ServerConfiguration = toml::from_str(&toml_str).unwrap();
    assert_eq!(parsed.server.port, 8443);
    assert_eq!(parsed.server.listen_address, "0.0.0.0");
    assert_eq!(parsed.server.console_inactivity_timeout_secs, 1800);
    assert_eq!(
      parsed.server.cert.as_deref(),
      Some("/etc/manta/tls/server.crt")
    );
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
