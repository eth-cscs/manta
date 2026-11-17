//! CLI context struct threaded through `manta-cli`'s call stack.
//!
//! Every dispatch handler in `crate::dispatch::*` receives an
//! `&AppContext` as its first parameter. It carries: the resolved
//! site name, the manta-server URL the CLI is bound to, optional
//! poll-tuning knobs from `cli.toml`, and the raw `config::Config`
//! handle for fields the typed struct doesn't surface.
//!
//! Handlers never touch a token: `process_cli` sets the token provider
//! here and [`crate::http_client::MantaClient::from_app_ctx`] hands it
//! to the client, which gets a new token for every request.
//!
//! The server's analogous `InfraContext` (with backend dispatcher and
//! per-site URLs) lives in `manta_server::server::common::app_context`
//! — it depends on `StaticBackendDispatcher`, which the CLI never
//! touches.

use config::Config;

/// Top-level CLI context, passed as `&AppContext` through CLI
/// handlers and commands.
///
/// # Example
///
/// Typical handler shape: build a client, call an OpenAPI method.
///
/// ```ignore
/// use crate::common::app_context::AppContext;
/// use crate::http_client::{MantaClient, OpenApiResultExt};
///
/// async fn list_groups(ctx: &AppContext<'_>) -> anyhow::Result<()> {
///   let client = MantaClient::from_app_ctx(ctx)?;
///   let groups = client
///     .openapi
///     .get_groups(None, client.site_hint())
///     .await
///     .into_anyhow().await?;
///   println!("{groups:#?}");
///   Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct AppContext<'a> {
  /// Resolved site (`--site` override, else `cli.toml`'s `site`), or
  /// `None` when neither was supplied. It's just the `X-Manta-Site`
  /// header on outbound `MantaClient` requests. When `None`,
  /// manta-server resolves the site from the groups and xnames each
  /// request names, or answers that it needs one; the client adopts a
  /// resolved site for the command's later calls. The purely-local
  /// `config set`/`config unset` commands need none.
  pub site_name: Option<&'a str>,
  /// URL of the manta HTTP server this CLI talks to. Required.
  pub manta_server_url: &'a str,
  /// Optional default group name from `cli.toml`'s
  /// `hsm_group`; threaded into the typed `*Params`'
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
  /// CLI-local read-only flag from `cli.toml`'s `read_only`. Fed into
  /// [`crate::common::read_only::read_only_gate`] at the top of
  /// [`crate::dispatch::process::process_cli`] to refuse
  /// backend-mutating verbs before any HTTP request leaves the
  /// process. `false` by default.
  pub read_only: bool,
  /// Where and how the CLI logs in to obtain its bearer token —
  /// `cli.toml`'s `oidc_*` keys with the CSCS defaults filled in.
  pub oidc: crate::common::oidc::OidcSettings<'a>,
  /// Raw loaded `cli.toml` settings; held alongside the parsed
  /// `CliConfiguration` so handlers can read fields (e.g. `log`)
  /// that don't live on the typed struct.
  pub settings: &'a Config,
  /// Supplies a new token for every request, set at the top of
  /// [`crate::dispatch::process::process_cli`] for every command that
  /// talks to the server. `None` for purely-local verbs.
  pub token_provider:
    Option<std::sync::Arc<crate::common::authentication::TokenProvider>>,
}

impl<'a> AppContext<'a> {
  /// The site this invocation targets, or a user-facing error when
  /// neither `--site` nor `cli.toml`'s `site` was supplied. Only for
  /// the few commands that need a site the server cannot resolve and
  /// want to say so before sending anything (`config set hsm`, whose
  /// group list is per site); everything else leaves a missing site to
  /// manta-server.
  ///
  /// # Errors
  ///
  /// Returns `Err` when `site_name` is `None` — the message names
  /// both ways to set it (`--site <name>` or the `site` key in
  /// `cli.toml`).
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
      read_only: false,
      oidc: crate::common::oidc::OidcSettings {
        issuer_url: "https://issuer.example",
        client_id: "test-client",
        headless: false,
      },
      settings,
      token_provider: None,
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
