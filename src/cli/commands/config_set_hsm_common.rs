use anyhow::{Context, Error, bail};
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;
use toml_edit::value;

use crate::{
  common::config::{read_config_toml, write_config_toml},
  manta_backend_dispatcher::StaticBackendDispatcher,
};

/// Sets an HSM group value in the manta configuration file.
///
/// `toml_key` is the TOML key to update
/// (e.g. `"hsm_group"` or `"parent_hsm_group"`).
/// `label` is a human-readable description used in log/print
/// messages (e.g. `"Target HSM group"` or
/// `"Parent HSM group"`).
pub async fn set_hsm_config_value(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  new_hsm: &str,
  toml_key: &str,
  label: &str,
) -> Result<(), Error> {
  let (path, mut doc) = read_config_toml()?;

  let mut settings_hsm_available_vec = backend
    .get_group_name_available(shasta_token)
    .await
    .unwrap_or_default();

  settings_hsm_available_vec
    .retain(|role| !role.eq("offline_access") && !role.eq("uma_authorization"));

  // VALIDATION
  // If 'hsm_available' is empty (admin user), fetch all HSM
  // groups from CSM to validate the requested group exists.
  let hsm_available_vec = if settings_hsm_available_vec.is_empty() {
    backend
      .get_all_groups(shasta_token)
      .await
      .context("Failed to fetch HSM groups")?
      .into_iter()
      .map(|hsm_group| hsm_group.label)
      .collect::<Vec<String>>()
  } else {
    settings_hsm_available_vec
  };

  validate_hsm_in_available(new_hsm, &hsm_available_vec)?;

  log::info!("Changing configuration to use {} '{}'", label, new_hsm);

  doc[toml_key] = value(new_hsm);

  write_config_toml(&path, &doc)?;

  println!("{label} set to '{new_hsm}'");

  Ok(())
}

fn validate_hsm_in_available(
  hsm_group: &str,
  hsm_available_vec: &[String],
) -> Result<(), Error> {
  if !hsm_available_vec.iter().any(|h| h == hsm_group) {
    bail!(
      "HSM group provided ({}) not valid, \
       please choose one of the following \
       options: {:?}",
      hsm_group,
      hsm_available_vec
    );
  }

  Ok(())
}
