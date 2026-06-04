//! Implements the `manta config set parent-hsm` command.

use anyhow::Error;
use clap::ArgMatches;

use crate::commands::config::common;
use crate::http_client::MantaClient;

/// Set the parent HSM group in configuration.
pub async fn exec(
  cli_config_set_parent_hsm: &ArgMatches,
  client: &MantaClient,
  token: &str,
) -> Result<(), Error> {
  let new_parent_hsm: &String = cli_config_set_parent_hsm
    .get_one("HSM_GROUP_NAME")
    .ok_or_else(|| Error::msg("new parent hsm group not defined"))?;

  common::set_hsm_config_value(
    client,
    token,
    new_parent_hsm,
    "parent_hsm_group",
    "Parent HSM group",
  )
  .await
}
