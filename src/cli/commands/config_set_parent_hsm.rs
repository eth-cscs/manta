use std::{fs, io::Write, path::PathBuf};

use anyhow::Error;
use clap::ArgMatches;
use directories::ProjectDirs;
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;
use toml_edit::{value, DocumentMut};

use crate::{common::authentication::get_api_token, manta_backend_dispatcher::StaticBackendDispatcher};

pub async fn exec(
  cli_config_set_parent_hsm: &ArgMatches,
  backend: &StaticBackendDispatcher,
  site_name: &str,
) -> Result<(), Error> {
  let shasta_token = get_api_token(&backend, &site_name).await?;

  let new_parent_hsm: &String = cli_config_set_parent_hsm
    .get_one("HSM_GROUP_NAME")
    .ok_or_else(|| Error::msg("new parent hsm group not defined"))?;

  set_parent_hsm(backend, &shasta_token, new_parent_hsm).await
}

pub async fn set_parent_hsm(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  new_hsm: &String,
  // all_hsm_available_vec: &[String],
) -> Result<(), Error> {
  // Read configuration file

  // XDG Base Directory Specification
  let project_dirs = ProjectDirs::from(
    "local", /*qualifier*/
    "cscs",  /*organization*/
    "manta", /*application*/
  );

  let mut path_to_manta_configuration_file =
    PathBuf::from(project_dirs.unwrap().config_dir());

  path_to_manta_configuration_file.push("config.toml"); // ~/.config/manta/config is the file

  log::debug!(
    "Reading manta configuration from {}",
    &path_to_manta_configuration_file.to_string_lossy()
  );

  let config_file_content =
    fs::read_to_string(path_to_manta_configuration_file.clone())
      .expect("Error reading configuration file");

  let mut doc = config_file_content
    .parse::<DocumentMut>()
    .expect("ERROR: could not parse configuration file to TOML");

  let mut settings_hsm_available_vec = backend
    .get_group_name_available(shasta_token)
    .await
    .unwrap_or(Vec::new());

  settings_hsm_available_vec
    .retain(|role| !role.eq("offline_access") && !role.eq("uma_authorization"));

  // VALIDATION
  // Validate user has access to new HSM group
  // 'hsm_available' config param is empty or does not exists (an admin user is running manta)
  // and 'hsm_group' has a value, then we fetch all HSM groups from CSM and check the user is
  // asking to put a valid HSM group in the configuration file
  let hsm_available_vec = if settings_hsm_available_vec.is_empty() {
    backend
      .get_all_groups(shasta_token)
      .await
      .unwrap()
      .into_iter()
      .map(|hsm_group_value| hsm_group_value.label)
      .collect::<Vec<String>>()
  } else {
    settings_hsm_available_vec
  };

  validate_hsm_group_and_hsm_available_config_params(
    new_hsm,
    &hsm_available_vec,
  )?;

  // All good, we are safe to update 'hsm_group' config param
  log::info!("Changing configuration to use HSM GROUP {}", new_hsm);

  // Update parent hsm in config file
  doc["parent_hsm_group"] = value(new_hsm);

  log::info!("New HSM group set successfully");

  // Update configuration file content
  let mut manta_configuration_file = std::fs::OpenOptions::new()
    .write(true)
    .truncate(true)
    .open(path_to_manta_configuration_file)
    .unwrap();

  manta_configuration_file
    .write_all(doc.to_string().as_bytes())
    .unwrap();
  manta_configuration_file.flush().unwrap();

  match doc.get("parent_hsm_group") {
    Some(hsm_value) => println!("Parent HSM group set to {hsm_value}"),
    None => println!("Parent HSM group unset"),
  }

  Ok(())
}

pub fn validate_hsm_group_and_hsm_available_config_params(
  hsm_group: &String,
  hsm_available_vec: &[String],
) -> Result<(), Error> {
  if !hsm_available_vec.contains(hsm_group) {
    Err(Error::msg(format!(
      "HSM group provided ({}) not valid, please choose one of the following options: {:?}",
      hsm_group, hsm_available_vec
    )))
  } else {
    Ok(())
  }
}
