use anyhow::Error;
use clap::ArgMatches;
use toml_edit::value;

use crate::common::config::{read_config_toml, write_config_toml};

pub fn exec(cli_config_set_log: &ArgMatches) -> Result<(), Error> {
  let log_level: &String = cli_config_set_log
    .get_one("LOG_LEVEL")
    .ok_or_else(|| Error::msg("Error"))?;

  set_log(log_level)
}

fn set_log(new_log_level_opt: &str) -> Result<(), Error> {
  let (path, mut doc) = read_config_toml()?;

  log::info!("Changing log verbosity level to {}", new_log_level_opt);

  doc["log"] = value(new_log_level_opt);

  write_config_toml(&path, &doc)?;

  match doc.get("log") {
    Some(log_level) => {
      println!("log verbosity set to {log_level}")
    }
    None => log::error!(
      "'log' key missing from config after \
       writing — this should not happen"
    ),
  }

  Ok(())
}
