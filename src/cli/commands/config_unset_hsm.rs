use std::{fs, io::Write, path::PathBuf};

use anyhow::{Context, Error};
use directories::ProjectDirs;
use toml_edit::DocumentMut;

pub async fn exec() -> Result<(), Error> {
  unset_hsm().await
}

async fn unset_hsm() -> Result<(), Error> {
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

  log::info!("Unset HSM group");
  doc.remove("hsm_group");

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

  println!("Target HSM group unset");

  Ok(())
}
