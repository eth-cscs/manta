use std::{fs, io::Write, path::PathBuf};

use directories::ProjectDirs;
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;
use toml_edit::DocumentMut;

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

pub async fn exec(backend: &StaticBackendDispatcher, shasta_token: &str) {
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
  /* let mut settings_hsm_available_vec =
  jwt_ops::get_hsm_name_available(shasta_token).unwrap_or(Vec::new()); */

  /* let mut settings_hsm_available_vec = jwt_ops::get_claims_from_jwt_token(shasta_token)
  .unwrap()
  .pointer("/realm_access/roles")
  .unwrap_or(&serde_json::json!([]))
  .as_array()
  .unwrap()
  .iter()
  .map(|role_value| role_value.as_str().unwrap().to_string())
  .collect::<Vec<String>>(); */

  settings_hsm_available_vec
    .retain(|role| !role.eq("offline_access") && !role.eq("uma_authorization"));

  log::info!("Unset parent HSM group");
  doc.remove("parent_hsm_group");

  // Update configuration file content
  log::info!("Update config file");
  let mut manta_configuration_file = std::fs::OpenOptions::new()
    .write(true)
    .truncate(true)
    .open(path_to_manta_configuration_file)
    .unwrap();

  /* let mut output = File::create(path_to_manta_configuration_file).unwrap();
  write!(output, "{}", doc.to_string()); */

  manta_configuration_file
    .write_all(doc.to_string().as_bytes())
    .unwrap();
  manta_configuration_file.flush().unwrap();

  println!("Parent HSM group unset");
}
