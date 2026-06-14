//! Implements the `manta config set hsm` command.

use anyhow::{Context, Error, bail};
use clap::ArgMatches;
use toml_edit::value;

use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::output::action_result;
use manta_shared::common::config::{read_config_toml, write_config_toml};

/// Set the default HSM group in configuration.
pub async fn exec(
  cli_config_set_hsm: &ArgMatches,
  client: &MantaClient,
  token: &str,
) -> Result<(), Error> {
  let new_hsm: &String = cli_config_set_hsm
    .get_one("HSM_GROUP_NAME")
    .ok_or_else(|| Error::msg("new hsm group not defined"))?;

  set_hsm_config_value(client, token, new_hsm).await
}

async fn set_hsm_config_value(
  client: &MantaClient,
  _shasta_token: &str,
  new_hsm: &str,
) -> Result<(), Error> {
  let (path, mut doc) = read_config_toml()?;

  let mut settings_group_available_vec = client
    .openapi
    .get_available_groups(client.site_name())
    .await
    .into_anyhow()
    .unwrap_or_default();

  settings_group_available_vec
    .retain(|role| !role.eq("offline_access") && !role.eq("uma_authorization"));

  // If 'group_available' is empty (admin user), fetch every group the
  // server exposes via `GET /groups` so the requested group can be
  // validated.
  let group_available_vec = if settings_group_available_vec.is_empty() {
    client
      .openapi
      .get_groups(None, client.site_name())
      .await
      .into_anyhow()
      .context("Failed to fetch HSM groups")?
      .into_iter()
      .map(|hsm_group| hsm_group.label)
      .collect::<Vec<String>>()
  } else {
    settings_group_available_vec
  };

  validate_group_in_available(new_hsm, &group_available_vec)?;

  tracing::info!("Changing configuration to use target HSM group '{new_hsm}'");

  doc["hsm_group"] = value(new_hsm);

  write_config_toml(&path, &doc)?;

  action_result::print(&format!("Target HSM group set to '{new_hsm}'"), None)?;

  Ok(())
}

fn validate_group_in_available(
  hsm_group: &str,
  hsm_available_vec: &[String],
) -> Result<(), Error> {
  if !hsm_available_vec.iter().any(|h| h == hsm_group) {
    bail!(
      "HSM group provided ({hsm_group}) not valid, \
       please choose one of the following \
       options: {hsm_available_vec:?}"
    );
  }

  Ok(())
}
