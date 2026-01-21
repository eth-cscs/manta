use anyhow::Error;
use clap::ArgMatches;

use crate::{
  cli::commands::config_set_parent_hsm, common::authentication::get_api_token,
  manta_backend_dispatcher::StaticBackendDispatcher,
};

pub async fn process_subcommand(
  cli_config_set_parent_hsm: &ArgMatches,
  backend: &StaticBackendDispatcher,
  site_name: &str,
) -> Result<(), Error> {
  let shasta_token = get_api_token(&backend, &site_name).await?;

  let new_parent_hsm: &String = cli_config_set_parent_hsm
    .get_one("HSM_GROUP_NAME")
    .ok_or_else(|| Error::msg("new parent hsm group not defined"))?;

  config_set_parent_hsm::exec(backend, &shasta_token, new_parent_hsm).await
}
