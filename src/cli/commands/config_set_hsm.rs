use std::{fs, io::Write, path::PathBuf};

use anyhow::Error;
use directories::ProjectDirs;
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;
use toml_edit::{DocumentMut, value};

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

pub async fn exec(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  new_hsm: &String,
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
    .unwrap_or_default();

  settings_hsm_available_vec
    .retain(|role| !role.eq("offline_access") && !role.eq("uma_authorization"));

  // VALIDATION
  // 'hsm_available' config param is empty or does not exists (an admin user is running manta)
  // and 'hsm_group' has a value, then we fetch all HSM groups from CSM and check the user is
  // asking to put a valid HSM group in the configuration file
  let hsm_available_vec = if settings_hsm_available_vec.is_empty() {
    backend
      .get_all_groups(shasta_token)
      .await
      .unwrap()
      .into_iter()
      .map(|hsm_group| hsm_group.label)
      .collect::<Vec<String>>()
  } else {
    settings_hsm_available_vec
  };

  validate_hsm_group_and_hsm_available_config_params(
    new_hsm,
    &hsm_available_vec,
  );

  // All good, we are safe to update 'hsm_group' config param
  log::info!("Changing configuration to use HSM GROUP {}", new_hsm);

  doc["hsm_group"] = value(new_hsm);

  // Update configuration file content
  let mut manta_configuration_file = std::fs::OpenOptions::new()
    .write(true)
    .truncate(true)
    .open(path_to_manta_configuration_file)
    .unwrap();

  // Write updated content to configuration file
  manta_configuration_file
    .write_all(doc.to_string().as_bytes())
    .unwrap();
  manta_configuration_file.flush().unwrap();

  println!("Target HSM group set to '{new_hsm}'");

  Ok(())
}

pub fn validate_hsm_group_and_hsm_available_config_params(
  hsm_group: &String,
  hsm_available_vec: &[String],
) {
  if !hsm_available_vec.contains(hsm_group) {
    eprintln!(
      "HSM group provided ({}) not valid, please choose one of the following options: {:?}",
      hsm_group, hsm_available_vec
    );
    std::process::exit(1);
  }
}
