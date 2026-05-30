//! CLI context struct threaded through `manta-cli`'s call stack.
//!
//! The server's analogous `InfraContext` (with backend dispatcher and
//! per-site URLs) lives in `manta-server::common::app_context` — it
//! depends on `StaticBackendDispatcher` which the CLI never touches.

use crate::common::kafka::Kafka;
use config::Config;

/// Top-level CLI context, passed as `&AppContext` through CLI
/// handlers and commands.
#[derive(Debug)]
pub struct AppContext<'a> {
  /// Site name used to set the `X-Manta-Site` header on outbound
  /// `MantaClient` requests.
  pub site_name: &'a str,
  /// URL of the manta HTTP server this CLI talks to. Required.
  pub manta_server_url: &'a str,
  /// Optional default HSM group name from `cli.toml`'s
  /// `parent_hsm_group`; threaded into the typed `*Params`'
  /// `settings_hsm_group_name` field by every command that builds one.
  pub settings_hsm_group_name_opt: Option<&'a str>,
  /// Optional Kafka audit producer; constructed at startup from
  /// `cli.toml`'s `[auditor.kafka]` block. `None` disables CLI-side
  /// audit emission.
  pub kafka_audit_opt: Option<&'a Kafka>,
  /// Optional per-request HTTP timeout (seconds) for outbound
  /// `MantaClient` calls — read from `cli.toml`'s
  /// `request_timeout_secs`. Honoured by commands that build their
  /// `MantaClient` via `MantaClient::from_app_ctx` (today: the power
  /// command, which can run minutes against large clusters). Other
  /// commands keep the default no-timeout behaviour.
  pub request_timeout_secs: Option<u64>,
  /// Raw loaded `cli.toml` settings; held alongside the parsed
  /// `CliConfiguration` so handlers can read fields (e.g. `log`)
  /// that don't live on the typed struct.
  pub settings: &'a Config,
}
