//! Implements the `manta config show` command.
//!
//! Builds a [`ConfigSummary`] from the merged local settings and,
//! when a site is selected, the per-invocation JWT-derived facts
//! (`username`, `name`, `is_admin`, `accessible_groups`) of a
//! [`SessionContext`] built here. The `read_only` flag is sourced from `AppContext::read_only`, not the
//! JWT. Rendering is delegated to [`crate::output::config_summary`].

use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::common::session::SessionContext;
use crate::http_client::MantaClient;
use crate::output::config_summary::{self, ConfigSummary, SessionSummary};
use manta_shared::common::config::get_cli_config_file_path;

/// Display the current manta configuration.
///
/// The per-site, server-derived fields (accessible groups, admin flag,
/// user names) come from a [`SessionContext`] — one
/// `GET /groups/available` plus the claims of the token it carried.
/// `GET /groups/available` names no group, so the server can't
/// resolve a site for it: when no site is selected the session is
/// skipped and the renderer prints the local config alone.
///
/// # Errors
///
/// Returns an error if the session fetch or the renderer fails. The config file path
/// lookup falls back to `"<unknown>"`, individual `Config` lookups use
/// defaults.
pub async fn exec(
  ctx: &AppContext<'_>,
  output_opt: Option<&str>,
) -> Result<(), Error> {
  let log_level = ctx
    .settings
    .get_string("log")
    .unwrap_or_else(|_| "error".to_string());
  // Absent or empty `hsm_group` both mean "no default group selected".
  let settings_hsm_group = ctx
    .settings
    .get_string("hsm_group")
    .ok()
    .filter(|s| !s.is_empty());
  // The raw `site` from cli.toml — kept alongside the resolved
  // `current_site` so the renderer can flag `--site` overrides.
  let settings_site = ctx
    .settings
    .get_string("site")
    .ok()
    .filter(|s| !s.is_empty());

  let session = match ctx.site_name {
    Some(_) => {
      let client = MantaClient::from_app_ctx(ctx)?;
      let s = SessionContext::build(&client).await?;
      Some(SessionSummary {
        username: s.username,
        name: s.name,
        is_admin: s.is_admin,
        accessible_groups: s.accessible_groups,
      })
    }
    None => None,
  };

  let summary = ConfigSummary {
    config_file: get_cli_config_file_path().map_or_else(
      |_| "<unknown>".to_string(),
      |p| p.to_string_lossy().to_string(),
    ),
    log_level,
    current_site: ctx.site_name.map(str::to_string),
    configured_site: settings_site,
    read_only: ctx.read_only,
    session,
    current_group: settings_hsm_group,
  };

  config_summary::print(&summary, output_opt)?;

  Ok(())
}
