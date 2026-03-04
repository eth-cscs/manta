use std::collections::HashMap;

use anyhow::{Context, Error};
use config::{Config, Value};
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;

use crate::{
  common::{authentication::get_api_token, config::get_config_file_path},
  manta_backend_dispatcher::StaticBackendDispatcher,
};

/// Prints Manta's configuration on screen
pub async fn exec(
  backend: &StaticBackendDispatcher,
  site_name: &str,
  settings: &Config,
) -> Result<(), Error> {
  let shasta_token = get_api_token(backend, site_name).await?;

  show(backend, Some(shasta_token), settings).await
}

async fn show(
  backend: &StaticBackendDispatcher,
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
    backend.get_group_name_available(&shasta_token).await.ok()
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
    get_config_file_path()
      .await
      .map(|p| p.to_string_lossy().to_string())
      .unwrap_or_else(|_| "<unknown>".to_string())
  );
  println!("Log level: {}", log_level);
  println!("Sites: {:?}", site_table.keys().collect::<Vec<&String>>());
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
