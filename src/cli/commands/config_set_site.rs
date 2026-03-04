use std::{fs, io::Write};

use anyhow::{Context, Error, bail};
use clap::ArgMatches;
use toml_edit::{DocumentMut, Table, value};

use crate::common::config::get_default_manta_config_file_path;

pub async fn exec(cli_config_set_site: &ArgMatches) -> Result<(), Error> {
  let new_site_opt: Option<&String> = cli_config_set_site.get_one("SITE_NAME");

  set_site(new_site_opt).await
}

async fn set_site(new_site_opt: Option<&String>) -> Result<(), Error> {
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

  let site_available_table = doc["sites"]
    .as_table()
    .context("No 'sites' table in configuration file")?;

  // VALIDATION
  if site_available_table.is_empty() {
    bail!("No 'sites' in config file");
  }

  validate_site_and_site_available_config_params(
    new_site_opt.context("Site name argument is required")?,
    site_available_table,
  )?;

  // All goot, we are safe to update 'site' config param
  log::info!(
    "Changing configuration to use 'site' {}",
    new_site_opt.context("Site name argument is required")?
  );

  doc["site"] = value(new_site_opt.context("Site name argument is required")?);

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

  match doc.get("site") {
    Some(hsm_value) => println!("site set to {hsm_value}"),
    None => log::error!(
      "'site' key missing from config after writing — this should not happen"
    ),
  }

  Ok(())
}

fn validate_site_and_site_available_config_params(
  site: &String,
  site_available_table: &Table,
) -> Result<(), Error> {
  if !site_available_table.contains_key(site) {
    bail!(
      "Site provided ({}) not valid. Please \
       choose one from the list below:\n{}",
      site,
      site_available_table
        .iter()
        .map(|(key, _value)| key.to_string())
        .collect::<Vec<String>>()
        .join(", ")
    );
  }

  Ok(())
}
