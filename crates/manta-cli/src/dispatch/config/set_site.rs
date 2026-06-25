//! Implements the `manta config set site` command.

use anyhow::{Context, Error};
use clap::ArgMatches;
use toml_edit::value;

use crate::output::action_result;
use manta_shared::common::config::{read_config_toml, write_config_toml};

/// Set the active site in configuration.
pub fn exec(cli_config_set_site: &ArgMatches) -> Result<(), Error> {
  let new_site_opt: Option<&str> = cli_config_set_site
    .get_one::<String>("SITE_NAME")
    .map(String::as_str);

  set_site(new_site_opt)
}

fn set_site(new_site_opt: Option<&str>) -> Result<(), Error> {
  let (path, mut doc) = read_config_toml()?;

  let new_site = new_site_opt.context("Site name argument is required")?;

  // The server is the source of truth for valid sites — the CLI does no
  // local validation. Write the name; the server rejects an unknown site
  // on the next request that carries it via the `X-Manta-Site` header.
  tracing::info!("Changing configuration to use 'site' {}", new_site);

  doc["site"] = value(new_site);

  write_config_toml(&path, &doc)?;

  if let Some(hsm_value) = doc.get("site") {
    action_result::print(&format!("site set to {hsm_value}"), None)?;
  } else {
    tracing::error!(
      "'site' key missing from config after \
       writing — this should not happen"
    );
  }

  Ok(())
}
