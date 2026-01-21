use anyhow::Error;
use clap::ArgMatches;

use crate::cli::commands::config_set_log;

pub async fn process_subcommand(
  cli_config_set_log: &ArgMatches,
) -> Result<(), Error> {
  let log_level: &String = cli_config_set_log
    .get_one("LOG_LEVEL")
    .ok_or_else(|| Error::msg("Error"))?;

  config_set_log::exec(log_level).await
}
