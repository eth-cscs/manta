//! Implements the `manta config show` command.

use std::collections::HashMap;

use anyhow::{Context, Error};
use config::{Config, Value};

use crate::http_client::MantaClient;
use crate::output::config_summary::{self, ConfigSummary};
use manta_shared::common::config::get_cli_config_file_path;

/// Display the current manta configuration.
pub async fn exec(
  client: &MantaClient,
  token: &str,
  settings: &Config,
  output_opt: Option<&str>,
) -> Result<(), Error> {
  show(client, Some(token.to_string()), settings, output_opt).await
}

async fn show(
  client: &MantaClient,
  shasta_token_opt: Option<String>,
  settings: &Config,
  output_opt: Option<&str>,
) -> Result<(), Error> {
  let log_level = settings
    .get_string("log")
    .unwrap_or_else(|_| "error".to_string());
  let settings_hsm_group = settings.get_string("hsm_group").unwrap_or_default();
  let settings_parent_hsm_group =
    settings.get_string("parent_hsm_group").unwrap_or_default();

  let hsm_group_available_opt = if let Some(shasta_token) = shasta_token_opt {
    match client.get_available_groups(&shasta_token).await {
      Ok(groups) => Some(groups),
      Err(e) => {
        tracing::warn!("Failed to fetch available HSM groups: {}", e);
        None
      }
    }
  } else {
    None
  };

  let site_table: HashMap<String, Value> = settings
    .get_table("sites")
    .context("'sites' table not found in config")?;

  let site_name = settings
    .get_string("site")
    .context("'site' key not found in config")?;

  let summary = ConfigSummary {
    config_file: get_cli_config_file_path().map_or_else(
      |_| "<unknown>".to_string(),
      |p| p.to_string_lossy().to_string(),
    ),
    log_level,
    sites: site_table.keys().cloned().collect(),
    current_site: site_name,
    groups_available: hsm_group_available_opt,
    current_hsm: settings_hsm_group,
    parent_hsm: settings_parent_hsm_group,
  };

  config_summary::print(&summary, output_opt)?;

  Ok(())
}
