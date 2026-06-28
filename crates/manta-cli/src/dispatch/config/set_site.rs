//! Implements the `manta config set site` command.
//!
//! Writes `site = "<name>"` to `cli.toml`. Server-side validation is
//! deferred until the next request that carries the `X-Manta-Site`
//! header — the CLI does not check the value locally.

use anyhow::Error;
use clap::ArgMatches;
use toml_edit::value;

use crate::output::action_result;
use manta_shared::common::config::{read_config_toml, write_config_toml};

/// Set the active site in configuration.
///
/// Consumes the clap matches for `config set site` (positional
/// `SITE_NAME`) and persists the value.
///
/// # Errors
///
/// Returns an error if the config file cannot be read or written, or
/// the renderer fails. The `SITE_NAME` positional is declared required
/// by clap, so its absence panics rather than returns.
pub fn exec(cli_config_set_site: &ArgMatches) -> Result<(), Error> {
  let new_site = cli_config_set_site
    .get_one::<String>("SITE_NAME")
    .expect("clap declares SITE_NAME as a required positional");

  let (path, mut doc) = read_config_toml()?;

  // The server is the source of truth for valid sites — the CLI does no
  // local validation. Write the name; the server rejects an unknown site
  // on the next request that carries it via the `X-Manta-Site` header.
  tracing::info!("Changing configuration to use 'site' {}", new_site);

  doc["site"] = value(new_site);

  write_config_toml(&path, &doc)?;

  action_result::print(&format!("site set to \"{new_site}\""), None)?;

  Ok(())
}
