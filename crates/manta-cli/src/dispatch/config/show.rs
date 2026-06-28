//! Implements the `manta config show` command.
//!
//! Builds a [`ConfigSummary`] from the merged local settings and,
//! when a site is selected, augments it with the per-site available
//! groups fetched from `GET /groups/available`. Rendering is delegated
//! to [`crate::output::config_summary`].

use anyhow::Error;
use config::Config;

use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::output::config_summary::{self, ConfigSummary};
use manta_shared::common::config::get_cli_config_file_path;

/// Display the current manta configuration.
///
/// `client` is `Some` only when a site was selected (`--site` or
/// `cli.toml`'s `site`); without one we still print the local config,
/// just without the per-site, server-derived fields (available groups,
/// current site). A failure to fetch the available groups is logged at
/// warn level and the field is left empty rather than aborting the
/// whole `show`.
///
/// # Errors
///
/// Returns an error only if the renderer fails. The config file path
/// lookup falls back to `"<unknown>"`, individual `Config` lookups use
/// defaults, and the available-groups fetch is best-effort.
pub async fn exec(
  client: Option<&MantaClient>,
  settings: &Config,
  output_opt: Option<&str>,
) -> Result<(), Error> {
  let log_level = settings
    .get_string("log")
    .unwrap_or_else(|_| "error".to_string());
  // Absent or empty `hsm_group` both mean "no default group selected",
  // mirroring how `current_site` is `None` when unset.
  let settings_hsm_group = settings
    .get_string("hsm_group")
    .ok()
    .filter(|s| !s.is_empty());

  // Available groups are per-site, so fetch them only when we have a
  // site-bound (and authenticated) client.
  let hsm_group_available_opt = match client {
    Some(c) => c
      .openapi
      .get_available_groups(c.site_name())
      .await
      .into_anyhow()
      .inspect_err(|e| {
        tracing::warn!("Failed to fetch available HSM groups: {e}")
      })
      .ok(),
    None => None,
  };

  let summary = ConfigSummary {
    config_file: get_cli_config_file_path().map_or_else(
      |_| "<unknown>".to_string(),
      |p| p.to_string_lossy().to_string(),
    ),
    log_level,
    current_site: client.map(|c| c.site_name().to_string()),
    read_only: settings.get_bool("read_only").unwrap_or(false),
    groups_available: hsm_group_available_opt,
    current_group: settings_hsm_group,
  };

  config_summary::print(&summary, output_opt)?;

  Ok(())
}
