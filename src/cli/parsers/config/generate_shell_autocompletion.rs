use std::path::PathBuf;

use anyhow::Error;

use clap::{ArgMatches, Command};

use crate::cli::commands::config_gen_autocomplete;

pub async fn process_subcommand(
  cli: Command,
  cli_config_generate_autocomplete: &ArgMatches,
) -> Result<(), Error> {
  let shell_opt: Option<String> =
    cli_config_generate_autocomplete.get_one("shell").cloned();

  let path_opt: Option<PathBuf> =
    cli_config_generate_autocomplete.get_one("path").cloned();

  config_gen_autocomplete::exec(cli, shell_opt, path_opt)
}
