use anyhow::Error;
use config::Config;

use crate::{
  cli::commands::config_show, common::authentication::get_api_token,
  manta_backend_dispatcher::StaticBackendDispatcher,
};

pub async fn process_subcommand(
  backend: &StaticBackendDispatcher,
  site_name: &str,
  settings: &Config,
) -> Result<(), Error> {
  let shasta_token = get_api_token(&backend, &site_name).await?;

  config_show::exec(backend, Some(shasta_token), settings).await
}
