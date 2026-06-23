//! Implements the `manta config show` command.

use anyhow::Error;
use config::Config;

use crate::http_client::{MantaClient, OpenApiResultExt};
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

  let hsm_group_available_opt = if shasta_token_opt.is_some() {
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
    current_site: client.site_name().to_string(),
    read_only: settings.get_bool("read_only").unwrap_or(false),
    groups_available: hsm_group_available_opt,
    current_hsm: settings_hsm_group,
  };

  config_summary::print(&summary, output_opt)?;

  Ok(())
}
