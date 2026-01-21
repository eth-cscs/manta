use std::{fs, io::Write, path::PathBuf};

use anyhow::Error;
use directories::ProjectDirs;
use toml_edit::DocumentMut;

pub async fn exec() -> Result<(), Error> {
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

  log::info!("Unset HSM group");
  doc.remove("hsm_group");

  // Update configuration file content
  log::info!("Update config file");
  let mut manta_configuration_file = std::fs::OpenOptions::new()
    .write(true)
    .truncate(true)
    .open(path_to_manta_configuration_file)
    .unwrap();

  manta_configuration_file
    .write_all(doc.to_string().as_bytes())
    .unwrap();
  manta_configuration_file.flush().unwrap();

  println!("Target HSM group unset");

  Ok(())
}
