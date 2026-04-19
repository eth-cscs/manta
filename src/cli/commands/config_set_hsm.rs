use anyhow::Error;
use clap::ArgMatches;

use crate::{
  cli::commands::config_set_hsm_common,
  manta_backend_dispatcher::StaticBackendDispatcher,
};

/// Set the default HSM group in configuration.
pub async fn exec(
  cli_config_set_hsm: &ArgMatches,
  backend: &StaticBackendDispatcher,
  token: &str,
) -> Result<(), Error> {
  let new_hsm: &String = cli_config_set_hsm
    .get_one("HSM_GROUP_NAME")
    .ok_or_else(|| Error::msg("new hsm group not defined"))?;

  config_set_hsm_common::set_hsm_config_value(
    backend,
    token,
    new_hsm,
    "hsm_group",
    "Target HSM group",
  )
  .await
}
