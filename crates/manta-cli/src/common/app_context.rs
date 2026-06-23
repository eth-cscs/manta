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
  /// Resolved site (`--site` override, else `cli.toml`'s `site`), or
  /// `None` when neither was supplied. It's just the `X-Manta-Site`
  /// header on outbound `MantaClient` requests; commands that reach
  /// the server obtain it via [`AppContext::require_site`]. The
  /// purely-local `config set`/`config unset` commands never call
  /// that, so they work with no site configured.
  pub site_name: Option<&'a str>,
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
  /// Mirror of `CliConfiguration.read_only`. The chokepoint in
  /// `crate::dispatch::process::process_cli` consults this before
  /// allowing any mutating verb to dispatch.
  pub read_only: bool,
}

impl<'a> AppContext<'a> {
  /// The site this invocation targets, or a user-facing error when
  /// neither `--site` nor `cli.toml`'s `site` was supplied. Call this
  /// at every point that needs to issue a request to `manta-server`;
  /// commands that touch only the local config file must not.
  pub fn require_site(&self) -> anyhow::Result<&'a str> {
    self.site_name.ok_or_else(|| {
      anyhow::anyhow!(
        "No site selected. Pass --site <name> or set `site` in cli.toml"
      )
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn ctx_with_site(site: Option<&'static str>) -> AppContext<'static> {
    // `settings` is borrowed, so leak a default Config for the test's
    // lifetime — cheap and avoids threading a real cli.toml through.
    let settings: &'static Config = Box::leak(Box::new(Config::default()));
    AppContext {
      site_name: site,
      manta_server_url: "https://example:8443",
      settings_group_name_opt: None,
      request_timeout_secs: None,
      power_poll_interval_secs: None,
      power_max_poll_attempts: None,
      sat_file_poll_interval_secs: None,
      sat_file_poll_budget_secs: None,
      sat_file_not_visible_budget_secs: None,
      settings,
      read_only: false,
    }
  }

  #[test]
  fn require_site_returns_the_name_when_set() {
    let ctx = ctx_with_site(Some("alps"));
    assert_eq!(ctx.require_site().unwrap(), "alps");
  }

  #[test]
  fn require_site_errors_when_unset() {
    let ctx = ctx_with_site(None);
    let err = ctx.require_site().unwrap_err().to_string();
    assert!(err.contains("No site selected"), "got: {err}");
  }
}
