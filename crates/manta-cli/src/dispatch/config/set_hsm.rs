//! Implements the `manta config set hsm` command.
//!
//! Validates the requested group against the user's available groups
//! (or, for admins with an empty `groups_available` claim, against the
//! full `GET /groups` list) and writes `hsm_group = "<name>"` to
//! `cli.toml`. The keycloak service roles `offline_access` and
//! `uma_authorization` are filtered out since they appear in the role
//! list but are not real HSM groups.

use anyhow::{Context, Error, bail};
use clap::ArgMatches;
use toml_edit::value;

use crate::http_client::{MantaClient, OpenApiResultExt};
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
/// Returns an error if `HSM_GROUP_NAME` is missing, the validation
/// fetch fails (admin fallback path), the requested group is not in
/// the available list, or the config file cannot be read or written.
pub async fn exec(
  cli_config_set_hsm: &ArgMatches,
  client: &MantaClient,
  token: &str,
) -> Result<(), Error> {
  let new_hsm: &String = cli_config_set_hsm
    .get_one("HSM_GROUP_NAME")
    .ok_or_else(|| Error::msg("new hsm group not defined"))?;

  set_hsm_config_value(client, token, new_hsm).await
}

/// Validate `new_hsm` against the user's available groups and persist
/// it to `cli.toml`. `_shasta_token` is unused — the client already
/// carries the bearer token; kept in the signature for symmetry with
/// other handlers.
async fn set_hsm_config_value(
  client: &MantaClient,
  _shasta_token: &str,
  new_hsm: &str,
) -> Result<(), Error> {
  let (path, mut doc) = read_config_toml()?;

  let mut settings_group_available_vec = client
    .openapi
    .get_available_groups(client.site_name())
    .await
    .into_anyhow()
    .unwrap_or_default();

  settings_group_available_vec
    .retain(|role| !role.eq("offline_access") && !role.eq("uma_authorization"));

  // If 'group_available' is empty (admin user), fetch every group the
  // server exposes via `GET /groups` so the requested group can be
  // validated.
  let group_available_vec = if settings_group_available_vec.is_empty() {
    client
      .openapi
      .get_groups(None, client.site_name())
      .await
      .into_anyhow()
      .context("Failed to fetch HSM groups")?
      .into_iter()
      .map(|hsm_group| hsm_group.label)
      .collect::<Vec<String>>()
  } else {
    settings_group_available_vec
  };

  validate_group_in_available(new_hsm, &group_available_vec)?;

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
