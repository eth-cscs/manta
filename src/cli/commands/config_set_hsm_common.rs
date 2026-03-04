use std::{fs, io::Write};

use anyhow::{Context, Error, bail};
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;
use toml_edit::{DocumentMut, value};

use crate::{
  common::config::get_default_manta_config_file_path,
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
  let path_to_manta_configuration_file = get_default_manta_config_file_path()?;

  log::debug!(
    "Reading manta configuration from {}",
    &path_to_manta_configuration_file.to_string_lossy()
  );

  let config_file_content =
    fs::read_to_string(path_to_manta_configuration_file.clone())
      .context("Error reading configuration file")?;

  let mut doc = config_file_content
    .parse::<DocumentMut>()
    .context("Could not parse configuration file as TOML")?;

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

  // Write updated content to configuration file
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
