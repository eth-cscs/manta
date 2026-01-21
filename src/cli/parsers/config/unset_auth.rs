use anyhow::Error;

use crate::cli::commands::config_unset_auth;

pub async fn process_subcommand() -> Result<(), Error> {
  config_unset_auth::exec().await
}
