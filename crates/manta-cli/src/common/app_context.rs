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
  /// uses the per-client defaults set in
  /// [`crate::http_client::MantaClient::new_with_timeout`].
  pub request_timeout_secs: Option<u64>,
  /// Override (seconds) for the `manta power` poll interval. `None`
  /// keeps the dispatcher's compiled default.
  pub power_poll_interval_secs: Option<u64>,
  /// Override for the `manta power` max poll attempts. `None` keeps
  /// the dispatcher's compiled default.
  pub power_max_poll_attempts: Option<u32>,
  /// Override (seconds) for `manta apply sat-file`'s session poll
  /// interval. `None` keeps the dispatcher's compiled default.
  pub sat_file_poll_interval_secs: Option<u64>,
  /// Override (seconds) for the SAT-file monitor loop's hard cap.
  /// `None` keeps the dispatcher's compiled default.
  pub sat_file_poll_budget_secs: Option<u64>,
  /// Override (seconds) for the SAT-file "session not yet visible"
  /// cap. `None` keeps the dispatcher's compiled default.
  pub sat_file_not_visible_budget_secs: Option<u64>,
  /// Raw loaded `cli.toml` settings; held alongside the parsed
  /// `CliConfiguration` so handlers can read fields (e.g. `log`)
  /// that don't live on the typed struct.
  pub settings: &'a Config,
}
