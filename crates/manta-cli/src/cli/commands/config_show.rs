//! Implements the `manta config show` command.

use std::collections::HashMap;

use anyhow::{Context, Error};
use config::{Config, Value};

use crate::cli::http_client::MantaClient;
use crate::common::config::get_cli_config_file_path;

/// Display the current manta configuration.
pub async fn exec(
  client: &MantaClient,
  token: &str,
  settings: &Config,
) -> Result<(), Error> {
  show(client, Some(token.to_string()), settings).await
}

async fn show(
  client: &MantaClient,
  shasta_token_opt: Option<String>,
  settings: &Config,
) -> Result<(), Error> {
  // Read configuration file
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

  // Print configuration file content to stdout
  println!(
    "Configuration file: {}",
    get_cli_config_file_path()
      .map(|p| p.to_string_lossy().to_string())
      .unwrap_or_else(|_| "<unknown>".to_string())
  );
  println!("Log level: {}", log_level);
  println!(
    "Sites: {}",
    site_table
      .keys()
      .cloned()
      .collect::<Vec<String>>()
      .join(", ")
  );
  println!("Current site: {}", site_name);
  println!(
    "Groups available: {}",
    hsm_group_available_opt
      .unwrap_or_else(|| vec![
        "Could not get list of groups available".to_string()
      ])
      .join(", ")
  );
  println!("Current HSM: {}", settings_hsm_group);
  println!("Parent HSM: {}", settings_parent_hsm_group);

  Ok(())
}
