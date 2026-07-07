//! Implements the `manta config set hsm` command.
//!
//! Validates the requested group against the user's accessible groups
//! (pre-fetched and filtered server-side, passed in via
//! `accessible_groups`) and writes `hsm_group = "<name>"` to
//! `cli.toml`.

use anyhow::{Error, bail};
use clap::ArgMatches;
use toml_edit::value;

use crate::http_client::MantaClient;
use crate::output::action_result;
use manta_shared::common::config::{read_config_toml, write_config_toml};

/// Set the default HSM group in configuration.
///
/// Consumes clap matches for `config set hsm` (positional
/// `HSM_GROUP_NAME`) and validates+writes the value via
/// `set_hsm_config_value`.
///
/// # Errors
///
/// Returns an error if `HSM_GROUP_NAME` is missing, the requested
/// group is not in the accessible list, or the config file cannot be
/// read or written.
pub async fn exec(
  cli_config_set_hsm: &ArgMatches,
  client: &MantaClient,
  token: &str,
  accessible_groups: &[String],
) -> Result<(), Error> {
  let new_hsm: &String = cli_config_set_hsm
    .get_one("HSM_GROUP_NAME")
    .ok_or_else(|| Error::msg("new hsm group not defined"))?;

  set_hsm_config_value(client, token, new_hsm, accessible_groups).await
}

/// Validate `new_hsm` against the user's accessible groups and persist
/// it to `cli.toml`. `_shasta_token` is unused — the client already
/// carries the bearer token; kept in the signature for symmetry with
/// other handlers.
async fn set_hsm_config_value(
  _client: &MantaClient,
  _shasta_token: &str,
  new_hsm: &str,
  accessible_groups: &[String],
) -> Result<(), Error> {
  let (path, mut doc) = read_config_toml()?;

  // Accessible groups are already filtered server-side and cached
  // on ctx.session; the function takes them as a parameter so it
  // doesn't need an AppContext reference.
  let settings_group_available_vec: Vec<String> = accessible_groups.to_vec();

  validate_group_in_available(new_hsm, &settings_group_available_vec)?;

  tracing::info!("Changing configuration to use target HSM group '{new_hsm}'");

  doc["hsm_group"] = value(new_hsm);

  write_config_toml(&path, &doc)?;

  action_result::print(&format!("Target HSM group set to '{new_hsm}'"), None)?;

  Ok(())
}

fn validate_group_in_available(
  hsm_group: &str,
  hsm_available_vec: &[String],
) -> Result<(), Error> {
  if !hsm_available_vec.iter().any(|h| h == hsm_group) {
    bail!(
      "HSM group provided ({hsm_group}) not valid, \
       please choose one of the following \
       options: {hsm_available_vec:?}"
    );
  }

  Ok(())
}
