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
  /// When `true`, the CLI refuses backend-mutating verbs (`add`,
  /// `apply`, `delete`, `migrate`, `power`, `run`, `restore`)
  /// before any HTTP request leaves the process. Toggled by
  /// `manta config set read-only` / `manta config unset read-only`.
  #[serde(default)]
  pub read_only: bool,
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
  /// Seconds between `GET /power/transitions/{id}` polls in
  /// `manta power on/off/reset`. `None` keeps the historical 3 s
  /// (see `crate::dispatch::power::DEFAULT_POWER_POLL_INTERVAL_SECS`).
  #[serde(default)]
  pub power_poll_interval_secs: Option<u64>,
  /// Maximum number of poll attempts before `manta power` gives up
  /// waiting for a transition to complete. `None` keeps the
  /// historical 300 (15 minutes at the default 3 s interval).
  #[serde(default)]
  pub power_max_poll_attempts: Option<u32>,
  /// Seconds between CFS-session status polls in
  /// `manta apply sat-file`'s monitor loop. `None` keeps the
  /// historical 10 s.
  #[serde(default)]
  pub sat_file_poll_interval_secs: Option<u64>,
  /// Hard cap (seconds) on the SAT-file monitor loop before it
  /// bails. `None` keeps the historical 4 h (14400 s).
  #[serde(default)]
  pub sat_file_poll_budget_secs: Option<u64>,
  /// Cap (seconds) on consecutive "session not yet visible"
  /// responses before SAT-file apply bails. `None` keeps the
  /// historical 5 min (300 s).
  #[serde(default)]
  pub sat_file_not_visible_budget_secs: Option<u64>,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn cli_configuration_roundtrip_toml_minimal() {
    let cfg = CliConfiguration {
      log: "info".to_string(),
      site: "alps".to_string(),
      read_only: false,
      manta_server_url: "https://manta-server.cscs.ch:8443".to_string(),
      socks5_proxy: Some("socks5h://127.0.0.1:1080".to_string()),
      request_timeout_secs: None,
      power_poll_interval_secs: None,
      power_max_poll_attempts: None,
      sat_file_poll_interval_secs: None,
      sat_file_poll_budget_secs: None,
      sat_file_not_visible_budget_secs: None,
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

  fn minimal_toml() -> String {
    r#"
log = "info"
site = "alps"
manta_server_url = "https://manta-server.example.com:8443"
"#
    .to_string()
  }

  #[test]
  fn read_only_defaults_to_false_when_absent() {
    let cfg: CliConfiguration = toml::from_str(&minimal_toml()).expect("parse");
    assert!(!cfg.read_only);
  }

  #[test]
  fn read_only_parses_true_when_present() {
    let mut s = minimal_toml();
    s.push_str("read_only = true\n");
    let cfg: CliConfiguration = toml::from_str(&s).expect("parse");
    assert!(cfg.read_only);
  }

  #[test]
  fn read_only_parses_false_when_explicitly_false() {
    let mut s = minimal_toml();
    s.push_str("read_only = false\n");
    let cfg: CliConfiguration = toml::from_str(&s).expect("parse");
    assert!(!cfg.read_only);
  }
}
