//! Typed schema for `cli.toml`.
//!
//! The untyped `config::Config` is loaded by
//! [`manta_shared::common::config::get_cli_configuration`]; this module
//! owns the typed deserialisation target.

use serde::{Deserialize, Serialize};

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
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn cli_configuration_roundtrip_toml_minimal() {
    let cfg = CliConfiguration {
      log: "info".to_string(),
      site: "alps".to_string(),
      parent_hsm_group: "nodes_free".to_string(),
      manta_server_url: "https://manta-server.cscs.ch:8443".to_string(),
      socks5_proxy: Some("socks5h://127.0.0.1:1080".to_string()),
      request_timeout_secs: None,
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
      site = "alps"
      parent_hsm_group = ""
      # missing manta_server_url
    "#;
    let result = toml::from_str::<CliConfiguration>(bad_toml);
    assert!(result.is_err());
  }
}
