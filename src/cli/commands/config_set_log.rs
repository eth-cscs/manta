use std::{fs, io::Write, path::PathBuf};

use directories::ProjectDirs;
use toml_edit::{value, DocumentMut};

pub async fn exec(new_log_level_opt: Option<&String>) {
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

  /* let settings = common::config_ops::get_configuration();

  let site_name = settings.get_string("site").unwrap();
  let site_detail_hashmap = settings.get_table("sites").unwrap();
  let site_detail_value = site_detail_hashmap
      .get(&site_name)
      .unwrap()
      .clone()
      .into_table()
      .unwrap();
  let site_available_vec = site_detail_hashmap
      .keys()
      .map(|site| site.clone())
      .collect::<Vec<String>>(); */

  // All goot, we are safe to update 'site' config param
  log::info!(
    "Changing log verbosity level to {}",
    new_log_level_opt.unwrap()
  );

  doc["log"] = value(new_log_level_opt.unwrap());

  // Update configuration file content
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

  match doc.get("log") {
    Some(log_level) => println!("log verbosity set to {log_level}"),
    None => eprintln!("ERROR: this should not happen"),
  }
}
