//! Implements the `manta config show` command.
//!
//! Builds a [`ConfigSummary`] from the merged local settings and,
//! when a session is available, the per-invocation JWT-derived facts
//! (`username`, `name`, `is_admin`, `accessible_groups`) attached to
//! [`crate::common::app_context::AppContext::session`]. The
//! `read_only` flag is sourced from `AppContext::read_only`, not the
//! JWT. Rendering is delegated to [`crate::output::config_summary`].

use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::output::config_summary::{self, ConfigSummary, SessionSummary};
use manta_shared::common::config::get_cli_config_file_path;

/// Display the current manta configuration.
///
/// The per-site, server-derived fields (current site, accessible
/// groups, admin/read-only flags) come from `ctx.session` — populated
/// for every authenticated command at the top of
/// [`crate::dispatch::process::process_cli`]. When no site is selected
/// (and therefore no session was built), the renderer prints the
/// local config alone.
///
/// # Errors
///
/// Returns an error only if the renderer fails. The config file path
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

  let session = ctx.session.as_ref().map(|s| SessionSummary {
    username: s.username.clone(),
    name: s.name.clone(),
    is_admin: s.is_admin,
    accessible_groups: s.accessible_groups.clone(),
  });

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
