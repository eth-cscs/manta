use anyhow::Error;

use crate::cli::commands::config_unset_hsm;

pub async fn process_subcommand() -> Result<(), Error> {
  config_unset_hsm::exec().await;

  Ok(())
}
