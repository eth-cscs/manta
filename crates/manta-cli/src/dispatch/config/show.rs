//! Implements the `manta config show` command.

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
/// current site).
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
  let hsm_group_available_opt = if let Some(client) = client {
    match client
      .openapi
      .get_available_groups(client.site_name())
      .await
      .into_anyhow()
    {
      Ok(groups) => Some(groups),
      Err(e) => {
        tracing::warn!("Failed to fetch available HSM groups: {}", e);
        None
      }
    }
  } else {
    None
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
    current_hsm: settings_hsm_group,
  };

  config_summary::print(&summary, output_opt)?;

  Ok(())
}
