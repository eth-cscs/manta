use anyhow::Error;
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;

use crate::{
  common::{
    authentication::get_api_token,
    config::{read_config_toml, write_config_toml},
  },
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
  let (path, mut doc) = read_config_toml()?;

  let mut settings_hsm_available_vec = backend
    .get_group_name_available(shasta_token)
    .await
    .unwrap_or_default();

  settings_hsm_available_vec
    .retain(|role| !role.eq("offline_access") && !role.eq("uma_authorization"));

  log::info!("Unset parent HSM group");
  doc.remove("parent_hsm_group");

  write_config_toml(&path, &doc)?;

  println!("Parent HSM group unset");

  Ok(())
}
