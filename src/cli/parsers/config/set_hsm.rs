use anyhow::Error;
use clap::ArgMatches;

use crate::{
  cli::commands::config_set_hsm, common::authentication::get_api_token,
  manta_backend_dispatcher::StaticBackendDispatcher,
};

pub async fn process_subcommand(
  cli_config_set_hsm: &ArgMatches,
  backend: &StaticBackendDispatcher,
  site_name: &str,
) -> Result<(), Error> {
  let shasta_token = get_api_token(&backend, &site_name).await?;

  config_set_hsm::exec(
    &backend,
    &shasta_token,
    cli_config_set_hsm.get_one::<String>("HSM_GROUP_NAME"),
  )
  .await;

  Ok(())
}
