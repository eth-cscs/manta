use anyhow::{Context, Error, bail};
use clap::ArgMatches;
use toml_edit::{Table, value};

use crate::common::config::{read_config_toml, write_config_toml};

/// Set the active site in configuration.
pub fn exec(cli_config_set_site: &ArgMatches) -> Result<(), Error> {
  let new_site_opt: Option<&str> = cli_config_set_site
    .get_one::<String>("SITE_NAME")
    .map(String::as_str);

  set_site(new_site_opt)
}

fn set_site(new_site_opt: Option<&str>) -> Result<(), Error> {
  let (path, mut doc) = read_config_toml()?;

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

  let new_site = new_site_opt.context("Site name argument is required")?;

  tracing::info!("Changing configuration to use 'site' {}", new_site);

  doc["site"] = value(new_site);

  write_config_toml(&path, &doc)?;

  match doc.get("site") {
    Some(hsm_value) => println!("site set to {hsm_value}"),
    None => tracing::error!(
      "'site' key missing from config after \
       writing — this should not happen"
    ),
  }

  Ok(())
}

fn validate_site_and_site_available_config_params(
  site: &str,
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
