use anyhow::Error;
use clap::ArgMatches;

use crate::{
  cli::commands::config_set_hsm_common,
  manta_backend_dispatcher::StaticBackendDispatcher,
};

/// Set the parent HSM group in configuration.
pub async fn exec(
  cli_config_set_parent_hsm: &ArgMatches,
  backend: &StaticBackendDispatcher,
  token: &str,
) -> Result<(), Error> {
  let new_parent_hsm: &String = cli_config_set_parent_hsm
    .get_one("HSM_GROUP_NAME")
    .ok_or_else(|| Error::msg("new parent hsm group not defined"))?;

  config_set_hsm_common::set_hsm_config_value(
    backend,
    token,
    new_parent_hsm,
    "parent_hsm_group",
    "Parent HSM group",
  )
  .await
}
