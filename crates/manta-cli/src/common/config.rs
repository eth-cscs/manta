//! Typed schema for `cli.toml`.
//!
//! [`manta_shared::common::config::get_cli_configuration`] reads
//! `~/.config/manta/cli.toml` into a `config::Config` (the untyped
//! layered loader from the `config` crate) and then `try_deserialize`s
//! it into the [`CliConfiguration`] declared here. Every CLI
//! subcommand pulls its operational knobs (timeouts, poll intervals,
//! read-only gate, server URL) out of this struct via
//! [`crate::common::app_context::AppContext`].
//!
//! Field-level docs below double as the operator-facing reference for
//! each TOML key. The TOML key name matches the Rust field name —
//! `serde` rename attributes are not used.

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
  pub site: Option<String>,
  /// URL of the manta HTTP server this CLI talks to. Required — the CLI
  /// no longer calls CSM/OCHAMI backends directly; every operation
  /// (including auth) is forwarded through `manta-server`.
  pub manta_server_url: String,
  /// Optional SOCKS5 proxy used to reach `manta_server_url`. Per-site
  /// proxying for backend traffic is the server's concern.
  pub socks5_proxy: Option<String>,
  /// Optional base URL of a `manta-cache-server`. When set and no site
  /// was named (`--site` / `site`), commands that target an HSM group
  /// or a plain xname list resolve their site through the cache before
  /// dispatch (see [`crate::common::site_resolution`]). Unset = the
  /// site must always be named explicitly, as before.
  #[serde(default)]
  pub cache_url: Option<String>,
  /// Bearer token for the cache's `/api/v1` endpoints — the value of
  /// the cache's `[server] api_token`. Only needed when the cache is
  /// configured to require one.
  #[serde(default)]
  pub cache_api_token: Option<String>,
  /// When `true`, the CLI refuses every backend-mutating verb before
  /// any HTTP request leaves the process (see
  /// [`crate::common::read_only::read_only_gate`] and
  /// [`crate::common::read_only::MUTATING_VERBS`]). Toggled with
  /// `manta config set read-only` / `manta config unset read-only`.
  #[serde(default)]
  pub read_only: bool,
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
      site: Some("alps".to_string()),
      manta_server_url: "https://manta-server.cscs.ch:8443".to_string(),
      socks5_proxy: Some("socks5h://127.0.0.1:1080".to_string()),
      cache_url: None,
      cache_api_token: None,
      read_only: false,
      request_timeout_secs: None,
      power_poll_interval_secs: None,
      power_max_poll_attempts: None,
      sat_file_poll_interval_secs: None,
      sat_file_poll_budget_secs: None,
      sat_file_not_visible_budget_secs: None,
    };
    let toml_str = toml::to_string(&cfg).unwrap();
    let parsed: CliConfiguration = toml::from_str(&toml_str).unwrap();
    assert_eq!(parsed.site.as_deref(), Some("alps"));
    assert_eq!(parsed.manta_server_url, "https://manta-server.cscs.ch:8443");
    assert_eq!(
      parsed.socks5_proxy.as_deref(),
      Some("socks5h://127.0.0.1:1080")
    );
  }

  #[test]
  fn cache_keys_parse_and_default_to_none() {
    let parsed: CliConfiguration = toml::from_str(
      "log = \"info\"\nmanta_server_url = \"https://m:8443\"\n\
       cache_url = \"https://cache.example.ch:8444\"\n\
       cache_api_token = \"sekrit\"\n",
    )
    .unwrap();
    assert_eq!(
      parsed.cache_url.as_deref(),
      Some("https://cache.example.ch:8444")
    );
    assert_eq!(parsed.cache_api_token.as_deref(), Some("sekrit"));

    let minimal: CliConfiguration =
      toml::from_str("log = \"info\"\nmanta_server_url = \"https://m:8443\"\n")
        .unwrap();
    assert!(minimal.cache_url.is_none());
    assert!(minimal.cache_api_token.is_none());
  }

  #[test]
  fn cli_configuration_site_optional() {
    let toml_str = r#"
      log = "info"
      manta_server_url = "https://manta-server.cscs.ch:8443"
    "#;
    let parsed: CliConfiguration = toml::from_str(toml_str).unwrap();
    assert!(parsed.site.is_none());
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

  #[test]
  fn read_only_defaults_to_false_when_absent() {
    let toml_str = r#"
      log = "info"
      site = "alps"
      manta_server_url = "https://manta-server.cscs.ch:8443"
    "#;
    let cfg: CliConfiguration = toml::from_str(toml_str).unwrap();
    assert!(!cfg.read_only);
  }

  #[test]
  fn read_only_parses_true_when_present() {
    let toml_str = r#"
      log = "info"
      site = "alps"
      manta_server_url = "https://manta-server.cscs.ch:8443"
      read_only = true
    "#;
    let cfg: CliConfiguration = toml::from_str(toml_str).unwrap();
    assert!(cfg.read_only);
  }

  #[test]
  fn read_only_parses_false_when_explicitly_false() {
    let toml_str = r#"
      log = "info"
      site = "alps"
      manta_server_url = "https://manta-server.cscs.ch:8443"
      read_only = false
    "#;
    let cfg: CliConfiguration = toml::from_str(toml_str).unwrap();
    assert!(!cfg.read_only);
  }
}
