use std::{fs, io::Write, path::PathBuf};

use anyhow::{Context, Error};
use directories::ProjectDirs;
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;
use toml_edit::DocumentMut;

use crate::{
  common::authentication::get_api_token,
  manta_backend_dispatcher::StaticBackendDispatcher,
};

pub async fn exec(
  backend: &StaticBackendDispatcher,
  site_name: &str,
) -> Result<(), Error> {
  let shasta_token = get_api_token(backend, site_name).await?;

  unset_parent_hsm(backend, &shasta_token).await
}

pub async fn unset_parent_hsm(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
) -> Result<(), Error> {
  // Read configuration file

  // XDG Base Directory Specification
  let project_dirs = ProjectDirs::from(
    "local", /*qualifier*/
    "cscs",  /*organization*/
    "manta", /*application*/
  );

  let mut path_to_manta_configuration_file = PathBuf::from(
    project_dirs
      .context(
        "Could not determine config directory \
           (home directory may not be set)",
      )?
      .config_dir(),
  );

  path_to_manta_configuration_file.push("config.toml"); // ~/.config/manta/config is the file

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

  log::info!("Unset parent HSM group");
  doc.remove("parent_hsm_group");

  // Update configuration file content
  log::info!("Update config file");
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

  println!("Parent HSM group unset");

  Ok(())
}
