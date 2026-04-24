use anyhow::Error;
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;

use crate::{
  common::config::{read_config_toml, write_config_toml},
  manta_backend_dispatcher::StaticBackendDispatcher,
};

/// Remove the parent HSM group from configuration.
pub async fn exec(
  backend: &StaticBackendDispatcher,
  token: &str,
) -> Result<(), Error> {
  unset_parent_hsm(backend, token).await
}

pub async fn unset_parent_hsm(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
) -> Result<(), Error> {
  let (path, mut doc) = read_config_toml()?;

  let mut settings_hsm_available_vec = backend
    .get_group_name_available(shasta_token)
    .await
    .unwrap_or_default();

  settings_hsm_available_vec
    .retain(|role| !role.eq("offline_access") && !role.eq("uma_authorization"));

  tracing::info!("Unset parent HSM group");
  doc.remove("parent_hsm_group");

  write_config_toml(&path, &doc)?;

  println!("Parent HSM group unset");

  Ok(())
}
