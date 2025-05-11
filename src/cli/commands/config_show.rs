use std::collections::HashMap;

use config::{Config, Value};
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

/// Prints Manta's configuration on screen
pub async fn exec(
  backend: &StaticBackendDispatcher,
  shasta_token_opt: Option<String>,
  /* _shasta_base_url: &str,
  _shasta_root_cert: &[u8], */
  settings: &Config,
) {
  // Read configuration file
  let log_level = settings.get_string("log").unwrap_or("error".to_string());
  let settings_hsm_group =
    settings.get_string("hsm_group").unwrap_or("".to_string());
  let settings_parent_hsm_group = settings
    .get_string("parent_hsm_group")
    .unwrap_or("".to_string());

  // let hsm_group_available: Vec<String> = get_hsm_name_available_from_jwt(shasta_token).await;
  let hsm_group_available_opt = if let Some(shasta_token) = shasta_token_opt {
    backend.get_group_name_available(&shasta_token).await.ok()
  } else {
    None
  };

  let site_table: HashMap<String, Value> = settings.get_table("sites").unwrap();

  // println!("\n\nSites: {:#?}", site_table);

  let site_name = settings.get_string("site").unwrap();

  // println!("\n\nsite:\n{:#?}", site);

  // Print configuration file content to stdout
  println!("Log level: {}", log_level);
  println!("Sites: {:?}", site_table.keys().collect::<Vec<&String>>());
  println!("Current site: {}", site_name);
  println!(
    "HSM available: {}",
    hsm_group_available_opt
      .unwrap_or(vec!["Could not get list of groups available".to_string()])
      .join(", ")
  );
  println!("Current HSM: {}", settings_hsm_group);
  println!("Parent HSM: {}", settings_parent_hsm_group);
}
