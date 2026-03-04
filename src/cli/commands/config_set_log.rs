use std::{fs, io::Write};

use anyhow::{Context, Error};
use clap::ArgMatches;
use toml_edit::{DocumentMut, value};

use crate::common::config::get_default_manta_config_file_path;

pub async fn exec(cli_config_set_log: &ArgMatches) -> Result<(), Error> {
  let log_level: &String = cli_config_set_log
    .get_one("LOG_LEVEL")
    .ok_or_else(|| Error::msg("Error"))?;

  set_log(log_level).await
}

async fn set_log(new_log_level_opt: &str) -> Result<(), Error> {
  let path_to_manta_configuration_file = get_default_manta_config_file_path()?;

  log::debug!(
    "Reading manta configuration from {}",
    &path_to_manta_configuration_file.to_string_lossy()
  );

  let config_file_content =
    fs::read_to_string(&path_to_manta_configuration_file)
      .context("Error reading configuration file")?;

  let mut doc = config_file_content
    .parse::<DocumentMut>()
    .context("Could not parse configuration file as TOML")?;

  // All goot, we are safe to update 'site' config param
  log::info!("Changing log verbosity level to {}", new_log_level_opt);

  doc["log"] = value(new_log_level_opt);

  // Update configuration file content
  let mut manta_configuration_file = std::fs::OpenOptions::new()
    .write(true)
    .truncate(true)
    .open(&path_to_manta_configuration_file)
    .context("Failed to open configuration file for writing")?;

  manta_configuration_file
    .write_all(doc.to_string().as_bytes())
    .context("Failed to write configuration file")?;

  manta_configuration_file
    .flush()
    .context("Failed to flush configuration file")?;

  match doc.get("log") {
    Some(log_level) => println!("log verbosity set to {log_level}"),
    None => log::error!(
      "'log' key missing from config after writing — this should not happen"
    ),
  }

  Ok(())
}
