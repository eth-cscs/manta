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
  /// URL of the manta HTTP server this CLI talks to. Required — the CLI
  /// no longer calls CSM/OCHAMI backends directly; every operation
  /// (including auth) is forwarded through `manta-server`.
  pub manta_server_url: String,
  /// Optional SOCKS5 proxy used to reach `manta_server_url`. Per-site
  /// proxying for backend traffic is the server's concern.
  pub socks5_proxy: Option<String>,
  /// Optional per-request HTTP timeout, in seconds, for calls reaching
  /// `manta_server_url`. Two clients live behind this knob:
  ///
  /// - The one-shot REST client (every `manta get`, `manta apply`,
  ///   `manta delete`, …): when this field is `None`, defaults to 300 s
  ///   (5 min) so a stuck call eventually fails rather than hanging
  ///   forever. When set, the supplied value wins.
  /// - The streaming client (SSE log tail, WebSocket console): when
  ///   this field is `None`, applies no timeout — a CFS log stream or
  ///   interactive console can stay open indefinitely. When set, the
  ///   supplied value applies and will truncate long streams; pick a
  ///   value larger than your worst-case session if you set it.
  ///
  /// Override this when running through a SOCKS5 tunnel or proxy that
  /// silently drops idle connections, or when a specific cluster takes
  /// longer than the 5-minute default to respond.
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
      manta_server_url: "https://manta-server.cscs.ch:8443".to_string(),
      socks5_proxy: Some("socks5h://127.0.0.1:1080".to_string()),
      request_timeout_secs: None,
    };
    let toml_str = toml::to_string(&cfg).unwrap();
    let parsed: CliConfiguration = toml::from_str(&toml_str).unwrap();
    assert_eq!(parsed.site, "alps");
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
      # missing manta_server_url
    "#;
    let result = toml::from_str::<CliConfiguration>(bad_toml);
    assert!(result.is_err());
  }
}
