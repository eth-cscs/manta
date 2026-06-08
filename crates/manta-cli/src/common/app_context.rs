//! CLI context struct threaded through `manta-cli`'s call stack.
//!
//! The server's analogous `InfraContext` (with backend dispatcher and
//! per-site URLs) lives in `manta_server::server::common::app_context`
//! — it depends on `StaticBackendDispatcher`, which the CLI never
//! touches.

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
  /// Optional default group name from `cli.toml`'s
  /// `parent_group`; threaded into the typed `*Params`'
  /// `settings_group_name` field by every command that builds one.
  pub settings_group_name_opt: Option<&'a str>,
  /// Optional per-request HTTP timeout (seconds) for outbound
  /// `MantaClient` calls — read from `cli.toml`'s
  /// `request_timeout_secs`. Threaded into every
  /// [`crate::http_client::MantaClient::from_app_ctx`] call. `None`
  /// keeps the default no-timeout behaviour.
  pub request_timeout_secs: Option<u64>,
  /// Raw loaded `cli.toml` settings; held alongside the parsed
  /// `CliConfiguration` so handlers can read fields (e.g. `log`)
  /// that don't live on the typed struct.
  pub settings: &'a Config,
}
