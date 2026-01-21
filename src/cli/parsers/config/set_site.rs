use anyhow::Error;
use clap::ArgMatches;

use crate::cli::commands::config_set_site;

pub async fn process_subcommand(
  cli_config_set_site: &ArgMatches,
) -> Result<(), Error> {
  let new_site_opt: Option<&String> = cli_config_set_site.get_one("SITE_NAME");

  config_set_site::exec(new_site_opt).await
}
