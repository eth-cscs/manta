use anyhow::Error;

use crate::{
  cli::commands::config_unset_parent_hsm,
  common::authentication::get_api_token,
  manta_backend_dispatcher::StaticBackendDispatcher,
};

pub async fn process_subcommand(
  backend: &StaticBackendDispatcher,
  site_name: &str,
) -> Result<(), Error> {
  let shasta_token = get_api_token(&backend, &site_name).await?;

  config_unset_parent_hsm::exec(backend, &shasta_token).await
}
